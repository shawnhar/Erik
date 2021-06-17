use std::fmt;
use std::iter::Peekable;
use crate::ops;
use crate::tokens::{Token, Tokenizer};


// Expressions are represented as a tree of nodes.
#[derive(Debug)]
pub enum ExpressionNode {
    Constant { value: f64 },
    Operator { op: ops::OperatorRef, args: Vec<ExpressionNode> },
    Function { name: String,         args: Vec<ExpressionNode> },
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
    stack: Vec<(ops::OperatorRef, Option<ExpressionNode>)>,
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
    fn push_operator(&mut self, mut op: ops::OperatorRef) -> Result<(), String> {
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
    fn binary_or_ternary(op: ops::OperatorRef, x: ExpressionNode, mut y: ExpressionNode) -> ExpressionNode {
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
                args.push(parse_expression(tokenizer, true)?);
            }

            tokenizer.next();
        }

        Ok(args)
    }


    // Decide whether we've reached the end of the expression.
    fn done_parsing(&self, tokenizer: &mut Peekable<Tokenizer>, is_nested: bool) -> bool {
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
pub fn parse_expression(tokenizer: &mut Peekable<Tokenizer>, is_nested: bool) -> Result<ExpressionNode, String>
{
    let mut parser = Parser {
        current: None,
        stack: vec![],
    };

    while !parser.done_parsing(tokenizer, is_nested) {
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
        test_error("1 2", "Invalid expression: expecting operator but got '2'.");
        test_error("e pi", "Invalid expression: expecting operator but got 'pi'.");
        test_error("foo() bar()", "Invalid expression: expecting operator but got 'bar'.");

        test_error("e(1)", "Wrong number of arguments for e(): expected 0 but got 1.");
        test_error("e(1,2,3)", "Wrong number of arguments for e(): expected 0 but got 3.");
        test_error("sin()", "Wrong number of arguments for sin(): expected 1 but got 0.");
        test_error("sin(1,2)", "Wrong number of arguments for sin(): expected 1 but got 2.");
        test_error("max()", "Wrong number of arguments for max(): expected 2 but got 0.");
        test_error("max(1)", "Wrong number of arguments for max(): expected 2 but got 1.");
        test_error("max(1,2,3)", "Wrong number of arguments for max(): expected 2 but got 3.");

        test_error("1ee2", "Invalid numeric constant '1ee2'.");
        test_error("sin(1ee2)", "Invalid numeric constant '1ee2'.");

        test_error("!+", "Invalid expression: unary ! operator is missing an operand.");
        test_error("++", "Invalid expression: binary + operator is missing an operand.");

        test_error("1()", "Invalid expression: unexpected open parenthesis.");
        test_error("()", "Invalid expression: unexpected close parenthesis.");
        test_error(")", "Invalid expression: too many close parentheses.");
        test_error("x(y+z)/sqrt(10))+2", "Invalid expression: too many close parentheses.");

        test_error("x+", "Invalid expression: unexpected end of input.");
        test_error("sqrt(", "Invalid expression: unexpected end of input.");
        test_error("(1", "Invalid expression: unexpected end of input.");
    }


    fn test_parse(expression: &str, expected: &str) {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        let expression = parse_expression(&mut tokenizer, false).unwrap();
        let result = format!("{}", expression);
        assert_eq!(result, expected);
    }


    fn test_error(expression: &str, expected_error: &str) {
        let mut tokenizer = Tokenizer::new(expression).peekable();
        let error = parse_expression(&mut tokenizer, false).unwrap_err();
        assert_eq!(error, expected_error);
    }
}
