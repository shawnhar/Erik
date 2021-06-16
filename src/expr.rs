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


// The parser turns a series of tokens into an expression tree.
struct Parser
{
    operator_stack: Vec<ops::OperatorRef>,
    value_stack:    Vec<Option<ExpressionNode>>,
    current_value:  Option<ExpressionNode>
}


impl Parser {
    // Pushes a numeric constant onto the stack.
    fn push_constant(&mut self, value: f64) -> Result<(), String> {
        if self.current_value.is_some() {
            return Err(String::from("Invalid expression aTodo."));
        }

        self.current_value = Some(ExpressionNode::Constant { value });
        
        Ok(())
    }


    // Pushes a symbol reference (variable or function call) onto the stack.
    fn push_symbol(&mut self, symbol: &str, tokenizer: &mut Peekable<Tokenizer>) -> Result<(), String> {
        if self.current_value.is_some() {
            return Err(String::from("Invalid expression bTodo."));
        }

        let args = parse_arguments(tokenizer)?;

        match ops::find_operator(symbol) {
            Some(op) => self.current_value = Some(ExpressionNode::Operator { op, args }),
            None     => self.current_value = Some(ExpressionNode::Function { name: String::from(symbol), args }),
        }

        Ok(())
    }


    fn push_operator(&mut self, mut op: ops::OperatorRef) -> Result<(), String> {
        // Special case to handle unary negation versus binary subtraction.
        if op == "-" && self.current_value.is_none() {
            op = &ops::NEGATE;
        }

        // Reduce the precedence stack
        if op != "(" {
            let precedence = op.precedence as u32;

            while !self.operator_stack.is_empty() {
                // Compare precedence with the operator on top of the stack.
                let stack = self.operator_stack.last().unwrap();

                let mut stack_precedence = stack.precedence as u32;

                if stack.arity != 1 {
                    if op.arity == 1 {
                        break;
                    }

                    if !stack.right_assoc {
                        stack_precedence = stack_precedence + 1;
                    }
                }

                if precedence >= stack_precedence {   // todo comparable
                    break;
                }

                // Pop from the stack.
                let stack = self.operator_stack.pop().unwrap();
                let stack_value = self.value_stack.pop().unwrap();

                match stack.arity {
    
                    1 => {
                        // Unary operator.
                        if self.current_value.is_none() || stack_value.is_some() {
                            return Err(String::from("Invalid expression iTodo."));
                        }

                        self.current_value = Some(ExpressionNode::Operator { op: stack, args: vec![ self.current_value.take().unwrap() ] });
                    },
                                        
                    2 => {
                        // Binary operator.
                        if self.current_value.is_none() || stack_value.is_none() {
                            return Err(String::from("Invalid expression hTodo."));
                        }

                        self.current_value = Some(ExpressionNode::Operator { op: stack, args: vec![ stack_value.unwrap(), self.current_value.take().unwrap() ] });
                        // TODO BinaryOrTernary(stack, stack_value, mCurrent);
                    },
                    
                    _ => {
                        // Match open and close braces.
                        if stack != "(" || stack_value.is_some() {
                            return Err(String::from("Invalid expression jTodo."));
                        }

                        if op == ")" {
                            if self.current_value.is_none() {
                                return Err(String::from("Invalid expression kTodo."));
                            }

                            return Ok(());
                        }
                    }
                }
            }
        }

        if op == ")" {
            // Swallow close braces, but not too many.
            if self.operator_stack.is_empty() {
                return Err(String::from("Invalid expression iTodo."));
            }
        }
        else {
            // Push onto the stack.
            self.operator_stack.push(op);
            self.value_stack.push(self.current_value.take());
        }
        
        Ok(())
    }
    
    
/*

    // decide whether this is a binary or ternary operator
    static ParseTree BinaryOrTernary(Operator op, ParseTree x, ParseTree y)
    {
            if (op == OpTable.QuestionMark) {
                    ParseTreeOperator colon = y as ParseTreeOperator;

                    if ((colon != null) && (colon.Operator == OpTable.Colon))
                            return new ParseTreeOperator(OpTable.Ternary, new ParseTree[] { x, colon.Args[0], colon.Args[1] } );
            }

            return new ParseTreeOperator(op, new ParseTree[] { x, y } );
    }
*/


    // Decide whether we've reached the end of the expression.
    fn done_parsing(&self, tokenizer: &mut Peekable<Tokenizer>, is_nested: bool) -> bool {
        if let Some(Ok(Token::Text(","))) = tokenizer.peek() {
            // Commas always terminate.
            tokenizer.next();
            true
        }
        else if is_nested {
            // Parsing x or y from something like f(x, y(z)).
            if peek_operator(tokenizer, ")") {
                !self.operator_stack.iter().any(|op| { op == "(" })
            }
            else {
                false
            }
        }
        else {
            // Parsing the top level expression.
            tokenizer.peek().is_none()
        }
    }
}


// Checks whether the next token is the specified operator.
fn peek_operator(tokenizer: &mut Peekable<Tokenizer>, opname: &str) -> bool {
    match tokenizer.peek() {
        Some(Ok(Token::Operator(op))) => op == opname,
        _ => false
    }
}


// Parses the arguments of a function call.
fn parse_arguments(tokenizer: &mut Peekable<Tokenizer>) -> Result<Vec<ExpressionNode>, String>
{
    let mut args = vec![];

    if peek_operator(tokenizer, "(") {
        tokenizer.next();

        while !peek_operator(tokenizer, ")") {
            args.push(parse_expression(tokenizer, true)?);
        }

        tokenizer.next();
    }

    Ok(args)
}


// The main expression parser.
pub fn parse_expression(tokenizer: &mut Peekable<Tokenizer>, is_nested: bool) -> Result<ExpressionNode, String>
{
    let mut parser = Parser {
        operator_stack: vec![],
        value_stack:    vec![],
        current_value:  None,
    };

    // Shift/reduce loop.
    while !parser.done_parsing(tokenizer, is_nested) {
        match tokenizer.next() {
            Some(token) => {
                match token? {
                    Token::Number(value) => parser.push_constant(value)?,
                    Token::Text(value)   => parser.push_symbol(value, tokenizer)?,
                    Token::Operator(op)  => parser.push_operator(op)?,
                }
            },
            None => return Err(String::from("Invalid expression xTODO."))
        }
    }

    // Flush the stack.
    if parser.current_value.is_none() {
        return Err(String::from("Invalid expression yTODO."));
    }

    parser.push_operator(&ops::TERMINATOR)?;

    // We should now have just one root node left on the value stack.
    if parser.operator_stack.len() != 1 || parser.value_stack.len() != 1 {
        return Err(String::from("Invalid expression zTODO."));
    }

    Ok(parser.value_stack.pop().unwrap().unwrap())
}


#[cfg(test)]
mod tests {
    // use super::*;


    #[test]
    fn foo() {
    }
}
