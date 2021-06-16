use crate::ops;


#[derive(Debug)]
pub enum ExpressionNode<'a> {
    Constant(f64),
    Operator(&'static ops::Operator, Vec<ExpressionNode<'a>>),
    Function(&'a str, Vec<ExpressionNode<'a>>)
}


#[cfg(test)]
mod tests {
    // use super::*;


    #[test]
    fn foo() {
    }
}
