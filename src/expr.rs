use std::f64;
use std::fmt;
use std::iter::Peekable;
use std::mem;
use crate::Context;
use crate::ops;
use crate::ops::{OpFunction, OperatorRef};
use crate::tokens::{Token, Tokenizer};


// Expressions are represented as a tree of nodes.
#[derive(Debug)]
pub enum ExpressionNode {
    Constant { value: f64 },
    Operator { op: OperatorRef, args: Vec<ExpressionNode> },
    Function { name: String,    args: Vec<ExpressionNode> },
}


// A user defined function consists of an expression tree plus list of parameter names.
#[derive(Debug)]
pub struct Function {
    expression: ExpressionNode,
    args: Vec<String>,
}


// Local context used while evaluating a function.
struct FunctionFrame<'a> {

    // Backlink to the global execution context.
    context: &'a Context,
    
    // Parameter names and values for the currently executing function.
    local_names: &'a Vec<String>,
    local_values: Vec<f64>,

    // Track recursion depth, so we can error out if it goes too far.
    recursion_count: u32,
}


const MAX_RECURSION: u32 = 256;


// Expression tree formatter, useful for debugging and unit tests.
impl fmt::Display for ExpressionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_args(args: &Vec<ExpressionNode>) -> String {
            args.iter()
                .map(|arg| { format!("{}", arg) })
                .collect::<Vec<String>>()
                .join(",")
        }

        match self {
            ExpressionNode::Constant{ value      } => write!(f, "{}", value),
            ExpressionNode::Operator{ op,   args } => write!(f, "{}({})", op.name, format_args(args)),
            ExpressionNode::Function{ name, args } => write!(f, "{}({})", name,    format_args(args)),
        }
    }
}


// The parser turns a series of tokens into an expression tree.
struct Parser
{
    current: Option<ExpressionNode>,
    stack: Vec<(OperatorRef, Option<ExpressionNode>)>,
}


impl Parser {
    // Pushes a numeric constant onto the stack.
    fn push_constant(&mut self, value: f64) -> Result<(), String> {
        if self.current.is_some() {
            return Err(format!("Invalid expression: expecting operator but got '{}'.", value));
        }

        self.current = Some(ExpressionNode::Constant { value });
        
        Ok(())
    }


    // Pushes a symbol reference (variable or function call) onto the stack.
    fn push_symbol(&mut self, symbol: &str, tokenizer: &mut Peekable<Tokenizer>) -> Result<(), String> {
        if self.current.is_some() {
            return Err(format!("Invalid expression: expecting operator but got '{}'.", symbol));
        }

        let args = Parser::parse_arguments(tokenizer)?;

        match ops::find_operator(symbol) {
            Some(op) => {
                if args.len() != op.arity as usize {
                    return Err(format!("Wrong number of arguments for {}(): expected {} but got {}.", op.name, op.arity, args.len()));
                }
                
                self.current = Some(ExpressionNode::Operator { op, args });
            },

            None => {
                self.current = Some(ExpressionNode::Function { name: String::from(symbol), args });
            },
        }

        Ok(())
    }


