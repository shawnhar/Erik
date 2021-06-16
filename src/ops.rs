// Ordering of these enum values determines parser behavior.
#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum Precedence {
    None,
    Brace,
    Terminator,
    Assign,
    Ternary,
    LogicalOr,
    LogicalAnd,
    BinaryOr,
    BinaryXor,
    BinaryAnd,
    CompareEq,
    CompareDiff,
    Shift,
    Addition,
    Multiply,
    Unary,
    Power,
}


// Implementation of an operator or built-in function.
#[derive(Debug)]
pub struct Operator {
    pub name: &'static str,
    pub precedence: Precedence,
    pub arity: u32,
    pub is_right_associative: bool,
}


pub type OperatorRef = &'static Operator;


// Allow comparing operators directly against their string names.
impl PartialEq<str> for Operator {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl PartialEq<str> for OperatorRef {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}


// Helper reduces repetititiveness of what follows.
const fn make_op(name: &'static str, precedence: Precedence, arity: u32) -> Operator {
    Operator { name, precedence, arity, is_right_associative: false }
}


pub static OPERATORS: [Operator; 52] = [
    // Special markers.
    make_op("(",     Precedence::Brace,       0),
    make_op(")",     Precedence::Brace,       0),
    make_op("=",     Precedence::Assign,      2),

    // Component parts of the ternary ?: operator.
    Operator { name: "?", precedence: Precedence::Ternary, arity: 2, is_right_associative: true },
    Operator { name: ":", precedence: Precedence::Ternary, arity: 2, is_right_associative: true },

    // Logical operators.
    make_op("||",    Precedence::LogicalOr,   2),
    make_op("&&",    Precedence::LogicalAnd,  2),

    // Binary operators.
    make_op("|",     Precedence::BinaryOr,    2),
    make_op("^^",    Precedence::BinaryXor,   2),
    make_op("&",     Precedence::BinaryAnd,   2),

    // Comparisons
    make_op("==",    Precedence::CompareEq,   2),
    make_op("!=",    Precedence::CompareEq,   2),
    make_op("<",     Precedence::CompareDiff, 2),
    make_op(">",     Precedence::CompareDiff, 2),
    make_op("<=",    Precedence::CompareDiff, 2),
    make_op(">=",    Precedence::CompareDiff, 2),

    // Shifts.
    make_op("<<",    Precedence::Shift,       2),
    make_op(">>",    Precedence::Shift,       2),
    make_op(">>>",   Precedence::Shift,       2),

    // Arithmetic.
    make_op("+",     Precedence::Addition,    2),
    make_op("-",     Precedence::Addition,    2),
    make_op("*",     Precedence::Multiply,    2),
    make_op("/",     Precedence::Multiply,    2),
    make_op("%",     Precedence::Multiply,    2),

    // Negation.
    make_op("!",     Precedence::Unary,       1),
    make_op("~",     Precedence::Unary,       1),

    // Raise to a power.
    make_op("^",     Precedence::Power,       2),

    // Math functions.
    make_op("max",   Precedence::None,        2),
    make_op("min",   Precedence::None,        2),
    make_op("sqrt",  Precedence::None,        1),
    make_op("exp",   Precedence::None,        1),
    make_op("ln",    Precedence::None,        1),
    make_op("log",   Precedence::None,        1),
    make_op("ceil",  Precedence::None,        1),
    make_op("floor", Precedence::None,        1),

    // Trig.
    make_op("sin",   Precedence::None,        1),
    make_op("cos",   Precedence::None,        1),
    make_op("tan",   Precedence::None,        1),
    make_op("sinh",  Precedence::None,        1),
    make_op("cosh",  Precedence::None,        1),
    make_op("tanh",  Precedence::None,        1),
    make_op("asin",  Precedence::None,        1),
    make_op("acos",  Precedence::None,        1),
    make_op("atan",  Precedence::None,        1),

    // Casts.
    make_op("s8",    Precedence::None,        1),
    make_op("u8",    Precedence::None,        1),
    make_op("s16",   Precedence::None,        1),
    make_op("u16",   Precedence::None,        1),
    make_op("s32",   Precedence::None,        1),
    make_op("u32",   Precedence::None,        1),

    // Constants.
    make_op("e",     Precedence::None,        0),
    make_op("pi",    Precedence::None,        0),
];


// Special operators, not accessible by name.
pub static TERMINATOR: Operator = make_op("terminator", Precedence::Terminator, 0);
pub static NEGATE:     Operator = make_op("-",          Precedence::Unary,      1);
// TODO Operator { name: "?:", precedence: Precedence::Ternary },


pub fn find_operator(opname: &str) -> Option<OperatorRef> {
    // Linear search is fine as there aren't that many operators and their names are short.
    OPERATORS.iter().find(|op| op == opname)
}
