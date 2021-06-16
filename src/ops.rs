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
    pub right_assoc: bool,
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


pub static OPERATORS: [Operator; 52] = [
    // Special marker operators.
    Operator { name: "(",     precedence: Precedence::Brace,  arity: 0, right_assoc: false },
    Operator { name: ")",     precedence: Precedence::Brace,  arity: 0, right_assoc: false },
    Operator { name: "=",     precedence: Precedence::Assign, arity: 2, right_assoc: false },

    // Component parts of the ternary ?: operator.
    Operator { name: "?",     precedence: Precedence::Ternary, arity: 2, right_assoc: true },
    Operator { name: ":",     precedence: Precedence::Ternary, arity: 2, right_assoc: true },

    // Lazily evaluated logical operators.
    Operator { name: "||",    precedence: Precedence::LogicalOr,  arity: 2, right_assoc: false },
    Operator { name: "&&",    precedence: Precedence::LogicalAnd, arity: 2, right_assoc: false },

    // Binary operators.
    Operator { name: "|",     precedence: Precedence::BinaryOr,  arity: 2, right_assoc: false },
    Operator { name: "^^",    precedence: Precedence::BinaryXor, arity: 2, right_assoc: false },
    Operator { name: "&",     precedence: Precedence::BinaryAnd, arity: 2, right_assoc: false },

    // Comparisons
    Operator { name: "==",    precedence: Precedence::CompareEq,   arity: 2, right_assoc: false },
    Operator { name: "!=",    precedence: Precedence::CompareEq,   arity: 2, right_assoc: false },
    Operator { name: "<",     precedence: Precedence::CompareDiff, arity: 2, right_assoc: false },
    Operator { name: ">",     precedence: Precedence::CompareDiff, arity: 2, right_assoc: false },
    Operator { name: "<=",    precedence: Precedence::CompareDiff, arity: 2, right_assoc: false },
    Operator { name: ">=",    precedence: Precedence::CompareDiff, arity: 2, right_assoc: false },

    // Shifts.
    Operator { name: "<<",    precedence: Precedence::Shift, arity: 2, right_assoc: false },
    Operator { name: ">>",    precedence: Precedence::Shift, arity: 2, right_assoc: false },
    Operator { name: ">>>",   precedence: Precedence::Shift, arity: 2, right_assoc: false },

    // Arithmetic.
    Operator { name: "+",     precedence: Precedence::Addition, arity: 2, right_assoc: false },
    Operator { name: "-",     precedence: Precedence::Addition, arity: 2, right_assoc: false },
    Operator { name: "*",     precedence: Precedence::Multiply, arity: 2, right_assoc: false },
    Operator { name: "/",     precedence: Precedence::Multiply, arity: 2, right_assoc: false },
    Operator { name: "%",     precedence: Precedence::Multiply, arity: 2, right_assoc: false },

    // Negation.
    Operator { name: "!",     precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "~",     precedence: Precedence::Unary, arity: 1, right_assoc: false },

    // Raise to a power.
    Operator { name: "^",     precedence: Precedence::Power, arity: 2, right_assoc: false },

    // Math functions.
    Operator { name: "max",   precedence: Precedence::None,  arity: 2, right_assoc: false },
    Operator { name: "min",   precedence: Precedence::None,  arity: 2, right_assoc: false },
    Operator { name: "sqrt",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "exp",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "ln",    precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "log",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "ceil",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "floor", precedence: Precedence::Unary, arity: 1, right_assoc: false },

    // Trig.
    Operator { name: "sin",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "cos",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "tan",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "sinh",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "cosh",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "tanh",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "asin",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "acos",  precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "atan",  precedence: Precedence::Unary, arity: 1, right_assoc: false },

    // Casts.
    Operator { name: "s8",    precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "u8",    precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "s16",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "u16",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "s32",   precedence: Precedence::Unary, arity: 1, right_assoc: false },
    Operator { name: "u32",   precedence: Precedence::Unary, arity: 1, right_assoc: false },

    // Constants.
    Operator { name: "e",     precedence: Precedence::None, arity: 0, right_assoc: false },
    Operator { name: "pi",    precedence: Precedence::None, arity: 0, right_assoc: false },
];


// Special operators, not accessible by name.
pub static TERMINATOR: Operator = Operator { name: "terminator", precedence: Precedence::Terminator, arity: 0, right_assoc: false };
pub static NEGATE:     Operator = Operator { name: "-",          precedence: Precedence::Unary,      arity: 1, right_assoc: false };
// TODO Operator { name: "?:", precedence: Precedence::Ternary },


pub fn find_operator(opname: &str) -> Option<OperatorRef> {
    // Linear search is fine as there aren't that many operators and their names are short.
    OPERATORS.iter().find(|op| op == opname)
}
