use std::f64;
use std::fmt;
use std::iter::Peekable;
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
            // Closing parenthisis terminates only if there are no open parens on the stack.
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


// The main expression parser.
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


// Recursive expression evaluator.
pub fn evaluate(expression: &ExpressionNode) -> Result<f64, String> {
    match expression {
        ExpressionNode::Constant{ value    } => Ok(*value),
        ExpressionNode::Operator{ op, args } => evaluate_operator(op, args),
        _ => Err(String::from("Functions aren't implemented yet."))
    }
}


fn evaluate_operator(op: OperatorRef, args: &Vec<ExpressionNode>) -> Result<f64, String> {
    match op.function {
        OpFunction::Nullary(function) => Ok(function()),
        OpFunction::Unary  (function) => Ok(function(evaluate(&args[0])?)),
        OpFunction::Binary (function) => Ok(function(evaluate(&args[0])?, evaluate(&args[1])?)),

        OpFunction::Lazy(function) => {
            // Used by the ||, &&, and ?: operators. Applying a function to the first
            // argument indicates which of the arguments to return. Unused arguments
            // are never evaluated. This lazy evaluation enables recursive functions.
            let arg0 = evaluate(&args[0])?;
            let which_arg = function(arg0);
            
            if which_arg == 0 {
                Ok(arg0)
            }
            else {
                evaluate(&args[which_arg])
            }
        },

        OpFunction::Invalid => Err(format!("Invalid use of {} operator.", op.name))
    }
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


    fn test_parse(expression: &str, expected: &str) {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        let expression = parse(&mut tokenizer, false).unwrap();
        let result = format!("{}", expression);
        assert_eq!(result, expected);
    }


    fn test_parse_error(expression: &str, expected_error: &str) {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        let error = parse(&mut tokenizer, false).unwrap_err();
        assert_eq!(error, expected_error);
    }


    #[test]
    fn eval_booleans() {
        assert_eq!(test_eval("1<2 || 3<4"), 1.0);
        assert_eq!(test_eval("1>2 || 3<4"), 1.0);
        assert_eq!(test_eval("1<2 || 3>4"), 1.0);
        assert_eq!(test_eval("1>2 || 3>4"), 0.0);
        assert_eq!(test_eval("23 || 42"), 23.0);
        assert_eq!(test_eval("0 || 42"), 42.0);
        assert_eq!(test_eval("23 || 0"), 23.0);

        assert_eq!(test_eval("1<2 && 3<4"), 1.0);
        assert_eq!(test_eval("1>2 && 3<4"), 0.0);
        assert_eq!(test_eval("1<2 && 3>4"), 0.0);
        assert_eq!(test_eval("1>2 && 3>4"), 0.0);
        assert_eq!(test_eval("23 && 42"), 42.0);
        assert_eq!(test_eval("0 && 42"), 0.0);
        assert_eq!(test_eval("23 && 0"), 0.0);

        assert_eq!(test_eval("!0"), 1.0);
        assert_eq!(test_eval("!1"), 0.0);
        assert_eq!(test_eval("!100"), 0.0);
        assert_eq!(test_eval("!-1"), 0.0);

        assert_eq!(test_eval("0 ? 2 : 3"), 3.0);
        assert_eq!(test_eval("1 ? 2 : 3"), 2.0);
    }


    #[test]
    fn eval_bitwise() {
        assert_eq!(test_eval("0x1234 | 0x5678"), 22140.0);
        assert_eq!(test_eval("-10000 | -12345"), -8201.0);

        assert_eq!(test_eval("0x1234 ^^ 0x5678"), 17484.0);
        assert_eq!(test_eval("-10000 ^^ -12345"), 5943.0);

        assert_eq!(test_eval("0x1234 & 0x5678"), 4656.0);
        assert_eq!(test_eval("-10000 & -12345"), -14144.0);

        assert_eq!(test_eval("1 << 0"), 1.0);
        assert_eq!(test_eval("1 << 1"), 2.0);
        assert_eq!(test_eval("1 << 4"), 16.0);
        assert_eq!(test_eval("1 << 30"), 1073741824.0);
        assert_eq!(test_eval("1 << 31"), -2147483648.0);
        assert_eq!(test_eval("1 << 32"), 1.0);
        assert_eq!(test_eval("1 << -1"), -2147483648.0);
        assert_eq!(test_eval("123 << 2"), 492.0);

        assert_eq!(test_eval("0x80000000 >> 0"), 2147483648.0);
        assert_eq!(test_eval("0x80000000 >> 1"), 1073741824.0);
        assert_eq!(test_eval("0x80000000 >> 4"), 134217728.0);
        assert_eq!(test_eval("0x80000000 >> 30"), 2.0);
        assert_eq!(test_eval("0x80000000 >> 31"), 1.0);
        assert_eq!(test_eval("0x80000000 >> 32"), 2147483648.0);
        assert_eq!(test_eval("0x80000000 >> -1"), 1.0);
        assert_eq!(test_eval("1234 >> 2"), 308.0);

        assert_eq!(test_eval("0x80000000 >>> 0"), -2147483648.0);
        assert_eq!(test_eval("0x80000000 >>> 1"), -1073741824.0);
        assert_eq!(test_eval("0x80000000 >>> 4"), -134217728.0);
        assert_eq!(test_eval("0x80000000 >>> 30"), -2.0);
        assert_eq!(test_eval("0x80000000 >>> 31"), -1.0);
        assert_eq!(test_eval("0x80000000 >>> 32"), -2147483648.0);
        assert_eq!(test_eval("0x80000000 >>> -1"), -1.0);
        assert_eq!(test_eval("1234 >>> 2"), 308.0);

        assert_eq!(test_eval("~0"), -1.0);
        assert_eq!(test_eval("~-1"), 0.0);
        assert_eq!(test_eval("~1234"), -1235.0);
    }


    #[test]
    fn eval_comparisons() {
        assert_eq!(test_eval("1 == 2"), 0.0);
        assert_eq!(test_eval("1 == 1"), 1.0);
        assert_eq!(test_eval("2 == 1"), 0.0);

        assert_eq!(test_eval("1 != 2"), 1.0);
        assert_eq!(test_eval("1 != 1"), 0.0);
        assert_eq!(test_eval("2 != 1"), 1.0);

        assert_eq!(test_eval("1 < 2"), 1.0);
        assert_eq!(test_eval("1 < 1"), 0.0);
        assert_eq!(test_eval("2 < 1"), 0.0);

        assert_eq!(test_eval("1 > 2"), 0.0);
        assert_eq!(test_eval("1 > 1"), 0.0);
        assert_eq!(test_eval("2 > 1"), 1.0);

        assert_eq!(test_eval("1 <= 2"), 1.0);
        assert_eq!(test_eval("1 <= 1"), 1.0);
        assert_eq!(test_eval("2 <= 1"), 0.0);

        assert_eq!(test_eval("1 >= 2"), 0.0);
        assert_eq!(test_eval("1 >= 1"), 1.0);
        assert_eq!(test_eval("2 >= 1"), 1.0);
    }


    #[test]
    fn eval_arithmetic() {
        assert_eq!(test_eval("2 + 3"), 5.0);
        assert_eq!(test_eval("2 + -3"), -1.0);

        assert_eq!(test_eval("2 - 3"), -1.0);
        assert_eq!(test_eval("2 - -3"), 5.0);

        assert_eq!(test_eval("2 * 3"), 6.0);
        assert_eq!(test_eval("2 * -3"), -6.0);

        assert_eq!(test_eval("3 / 2"), 1.5);
        assert_eq!(test_eval("3 / -2"), -1.5);
        assert!(test_eval("3 / 0").is_infinite());

        assert_eq!(test_eval("3 % 2"), 1.0);
        assert_eq!(test_eval("4 % 2"), 0.0);
        assert_eq!(test_eval("10 % 3"), 1.0);
        assert_eq!(test_eval("11 % 3"), 2.0);
        assert_eq!(test_eval("27 % 4"), 3.0);
        assert_eq!(test_eval("28 % 5"), 3.0);
        assert_eq!(test_eval("17.5 % 4.25"), 0.5);
        assert_eq!(test_eval("-16.5 % 5.25"), 4.5);
        assert_eq!(test_eval("-16.5 % -5.25"), 4.5);
        assert_eq!(test_eval("16.5 % 5.25"), 0.75);
        assert_eq!(test_eval("16.5 % -5.25"), 0.75);

        assert_eq!(test_eval("2 ^ 3"), 8.0);
        assert_eq!(test_eval("2 ^ 1"), 2.0);
        assert_eq!(test_eval("2 ^ 0"), 1.0);
        assert_eq!(test_eval("2 ^ -2"), 0.25);
        assert_eq!(test_eval("256 ^ 0.25"), 4.0);
    }


    #[test]
    fn eval_math_functions() {
        assert_eq!(test_eval("max(1, 2)"), 2.0);
        assert_eq!(test_eval("max(2, 1)"), 2.0);

        assert_eq!(test_eval("min(1, 2)"), 1.0);
        assert_eq!(test_eval("min(2, 1)"), 1.0);

        assert_eq!(test_eval("sqrt(256)"), 16.0);
        assert_eq!(test_eval("sqrt(100)"), 10.0);
        assert_eq!(test_eval("sqrt(1)"), 1.0);
        assert_eq!(test_eval("sqrt(0)"), 0.0);
        assert!(test_eval("sqrt(-1)").is_nan());

        assert_eq!(test_eval("exp(0)"), 1.0);
        assert_eq!(test_eval("exp(1)"), f64::consts::E);

        assert_eq!(test_eval("ln(e)"), 1.0);
        assert_eq!(test_eval("ln(1)"), 0.0);
        assert!(test_eval("ln(0)").is_infinite());

        assert_eq!(test_eval("log(100)"), 2.0);
        assert_eq!(test_eval("log(10)"), 1.0);
        assert_eq!(test_eval("log(1)"), 0.0);
        assert!(test_eval("log(0)").is_infinite());

        assert_eq!(test_eval("log2(4)"), 2.0);
        assert_eq!(test_eval("log2(2)"), 1.0);
        assert_eq!(test_eval("log2(1)"), 0.0);
        assert!(test_eval("log2(0)").is_infinite());

        assert_eq!(test_eval("abs(-123)"), 123.0);
        assert_eq!(test_eval("abs(0)"), 0.0);
        assert_eq!(test_eval("abs(123)"), 123.0);

        assert_eq!(test_eval("ceil(-1.1)"), -1.0);
        assert_eq!(test_eval("ceil(-1)"), -1.0);
        assert_eq!(test_eval("ceil(-0.9)"), 0.0);
        assert_eq!(test_eval("ceil(-0.5)"), 0.0);
        assert_eq!(test_eval("ceil(-0.1)"), 0.0);
        assert_eq!(test_eval("ceil(0)"), 0.0);
        assert_eq!(test_eval("ceil(0.1)"), 1.0);
        assert_eq!(test_eval("ceil(0.5)"), 1.0);
        assert_eq!(test_eval("ceil(0.9)"), 1.0);
        assert_eq!(test_eval("ceil(1)"), 1.0);
        assert_eq!(test_eval("ceil(1.1)"), 2.0);

        assert_eq!(test_eval("floor(-1.1)"), -2.0);
        assert_eq!(test_eval("floor(-1)"), -1.0);
        assert_eq!(test_eval("floor(-0.9)"), -1.0);
        assert_eq!(test_eval("floor(-0.5)"), -1.0);
        assert_eq!(test_eval("floor(-0.1)"), -1.0);
        assert_eq!(test_eval("floor(0)"), 0.0);
        assert_eq!(test_eval("floor(0.1)"), 0.0);
        assert_eq!(test_eval("floor(0.5)"), 0.0);
        assert_eq!(test_eval("floor(0.9)"), 0.0);
        assert_eq!(test_eval("floor(1)"), 1.0);
        assert_eq!(test_eval("floor(1.1)"), 1.0);

        assert_eq!(test_eval("round(-1.1)"), -1.0);
        assert_eq!(test_eval("round(-1)"), -1.0);
        assert_eq!(test_eval("round(-0.9)"), -1.0);
        assert_eq!(test_eval("round(-0.5)"), -1.0);
        assert_eq!(test_eval("round(-0.1)"), 0.0);
        assert_eq!(test_eval("round(0)"), 0.0);
        assert_eq!(test_eval("round(0.1)"), 0.0);
        assert_eq!(test_eval("round(0.5)"), 1.0);
        assert_eq!(test_eval("round(0.9)"), 1.0);
        assert_eq!(test_eval("round(1)"), 1.0);
        assert_eq!(test_eval("round(1.1)"), 1.0);
    }


    #[test]
    fn eval_trig() {
        let value: f64 = 0.5;
        let value2: f64 = 1.5;
        
        assert_eq!(test_eval("sin(0.5)"), value.sin());
        assert_eq!(test_eval("cos(0.5)"), value.cos());
        assert_eq!(test_eval("tan(0.5)"), value.tan());
        assert_eq!(test_eval("sinh(0.5)"), value.sinh());
        assert_eq!(test_eval("cosh(0.5)"), value.cosh());
        assert_eq!(test_eval("tanh(0.5)"), value.tanh());
        assert_eq!(test_eval("asin(0.5)"), value.asin());
        assert_eq!(test_eval("acos(0.5)"), value.acos());
        assert_eq!(test_eval("atan(0.5)"), value.atan());
        assert_eq!(test_eval("asinh(0.5)"), value.asinh());
        assert_eq!(test_eval("acosh(1.5)"), value2.acosh());
        assert_eq!(test_eval("atanh(0.5)"), value.atanh());
    }


    #[test]
    fn eval_casts() {
        assert_eq!(test_eval("i8(-1)"), -1.0);
        assert_eq!(test_eval("i8(0)"), 0.0);
        assert_eq!(test_eval("i8(1)"), 1.0);
        assert_eq!(test_eval("i8(127)"), 127.0);
        assert_eq!(test_eval("i8(128)"), -128.0);
        assert_eq!(test_eval("i8(129)"), -127.0);
        assert_eq!(test_eval("i8(255)"), -1.0);
        assert_eq!(test_eval("i8(256)"), 0.0);
        assert_eq!(test_eval("i8(257)"), 1.0);
        assert_eq!(test_eval("i8(32767)"), -1.0);
        assert_eq!(test_eval("i8(32768)"), 0.0);
        assert_eq!(test_eval("i8(32769)"), 1.0);
        assert_eq!(test_eval("i8(65535)"), -1.0);
        assert_eq!(test_eval("i8(65536)"), 0.0);
        assert_eq!(test_eval("i8(65537)"), 1.0);
        assert_eq!(test_eval("i8(2147483647)"), -1.0);
        assert_eq!(test_eval("i8(2147483648)"), 0.0);
        assert_eq!(test_eval("i8(2147483649)"), 1.0);

        assert_eq!(test_eval("u8(-1)"), 255.0);
        assert_eq!(test_eval("u8(0)"), 0.0);
        assert_eq!(test_eval("u8(1)"), 1.0);
        assert_eq!(test_eval("u8(127)"), 127.0);
        assert_eq!(test_eval("u8(128)"), 128.0);
        assert_eq!(test_eval("u8(129)"), 129.0);
        assert_eq!(test_eval("u8(255)"), 255.0);
        assert_eq!(test_eval("u8(256)"), 0.0);
        assert_eq!(test_eval("u8(257)"), 1.0);
        assert_eq!(test_eval("u8(32767)"), 255.0);
        assert_eq!(test_eval("u8(32768)"), 0.0);
        assert_eq!(test_eval("u8(32769)"), 1.0);
        assert_eq!(test_eval("u8(65535)"), 255.0);
        assert_eq!(test_eval("u8(65536)"), 0.0);
        assert_eq!(test_eval("u8(65537)"), 1.0);
        assert_eq!(test_eval("u8(2147483647)"), 255.0);
        assert_eq!(test_eval("u8(2147483648)"), 0.0);
        assert_eq!(test_eval("u8(2147483649)"), 1.0);

        assert_eq!(test_eval("i16(-1)"), -1.0);
        assert_eq!(test_eval("i16(0)"), 0.0);
        assert_eq!(test_eval("i16(1)"), 1.0);
        assert_eq!(test_eval("i16(256)"), 256.0);
        assert_eq!(test_eval("i16(32767)"), 32767.0);
        assert_eq!(test_eval("i16(32768)"), -32768.0);
        assert_eq!(test_eval("i16(32769)"), -32767.0);
        assert_eq!(test_eval("i16(65535)"), -1.0);
        assert_eq!(test_eval("i16(65536)"), 0.0);
        assert_eq!(test_eval("i16(65537)"), 1.0);
        assert_eq!(test_eval("i16(2147483647)"), -1.0);
        assert_eq!(test_eval("i16(2147483648)"), 0.0);
        assert_eq!(test_eval("i16(2147483649)"), 1.0);

        assert_eq!(test_eval("u16(-1)"), 65535.0);
        assert_eq!(test_eval("u16(0)"), 0.0);
        assert_eq!(test_eval("u16(1)"), 1.0);
        assert_eq!(test_eval("u16(256)"), 256.0);
        assert_eq!(test_eval("u16(32767)"), 32767.0);
        assert_eq!(test_eval("u16(32768)"), 32768.0);
        assert_eq!(test_eval("u16(32769)"), 32769.0);
        assert_eq!(test_eval("u16(65535)"), 65535.0);
        assert_eq!(test_eval("u16(65536)"), 0.0);
        assert_eq!(test_eval("u16(65537)"), 1.0);
        assert_eq!(test_eval("u16(2147483647)"), 65535.0);
        assert_eq!(test_eval("u16(2147483648)"), 0.0);
        assert_eq!(test_eval("u16(2147483649)"), 1.0);

        assert_eq!(test_eval("i32(-1)"), -1.0);
        assert_eq!(test_eval("i32(0)"), 0.0);
        assert_eq!(test_eval("i32(1)"), 1.0);
        assert_eq!(test_eval("i32(65536)"), 65536.0);
        assert_eq!(test_eval("i32(2147483647)"), 2147483647.0);
        assert_eq!(test_eval("i32(2147483648)"), -2147483648.0);
        assert_eq!(test_eval("i32(2147483649)"), -2147483647.0);
        assert_eq!(test_eval("i32(4294967295)"), -1.0);
        assert_eq!(test_eval("i32(4294967296)"), 0.0);
        assert_eq!(test_eval("i32(4294967297)"), 1.0);

        assert_eq!(test_eval("u32(-1)"), 4294967295.0);
        assert_eq!(test_eval("u32(0)"), 0.0);
        assert_eq!(test_eval("u32(1)"), 1.0);
        assert_eq!(test_eval("u32(65536)"), 65536.0);
        assert_eq!(test_eval("u32(2147483647)"), 2147483647.0);
        assert_eq!(test_eval("u32(2147483648)"), 2147483648.0);
        assert_eq!(test_eval("u32(2147483649)"), 2147483649.0);
        assert_eq!(test_eval("u32(4294967295)"), 4294967295.0);
        assert_eq!(test_eval("u32(4294967296)"), 0.0);
        assert_eq!(test_eval("u32(4294967297)"), 1.0);
    }


    #[test]
    fn eval_special_constants() {
        assert_eq!(test_eval("e"), f64::consts::E);
        assert_eq!(test_eval("pi"), f64::consts::PI);
    }


    #[test]
    fn eval_invalid_operators() {
        test_eval_error("a = b", "Invalid use of = operator.");
        test_eval_error("a ? b", "Invalid use of ? operator.");
        test_eval_error("a : b", "Invalid use of : operator.");
    }


    fn test_eval(expression: &str) -> f64 {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        let expression = parse(&mut tokenizer, false).unwrap();
        evaluate(&expression).unwrap()
    }


    fn test_eval_error(expression: &str, expected_error: &str) {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        let expression = parse(&mut tokenizer, false).unwrap();
        let error = evaluate(&expression).unwrap_err();
        assert_eq!(error, expected_error);
    }
}
