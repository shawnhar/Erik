#[derive(Debug)]
pub enum Precedence {
    // None,
    // Brace,
    // Terminator,
    // Assign,
    // Ternary,
    // LogicalOr,
    // LogicalXor,
    // LogicalAnd,
    // BinaryOr,
    // BinaryXor,
    // BinaryAnd,
    CompareEq,
    CompareDiff,
    // Shift,
    // Addition,
    // Multiply,
    // Unary,
    // Power,
}


#[derive(Debug)]
pub struct Operator {
    pub name: &'static str,
    pub precedence: Precedence
}


pub static OPERATORS: [Operator; 6] = [
    // Comparisons.
    Operator { name: "==", precedence: Precedence::CompareEq   },
    Operator { name: "!=", precedence: Precedence::CompareEq   },
    Operator { name: "<",  precedence: Precedence::CompareDiff },
    Operator { name: ">",  precedence: Precedence::CompareDiff },
    Operator { name: "<=", precedence: Precedence::CompareDiff },
    Operator { name: ">=", precedence: Precedence::CompareDiff },
];