    // Pushes an operator onto the stack, performing a shift/reduce loop based on precedence.
    fn push_operator(&mut self, mut op: OperatorRef) -> Result<(), String> {
        // Turn binary subtraction into unary negation if there is no current value.
        if op == "-" && self.current.is_none() {
            op = &ops::NEGATE;
        }

        // Reduce the operator stack according to precedence.
        if op != "(" {
            let op_precedence = op.precedence as u32;

            while let Some((stack_op, _)) = self.stack.last() {
                let mut stack_precedence = stack_op.precedence as u32;

                if stack_op.arity != 1 {
                    if op.arity == 1 {
                        break;
                    }

                    if !stack_op.is_right_associative {
                        stack_precedence = stack_precedence + 1;
                    }
                }

                if op_precedence >= stack_precedence {
                    break;
                }

                // Pop from the stack, taking ownership where we were previously examining a borrow.
                let (stack_op, stack_value) = self.stack.pop().unwrap();

                match stack_op.arity {
                    1 => {
                        // Unary operator.
                        match (self.current.take(), stack_value) {
                            (Some(current), None) => {
                                self.current = Some(ExpressionNode::Operator { op: stack_op, args: vec![ current ] });
                            },
                            _ => return Err(format!("Invalid expression: unary {} operator is missing an operand.", stack_op.name))
                        }
                    },

                    2 => {
                        // Binary operator - or it could be adjacent ? and : which combine to form a ternary.
                        match (self.current.take(), stack_value) {
                            (Some(current), Some(stack)) => {
                                self.current = Some(Parser::binary_or_ternary(stack_op, stack, current));
                            },
                            _ => return Err(format!("Invalid expression: binary {} operator is missing an operand.", stack_op.name))
                        }
                    },
                    
                    _ => {
                        // Match open and close braces.
                        if stack_op != "(" || stack_value.is_some() {
                            return Err(String::from("Invalid expression: unexpected open parenthesis."));
                        }

                        if op == ")" {
                            if self.current.is_none() {
                                return Err(String::from("Invalid expression: unexpected close parenthesis."));
                            }
                            else {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        if op == ")" {
            // Swallow close braces, but not too many.
            if self.stack.is_empty() {
                return Err(String::from("Invalid expression: too many close parentheses."));
            }
        }
        else {
            // Push onto the stack.
            self.stack.push((op, self.current.take()));
        }
        
        Ok(())
    }


    // Decides whether we are dealing with a binary or ternary operator.
    fn binary_or_ternary(op: OperatorRef, x: ExpressionNode, mut y: ExpressionNode) -> ExpressionNode {
        if op == "?" {
            if let ExpressionNode::Operator{ op: y_op, args: ref mut y_args } = y {
                if y_op == ":" {
                    // Merge adjacent ? and : operators into a combined ternary operator.
                    return ExpressionNode::Operator {
                        op: &ops::TERNARY,
                        args: vec![ x, y_args.remove(0), y_args.remove(0) ]
                    };
                }
            }
        }

        // It's a regular binary operator.
        ExpressionNode::Operator { op, args: vec![ x, y ] }
    }


    // Parses the arguments of a function call.
    fn parse_arguments(tokenizer: &mut Peekable<Tokenizer>) -> Result<Vec<ExpressionNode>, String>
    {
        let mut args = vec![];

        if Parser::peek_operator(tokenizer, "(") {
            tokenizer.next();

            while !Parser::peek_operator(tokenizer, ")") {
                args.push(parse(tokenizer, true)?);
            }

            tokenizer.next();
        }

        Ok(args)
    }


    // Decide whether we've reached the end of the expression.
    fn is_finished(&self, tokenizer: &mut Peekable<Tokenizer>, is_nested: bool) -> bool {
        if let Some(Ok(Token::Text(","))) = tokenizer.peek() {
            // Commas always terminate.
            tokenizer.next();
            true
        }
        else if is_nested {
            // Parsing x or y from something like f(x, y(z)).
            // Closing parenthesis terminates only if there are no open parens on the stack.
            if Parser::peek_operator(tokenizer, ")") {
                !self.stack.iter().any(|op| { op.0 == "(" })
            }
            else {
                false
            }
        }
        else {
            // When parsing a top level expression, we're done if the input runs out.
            tokenizer.peek().is_none()
        }
    }


    // Checks whether the next token is the specified operator.
    fn peek_operator(tokenizer: &mut Peekable<Tokenizer>, opname: &str) -> bool {
        match tokenizer.peek() {
            Some(Ok(Token::Operator(op))) => op == opname,
            _ => false
        }
    }
}


// Expression parser entrypoint.
pub fn parse(tokenizer: &mut Peekable<Tokenizer>, is_nested: bool) -> Result<ExpressionNode, String>
{
    let mut parser = Parser {
        current: None,
        stack: vec![],
    };

    while !parser.is_finished(tokenizer, is_nested) {
        match tokenizer.next() {
            Some(token) => {
                match token? {
                    Token::Number(value) => parser.push_constant(value)?,
                    Token::Text(value)   => parser.push_symbol(value, tokenizer)?,
                    Token::Operator(op)  => parser.push_operator(op)?,
                }
            },
            None => return Err(String::from("Invalid expression: unexpected end of input."))
        }
    }

    // Collapse the stack.
    if parser.current.is_none() {
        return Err(String::from("Invalid expression: unexpected end of input."));
    }

    parser.push_operator(&ops::TERMINATOR)?;

    // We should now have just one item left.
    if parser.stack.len() != 1 {
        return Err(String::from("Invalid expression: unexpected end of input."));
    }

    Ok(parser.stack.pop().unwrap().1.unwrap())
}


// Expression evaluator entrypoint.
pub fn evaluate(expression: &ExpressionNode, context: &Context) -> Result<f64, String> {
    let frame = FunctionFrame {
        context,
        local_names: &vec![],
        local_values: vec![],
        recursion_count: 0
    };
    
    eval(expression, &frame)
}


// Recursive expression evaluator.
fn eval(expression: &ExpressionNode, frame: &FunctionFrame) -> Result<f64, String> {
    match expression {
        ExpressionNode::Constant{ value      } => Ok(*value),
        ExpressionNode::Operator{ op, args   } => evaluate_operator(op, args, frame),
        ExpressionNode::Function{ name, args } => evaluate_function(name, args, frame),
    }
}


fn evaluate_operator(op: OperatorRef, args: &Vec<ExpressionNode>, frame: &FunctionFrame) -> Result<f64, String> {
    match op.function {
        // No need for bound checks because the parser never outputs operators with wrong argument count.
        OpFunction::Nullary(function) => Ok(function()),
        OpFunction::Unary  (function) => Ok(function(eval(&args[0], frame)?)),
        OpFunction::Binary (function) => Ok(function(eval(&args[0], frame)?, eval(&args[1], frame)?)),

        OpFunction::Lazy(function) => {
            // Used by the ||, &&, and ?: operators. A function applied to the first
            // argument indicates which of the arguments to return. Unused arguments
            // are never evaluated. This lazy evaluation enables recursive functions.
            let arg0 = eval(&args[0], frame)?;
            let which_arg = function(arg0);
            
            if which_arg == 0 {
                Ok(arg0)
            }
            else {
                eval(&args[which_arg], frame)
            }
        },

        OpFunction::Invalid => Err(format!("Invalid use of {} operator.", op.name))
    }
}


fn evaluate_function(name: &String, args: &Vec<ExpressionNode>, frame: &FunctionFrame) -> Result<f64, String> {
    if let Some(which_local) = frame.local_names.iter().position(|local_name| { local_name == name }) {
        // Looking up a local function parameter.
        if args.is_empty() {
            Ok(frame.local_values[which_local])
        }
        else {
            Err(format!("Use of {}() as first class function is not supported.", name))
        }
    }
    else {
        // Calling a user defined function.
        match frame.context.functions.get(name) {
            Some(function) => {
                if args.len() != function.args.len() {
                    return Err(format!("Wrong number of arguments for {}(): expected {} but got {}.", name, function.args.len(), args.len()));
                }

                let mut child_args = Vec::with_capacity(args.len());
                
                for arg in args {
                    child_args.push(eval(arg, frame)?);
                }
     
                if frame.recursion_count > MAX_RECURSION {
                    return Err(String::from("Excessive recursion."));
                }
     
                let child_frame = FunctionFrame {
                    context: frame.context,
                    local_names: &function.args,
                    local_values: child_args,
                    recursion_count: frame.recursion_count + 1
                };
                
                eval(&function.expression, &child_frame)
            },
            
            None => Err(format!("Unknown value {}.", name))
        }
    }
}


// If given an expression of the form x=y or f(x)=y, rearranges it into a user defined function.
pub fn deconstruct_function_definition(expression: &mut ExpressionNode) -> Option<(Function, String)> {

    // Do we have an x=y expression?
    if let ExpressionNode::Operator{ op: assign_op, args: assign_args } = expression {
        if assign_op == "=" {

            // Is x of type Function?
            if let ExpressionNode::Function{ name: function_name, args: function_args } = &mut assign_args[0] {

                // Are all args passed to x themselves of type Function?
                let mut args = Vec::with_capacity(function_args.len());

                for arg in function_args {
                    match arg {
                        ExpressionNode::Function{ name: arg_name, args: arg_args } if arg_args.is_empty() => args.push(arg_name),
                        _ => return None
                    }
                }

                // Take ownership and return the deconstructed function data.
                let function_name = mem::take(function_name);
                let args = args.iter_mut().map(|arg| { mem::take(*arg) }).collect();
                let function_body = assign_args.pop().unwrap();

                return Some((Function { expression: function_body, args }, function_name));
            }
        }
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn parse_expressions() {
        test_parse("23", "23");
        test_parse("1+2", "+(1,2)");
        test_parse("1+2+3", "+(+(1,2),3)");
        test_parse("1*2+3", "+(*(1,2),3)");
        test_parse("1+2*3", "+(1,*(2,3))");
        test_parse("(1*2)+3", "+(*(1,2),3)");
        test_parse("(1+(2*3))", "+(1,*(2,3))");
        test_parse("1*(2+3)", "*(1,+(2,3))");
        test_parse("(1+2)*3", "*(+(1,2),3)");
        test_parse("1+2==3<sqrt(10)^5", "==(+(1,2),<(3,^(sqrt(10),5)))");
        test_parse("e", "e()");
        test_parse("sin(e())", "sin(e())");
        test_parse("max(1,2)", "max(1,2)");
        test_parse("foo(1,2,3,4)", "foo(1,2,3,4)");
        test_parse("foo(1,bar(x,y+foo(bar())))", "foo(1,bar(x(),+(y(),foo(bar()))))");
    }


    #[test]
    fn parse_unary_negate() {
        test_parse("-23", "-(23)");
        test_parse("1-2", "-(1,2)");
        test_parse("1--2", "-(1,-(2))");
        test_parse("-1-2", "-(-(1),2)");
        test_parse("-1--2---3", "-(-(-(1),-(2)),-(-(3)))");
    }


    #[test]
    fn parse_ternary() {
        test_parse("1?2:3", "?:(1,2,3)");
        test_parse("1 == 2 ? 3 + 4 : 5", "?:(==(1,2),+(3,4),5)");
    }


    #[test]
    fn parse_commas_terminate() {
        test_parse("1+2,3", "+(1,2)");
    }


    fn do_parse(expression: &str) -> Result<ExpressionNode, String> {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        parse(&mut tokenizer, false)
    }
    
    
    fn test_parse(expression: &str, expected: &str) {
        let expression = do_parse(expression).unwrap();
        let result = format!("{}", expression);
        assert_eq!(result, expected);
    }


    #[test]
    fn parse_errors() {
        test_parse_error("1 2", "Invalid expression: expecting operator but got '2'.");
        test_parse_error("e pi", "Invalid expression: expecting operator but got 'pi'.");
        test_parse_error("foo() bar()", "Invalid expression: expecting operator but got 'bar'.");

        test_parse_error("e(1)", "Wrong number of arguments for e(): expected 0 but got 1.");
        test_parse_error("e(1,2,3)", "Wrong number of arguments for e(): expected 0 but got 3.");
        test_parse_error("sin()", "Wrong number of arguments for sin(): expected 1 but got 0.");
        test_parse_error("sin(1,2)", "Wrong number of arguments for sin(): expected 1 but got 2.");
        test_parse_error("max()", "Wrong number of arguments for max(): expected 2 but got 0.");
        test_parse_error("max(1)", "Wrong number of arguments for max(): expected 2 but got 1.");
        test_parse_error("max(1,2,3)", "Wrong number of arguments for max(): expected 2 but got 3.");

        test_parse_error("1ee2", "Invalid numeric constant '1ee2'.");
        test_parse_error("sin(1ee2)", "Invalid numeric constant '1ee2'.");

        test_parse_error("!+", "Invalid expression: unary ! operator is missing an operand.");
        test_parse_error("++", "Invalid expression: binary + operator is missing an operand.");

        test_parse_error("1()", "Invalid expression: unexpected open parenthesis.");
        test_parse_error("()", "Invalid expression: unexpected close parenthesis.");
        test_parse_error(")", "Invalid expression: too many close parentheses.");
        test_parse_error("x(y+z)/sqrt(10))+2", "Invalid expression: too many close parentheses.");

        test_parse_error("x+", "Invalid expression: unexpected end of input.");
        test_parse_error("sqrt(", "Invalid expression: unexpected end of input.");
        test_parse_error("(1", "Invalid expression: unexpected end of input.");
    }


    fn test_parse_error(expression: &str, expected_error: &str) {
        let error = do_parse(expression).unwrap_err();
        assert_eq!(error, expected_error);
    }


    #[test]
    fn eval_booleans() {
        assert_eq!(unwrap_eval("1<2 || 3<4"), 1.0);
        assert_eq!(unwrap_eval("1>2 || 3<4"), 1.0);
        assert_eq!(unwrap_eval("1<2 || 3>4"), 1.0);
        assert_eq!(unwrap_eval("1>2 || 3>4"), 0.0);
        assert_eq!(unwrap_eval("23 || 42"), 23.0);
        assert_eq!(unwrap_eval("0 || 42"), 42.0);
        assert_eq!(unwrap_eval("23 || 0"), 23.0);

        assert_eq!(unwrap_eval("1<2 && 3<4"), 1.0);
        assert_eq!(unwrap_eval("1>2 && 3<4"), 0.0);
        assert_eq!(unwrap_eval("1<2 && 3>4"), 0.0);
        assert_eq!(unwrap_eval("1>2 && 3>4"), 0.0);
        assert_eq!(unwrap_eval("23 && 42"), 42.0);
        assert_eq!(unwrap_eval("0 && 42"), 0.0);
        assert_eq!(unwrap_eval("23 && 0"), 0.0);

        assert_eq!(unwrap_eval("!0"), 1.0);
        assert_eq!(unwrap_eval("!1"), 0.0);
        assert_eq!(unwrap_eval("!100"), 0.0);
        assert_eq!(unwrap_eval("!-1"), 0.0);

        assert_eq!(unwrap_eval("0 ? 2 : 3"), 3.0);
        assert_eq!(unwrap_eval("1 ? 2 : 3"), 2.0);
    }


    #[test]
    fn eval_bitwise() {
        assert_eq!(unwrap_eval("0x1234 | 0x5678"), 22140.0);
        assert_eq!(unwrap_eval("-10000 | -12345"), -8201.0);

        assert_eq!(unwrap_eval("0x1234 ^^ 0x5678"), 17484.0);
        assert_eq!(unwrap_eval("-10000 ^^ -12345"), 5943.0);

        assert_eq!(unwrap_eval("0x1234 & 0x5678"), 4656.0);
        assert_eq!(unwrap_eval("-10000 & -12345"), -14144.0);

        assert_eq!(unwrap_eval("1 << 0"), 1.0);
        assert_eq!(unwrap_eval("1 << 1"), 2.0);
        assert_eq!(unwrap_eval("1 << 4"), 16.0);
        assert_eq!(unwrap_eval("1 << 30"), 1073741824.0);
        assert_eq!(unwrap_eval("1 << 31"), -2147483648.0);
        assert_eq!(unwrap_eval("1 << 32"), 1.0);
        assert_eq!(unwrap_eval("1 << -1"), -2147483648.0);
        assert_eq!(unwrap_eval("123 << 2"), 492.0);

        assert_eq!(unwrap_eval("0x80000000 >> 0"), 2147483648.0);
        assert_eq!(unwrap_eval("0x80000000 >> 1"), 1073741824.0);
        assert_eq!(unwrap_eval("0x80000000 >> 4"), 134217728.0);
        assert_eq!(unwrap_eval("0x80000000 >> 30"), 2.0);
        assert_eq!(unwrap_eval("0x80000000 >> 31"), 1.0);
        assert_eq!(unwrap_eval("0x80000000 >> 32"), 2147483648.0);
        assert_eq!(unwrap_eval("0x80000000 >> -1"), 1.0);
        assert_eq!(unwrap_eval("1234 >> 2"), 308.0);

        assert_eq!(unwrap_eval("0x80000000 >>> 0"), -2147483648.0);
        assert_eq!(unwrap_eval("0x80000000 >>> 1"), -1073741824.0);
        assert_eq!(unwrap_eval("0x80000000 >>> 4"), -134217728.0);
        assert_eq!(unwrap_eval("0x80000000 >>> 30"), -2.0);
        assert_eq!(unwrap_eval("0x80000000 >>> 31"), -1.0);
        assert_eq!(unwrap_eval("0x80000000 >>> 32"), -2147483648.0);
        assert_eq!(unwrap_eval("0x80000000 >>> -1"), -1.0);
        assert_eq!(unwrap_eval("1234 >>> 2"), 308.0);

        assert_eq!(unwrap_eval("~0"), -1.0);
        assert_eq!(unwrap_eval("~-1"), 0.0);
        assert_eq!(unwrap_eval("~1234"), -1235.0);
    }


    #[test]
    fn eval_comparisons() {
        assert_eq!(unwrap_eval("1 == 2"), 0.0);
        assert_eq!(unwrap_eval("1 == 1"), 1.0);
        assert_eq!(unwrap_eval("2 == 1"), 0.0);

        assert_eq!(unwrap_eval("1 != 2"), 1.0);
        assert_eq!(unwrap_eval("1 != 1"), 0.0);
        assert_eq!(unwrap_eval("2 != 1"), 1.0);

        assert_eq!(unwrap_eval("1 < 2"), 1.0);
        assert_eq!(unwrap_eval("1 < 1"), 0.0);
        assert_eq!(unwrap_eval("2 < 1"), 0.0);

        assert_eq!(unwrap_eval("1 > 2"), 0.0);
        assert_eq!(unwrap_eval("1 > 1"), 0.0);
        assert_eq!(unwrap_eval("2 > 1"), 1.0);

        assert_eq!(unwrap_eval("1 <= 2"), 1.0);
        assert_eq!(unwrap_eval("1 <= 1"), 1.0);
        assert_eq!(unwrap_eval("2 <= 1"), 0.0);

        assert_eq!(unwrap_eval("1 >= 2"), 0.0);
        assert_eq!(unwrap_eval("1 >= 1"), 1.0);
        assert_eq!(unwrap_eval("2 >= 1"), 1.0);
    }


    #[test]
    fn eval_arithmetic() {
        assert_eq!(unwrap_eval("2 + 3"), 5.0);
        assert_eq!(unwrap_eval("2 + -3"), -1.0);

        assert_eq!(unwrap_eval("2 - 3"), -1.0);
        assert_eq!(unwrap_eval("2 - -3"), 5.0);

        assert_eq!(unwrap_eval("2 * 3"), 6.0);
        assert_eq!(unwrap_eval("2 * -3"), -6.0);

        assert_eq!(unwrap_eval("3 / 2"), 1.5);
        assert_eq!(unwrap_eval("3 / -2"), -1.5);
        assert!(unwrap_eval("3 / 0").is_infinite());

        assert_eq!(unwrap_eval("3 % 2"), 1.0);
        assert_eq!(unwrap_eval("4 % 2"), 0.0);
        assert_eq!(unwrap_eval("10 % 3"), 1.0);
        assert_eq!(unwrap_eval("11 % 3"), 2.0);
        assert_eq!(unwrap_eval("27 % 4"), 3.0);
        assert_eq!(unwrap_eval("28 % 5"), 3.0);
        assert_eq!(unwrap_eval("17.5 % 4.25"), 0.5);
        assert_eq!(unwrap_eval("-16.5 % 5.25"), 4.5);
        assert_eq!(unwrap_eval("-16.5 % -5.25"), 4.5);
        assert_eq!(unwrap_eval("16.5 % 5.25"), 0.75);
        assert_eq!(unwrap_eval("16.5 % -5.25"), 0.75);

        assert_eq!(unwrap_eval("2 ^ 3"), 8.0);
        assert_eq!(unwrap_eval("2 ^ 1"), 2.0);
        assert_eq!(unwrap_eval("2 ^ 0"), 1.0);
        assert_eq!(unwrap_eval("2 ^ -2"), 0.25);
        assert_eq!(unwrap_eval("256 ^ 0.25"), 4.0);
    }


    #[test]
    fn eval_math_ops() {
        assert_eq!(unwrap_eval("max(1, 2)"), 2.0);
        assert_eq!(unwrap_eval("max(2, 1)"), 2.0);

        assert_eq!(unwrap_eval("min(1, 2)"), 1.0);
        assert_eq!(unwrap_eval("min(2, 1)"), 1.0);

        assert_eq!(unwrap_eval("sqrt(256)"), 16.0);
        assert_eq!(unwrap_eval("sqrt(100)"), 10.0);
        assert_eq!(unwrap_eval("sqrt(1)"), 1.0);
        assert_eq!(unwrap_eval("sqrt(0)"), 0.0);
        assert!(unwrap_eval("sqrt(-1)").is_nan());

        assert_eq!(unwrap_eval("exp(0)"), 1.0);
        assert_eq!(unwrap_eval("exp(1)"), f64::consts::E);

        assert_eq!(unwrap_eval("ln(e)"), 1.0);
        assert_eq!(unwrap_eval("ln(1)"), 0.0);
        assert!(unwrap_eval("ln(0)").is_infinite());

        assert_eq!(unwrap_eval("log(100)"), 2.0);
        assert_eq!(unwrap_eval("log(10)"), 1.0);
        assert_eq!(unwrap_eval("log(1)"), 0.0);
        assert!(unwrap_eval("log(0)").is_infinite());

        assert_eq!(unwrap_eval("log2(4)"), 2.0);
        assert_eq!(unwrap_eval("log2(2)"), 1.0);
        assert_eq!(unwrap_eval("log2(1)"), 0.0);
        assert!(unwrap_eval("log2(0)").is_infinite());

        assert_eq!(unwrap_eval("abs(-123)"), 123.0);
        assert_eq!(unwrap_eval("abs(0)"), 0.0);
        assert_eq!(unwrap_eval("abs(123)"), 123.0);

        assert_eq!(unwrap_eval("ceil(-1.1)"), -1.0);
        assert_eq!(unwrap_eval("ceil(-1)"), -1.0);
        assert_eq!(unwrap_eval("ceil(-0.9)"), 0.0);
        assert_eq!(unwrap_eval("ceil(-0.5)"), 0.0);
        assert_eq!(unwrap_eval("ceil(-0.1)"), 0.0);
        assert_eq!(unwrap_eval("ceil(0)"), 0.0);
        assert_eq!(unwrap_eval("ceil(0.1)"), 1.0);
        assert_eq!(unwrap_eval("ceil(0.5)"), 1.0);
        assert_eq!(unwrap_eval("ceil(0.9)"), 1.0);
        assert_eq!(unwrap_eval("ceil(1)"), 1.0);
        assert_eq!(unwrap_eval("ceil(1.1)"), 2.0);

        assert_eq!(unwrap_eval("floor(-1.1)"), -2.0);
        assert_eq!(unwrap_eval("floor(-1)"), -1.0);
        assert_eq!(unwrap_eval("floor(-0.9)"), -1.0);
        assert_eq!(unwrap_eval("floor(-0.5)"), -1.0);
        assert_eq!(unwrap_eval("floor(-0.1)"), -1.0);
        assert_eq!(unwrap_eval("floor(0)"), 0.0);
        assert_eq!(unwrap_eval("floor(0.1)"), 0.0);
        assert_eq!(unwrap_eval("floor(0.5)"), 0.0);
        assert_eq!(unwrap_eval("floor(0.9)"), 0.0);
        assert_eq!(unwrap_eval("floor(1)"), 1.0);
        assert_eq!(unwrap_eval("floor(1.1)"), 1.0);

        assert_eq!(unwrap_eval("round(-1.1)"), -1.0);
        assert_eq!(unwrap_eval("round(-1)"), -1.0);
        assert_eq!(unwrap_eval("round(-0.9)"), -1.0);
        assert_eq!(unwrap_eval("round(-0.5)"), -1.0);
        assert_eq!(unwrap_eval("round(-0.1)"), 0.0);
        assert_eq!(unwrap_eval("round(0)"), 0.0);
        assert_eq!(unwrap_eval("round(0.1)"), 0.0);
        assert_eq!(unwrap_eval("round(0.5)"), 1.0);
        assert_eq!(unwrap_eval("round(0.9)"), 1.0);
        assert_eq!(unwrap_eval("round(1)"), 1.0);
        assert_eq!(unwrap_eval("round(1.1)"), 1.0);
    }


    #[test]
    fn eval_trig() {
        let value: f64 = 0.5;
        let value2: f64 = 1.5;
        
        assert_eq!(unwrap_eval("sin(0.5)"), value.sin());
        assert_eq!(unwrap_eval("cos(0.5)"), value.cos());
        assert_eq!(unwrap_eval("tan(0.5)"), value.tan());
        assert_eq!(unwrap_eval("sinh(0.5)"), value.sinh());
        assert_eq!(unwrap_eval("cosh(0.5)"), value.cosh());
        assert_eq!(unwrap_eval("tanh(0.5)"), value.tanh());
        assert_eq!(unwrap_eval("asin(0.5)"), value.asin());
        assert_eq!(unwrap_eval("acos(0.5)"), value.acos());
        assert_eq!(unwrap_eval("atan(0.5)"), value.atan());
        assert_eq!(unwrap_eval("asinh(0.5)"), value.asinh());
        assert_eq!(unwrap_eval("acosh(1.5)"), value2.acosh());
        assert_eq!(unwrap_eval("atanh(0.5)"), value.atanh());
    }


    #[test]
    fn eval_casts() {
        assert_eq!(unwrap_eval("i8(-1)"), -1.0);
        assert_eq!(unwrap_eval("i8(0)"), 0.0);
        assert_eq!(unwrap_eval("i8(1)"), 1.0);
        assert_eq!(unwrap_eval("i8(127)"), 127.0);
        assert_eq!(unwrap_eval("i8(128)"), -128.0);
        assert_eq!(unwrap_eval("i8(129)"), -127.0);
        assert_eq!(unwrap_eval("i8(255)"), -1.0);
        assert_eq!(unwrap_eval("i8(256)"), 0.0);
        assert_eq!(unwrap_eval("i8(257)"), 1.0);
        assert_eq!(unwrap_eval("i8(32767)"), -1.0);
        assert_eq!(unwrap_eval("i8(32768)"), 0.0);
        assert_eq!(unwrap_eval("i8(32769)"), 1.0);
        assert_eq!(unwrap_eval("i8(65535)"), -1.0);
        assert_eq!(unwrap_eval("i8(65536)"), 0.0);
        assert_eq!(unwrap_eval("i8(65537)"), 1.0);
        assert_eq!(unwrap_eval("i8(2147483647)"), -1.0);
        assert_eq!(unwrap_eval("i8(2147483648)"), 0.0);
        assert_eq!(unwrap_eval("i8(2147483649)"), 1.0);

        assert_eq!(unwrap_eval("u8(-1)"), 255.0);
        assert_eq!(unwrap_eval("u8(0)"), 0.0);
        assert_eq!(unwrap_eval("u8(1)"), 1.0);
        assert_eq!(unwrap_eval("u8(127)"), 127.0);
        assert_eq!(unwrap_eval("u8(128)"), 128.0);
        assert_eq!(unwrap_eval("u8(129)"), 129.0);
        assert_eq!(unwrap_eval("u8(255)"), 255.0);
        assert_eq!(unwrap_eval("u8(256)"), 0.0);
        assert_eq!(unwrap_eval("u8(257)"), 1.0);
        assert_eq!(unwrap_eval("u8(32767)"), 255.0);
        assert_eq!(unwrap_eval("u8(32768)"), 0.0);
        assert_eq!(unwrap_eval("u8(32769)"), 1.0);
        assert_eq!(unwrap_eval("u8(65535)"), 255.0);
        assert_eq!(unwrap_eval("u8(65536)"), 0.0);
        assert_eq!(unwrap_eval("u8(65537)"), 1.0);
        assert_eq!(unwrap_eval("u8(2147483647)"), 255.0);
        assert_eq!(unwrap_eval("u8(2147483648)"), 0.0);
        assert_eq!(unwrap_eval("u8(2147483649)"), 1.0);

        assert_eq!(unwrap_eval("i16(-1)"), -1.0);
        assert_eq!(unwrap_eval("i16(0)"), 0.0);
        assert_eq!(unwrap_eval("i16(1)"), 1.0);
        assert_eq!(unwrap_eval("i16(256)"), 256.0);
        assert_eq!(unwrap_eval("i16(32767)"), 32767.0);
        assert_eq!(unwrap_eval("i16(32768)"), -32768.0);
        assert_eq!(unwrap_eval("i16(32769)"), -32767.0);
        assert_eq!(unwrap_eval("i16(65535)"), -1.0);
        assert_eq!(unwrap_eval("i16(65536)"), 0.0);
        assert_eq!(unwrap_eval("i16(65537)"), 1.0);
        assert_eq!(unwrap_eval("i16(2147483647)"), -1.0);
        assert_eq!(unwrap_eval("i16(2147483648)"), 0.0);
        assert_eq!(unwrap_eval("i16(2147483649)"), 1.0);

        assert_eq!(unwrap_eval("u16(-1)"), 65535.0);
        assert_eq!(unwrap_eval("u16(0)"), 0.0);
        assert_eq!(unwrap_eval("u16(1)"), 1.0);
        assert_eq!(unwrap_eval("u16(256)"), 256.0);
        assert_eq!(unwrap_eval("u16(32767)"), 32767.0);
        assert_eq!(unwrap_eval("u16(32768)"), 32768.0);
        assert_eq!(unwrap_eval("u16(32769)"), 32769.0);
        assert_eq!(unwrap_eval("u16(65535)"), 65535.0);
        assert_eq!(unwrap_eval("u16(65536)"), 0.0);
        assert_eq!(unwrap_eval("u16(65537)"), 1.0);
        assert_eq!(unwrap_eval("u16(2147483647)"), 65535.0);
        assert_eq!(unwrap_eval("u16(2147483648)"), 0.0);
        assert_eq!(unwrap_eval("u16(2147483649)"), 1.0);

        assert_eq!(unwrap_eval("i32(-1)"), -1.0);
        assert_eq!(unwrap_eval("i32(0)"), 0.0);
        assert_eq!(unwrap_eval("i32(1)"), 1.0);
        assert_eq!(unwrap_eval("i32(65536)"), 65536.0);
        assert_eq!(unwrap_eval("i32(2147483647)"), 2147483647.0);
        assert_eq!(unwrap_eval("i32(2147483648)"), -2147483648.0);
        assert_eq!(unwrap_eval("i32(2147483649)"), -2147483647.0);
        assert_eq!(unwrap_eval("i32(4294967295)"), -1.0);
        assert_eq!(unwrap_eval("i32(4294967296)"), 0.0);
        assert_eq!(unwrap_eval("i32(4294967297)"), 1.0);

        assert_eq!(unwrap_eval("u32(-1)"), 4294967295.0);
        assert_eq!(unwrap_eval("u32(0)"), 0.0);
        assert_eq!(unwrap_eval("u32(1)"), 1.0);
        assert_eq!(unwrap_eval("u32(65536)"), 65536.0);
        assert_eq!(unwrap_eval("u32(2147483647)"), 2147483647.0);
        assert_eq!(unwrap_eval("u32(2147483648)"), 2147483648.0);
        assert_eq!(unwrap_eval("u32(2147483649)"), 2147483649.0);
        assert_eq!(unwrap_eval("u32(4294967295)"), 4294967295.0);
        assert_eq!(unwrap_eval("u32(4294967296)"), 0.0);
        assert_eq!(unwrap_eval("u32(4294967297)"), 1.0);
    }


    #[test]
    fn eval_special_constants() {
        assert_eq!(unwrap_eval("e"), f64::consts::E);
        assert_eq!(unwrap_eval("pi"), f64::consts::PI);
    }


    fn do_eval(expression: &str, context: &mut Context) -> Result<f64, String> {
        let expression = do_parse(expression).unwrap();
        evaluate(&expression, context)
    }


    fn unwrap_eval(expression: &str) -> f64 {
        do_eval(expression, &mut Context::new()).unwrap()
    }


    #[test]
    fn eval_invalid_operators() {
        test_eval_error("a = b", "Invalid use of = operator.");
        test_eval_error("a ? b", "Invalid use of ? operator.");
        test_eval_error("a : b", "Invalid use of : operator.");
    }


    fn test_eval_error(expression: &str, expected_error: &str) {
        let error = do_eval(expression, &mut Context::new()).unwrap_err();
        assert_eq!(error, expected_error);
    }


    #[test]
    fn deconstruct_function() {
        test_deconstruct("f=1").unwrap();
        test_deconstruct("f(x)=1").unwrap();
        test_deconstruct("f(x,y)=1").unwrap();

        assert!(test_deconstruct("1+1").is_none());
        assert!(test_deconstruct("1=1").is_none());
        assert!(test_deconstruct("1+1=1").is_none());
        assert!(test_deconstruct("f(1)=1").is_none());
        assert!(test_deconstruct("f(x(y))=1").is_none());
    }


    fn test_deconstruct(expression: &str) -> Option<(Function, String)> {
        let mut expression = do_parse(expression).unwrap();
        deconstruct_function_definition(&mut expression)
    }


    #[test]
    fn variables() {
        let mut context = Context::new();
        
        define_function("x = 23", &mut context);
        define_function("y = 42", &mut context);
        
        assert_eq!(do_eval("x", &mut context).unwrap(), 23.0);
        assert_eq!(do_eval("y", &mut context).unwrap(), 42.0);

        assert_eq!(do_eval("z", &mut context).unwrap_err(), "Unknown value z.");
    }


    #[test]
    fn functions() {
        let mut context = Context::new();
        
        define_function("f(x) = x*2", &mut context);
        define_function("g(x, y) = x*y", &mut context);
        define_function("h(x) = f(x) + g(x, 3)", &mut context);
        
        assert_eq!(do_eval("f(10)", &mut context).unwrap(), 20.0);
        assert_eq!(do_eval("f(f(1))", &mut context).unwrap(), 4.0);
        assert_eq!(do_eval("g(2, 3)", &mut context).unwrap(), 6.0);
        assert_eq!(do_eval("h(5)", &mut context).unwrap(), 25.0);

        assert_eq!(do_eval("z", &mut context).unwrap_err(), "Unknown value z.");
    }


    #[test]
    fn function_errors() {
        let mut context = Context::new();
        
        define_function("x = 1", &mut context);
        define_function("f(x) = 1", &mut context);
        define_function("g(x, y) = 1", &mut context);
        
        assert_eq!(do_eval("x(1)", &mut context).unwrap_err(), "Wrong number of arguments for x(): expected 0 but got 1.");
        assert_eq!(do_eval("f", &mut context).unwrap_err(), "Wrong number of arguments for f(): expected 1 but got 0.");
        assert_eq!(do_eval("f(1, 2)", &mut context).unwrap_err(), "Wrong number of arguments for f(): expected 1 but got 2.");
        assert_eq!(do_eval("g()", &mut context).unwrap_err(), "Wrong number of arguments for g(): expected 2 but got 0.");
        assert_eq!(do_eval("g(1)", &mut context).unwrap_err(), "Wrong number of arguments for g(): expected 2 but got 1.");
        assert_eq!(do_eval("g(1, 2, 3)", &mut context).unwrap_err(), "Wrong number of arguments for g(): expected 2 but got 3.");

        define_function("f(x) = x(1)", &mut context);

        assert_eq!(do_eval("f(1)", &mut context).unwrap_err(), "Use of x() as first class function is not supported.");
    }


    #[test]
    fn recursion() {
        let mut context = Context::new();
        
        define_function("factorial(n) = n>1 ? n * factorial(n-1) : 1", &mut context);
        
        assert_eq!(do_eval("factorial(1)", &mut context).unwrap(), 1.0);
        assert_eq!(do_eval("factorial(2)", &mut context).unwrap(), 2.0);
        assert_eq!(do_eval("factorial(3)", &mut context).unwrap(), 6.0);
        assert_eq!(do_eval("factorial(10)", &mut context).unwrap(), 3628800.0);

        assert_eq!(do_eval("factorial(1000)", &mut context).unwrap_err(), "Excessive recursion.");
    }


    fn define_function(expression: &str, context: &mut Context) {
        let (function, function_name) = test_deconstruct(expression).unwrap();
        context.functions.insert(function_name, function);
    }
}
