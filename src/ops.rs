#[derive(Debug)]
pub enum Precedence {
    None,
    Brace,
    // TODO Terminator,
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


#[derive(Debug)]
pub struct Operator {
    pub name: &'static str,
    pub precedence: Precedence
}


pub static OPERATORS: [Operator; 56] = [
    // Special marker operators.
    Operator { name: "(",     precedence: Precedence::Brace  },
    Operator { name: ")",     precedence: Precedence::Brace  },
    Operator { name: "=",     precedence: Precedence::Assign },

    // Component parts of the ternary ?: operator.
    Operator { name: "?",     precedence: Precedence::Ternary },
    Operator { name: ":",     precedence: Precedence::Ternary },

    // Lazily evaluated logical operators.
    // TODO Operator { name: "", precedence: Precedence::Ternary },
    Operator { name: "||",    precedence: Precedence::LogicalOr  },
    Operator { name: "&&",    precedence: Precedence::LogicalAnd },

    // Binary operators.
    Operator { name: "|",     precedence: Precedence::BinaryOr  },
    Operator { name: "^^",    precedence: Precedence::BinaryXor },
    Operator { name: "&",     precedence: Precedence::BinaryAnd },

    // Comparisons
    Operator { name: "==",    precedence: Precedence::CompareEq   },
    Operator { name: "!=",    precedence: Precedence::CompareEq   },
    Operator { name: "<",     precedence: Precedence::CompareDiff },
    Operator { name: ">",     precedence: Precedence::CompareDiff },
    Operator { name: "<=",    precedence: Precedence::CompareDiff },
    Operator { name: ">=",    precedence: Precedence::CompareDiff },

    // Shifts.
    Operator { name: "<<",    precedence: Precedence::Shift },
    Operator { name: ">>",    precedence: Precedence::Shift },
    Operator { name: ">>>",   precedence: Precedence::Shift },

    // Arithmetic.
    Operator { name: "+",     precedence: Precedence::Addition },
    Operator { name: "-",     precedence: Precedence::Addition },
    Operator { name: "*",     precedence: Precedence::Multiply },
    Operator { name: "/",     precedence: Precedence::Multiply },
    Operator { name: "%",     precedence: Precedence::Multiply },

    // Negation.
    Operator { name: "!",     precedence: Precedence::Unary },
    Operator { name: "~",     precedence: Precedence::Unary },
    Operator { name: "neg",   precedence: Precedence::Unary },

    // Raise to a power.
    Operator { name: "^",     precedence: Precedence::Power },

    // Math functions.
    Operator { name: "max",   precedence: Precedence::None },
    Operator { name: "min",   precedence: Precedence::None },
    Operator { name: "sqrt",  precedence: Precedence::None },
    Operator { name: "exp",   precedence: Precedence::None },
    Operator { name: "ln",    precedence: Precedence::None },
    Operator { name: "log",   precedence: Precedence::None },
    Operator { name: "ceil",  precedence: Precedence::None },
    Operator { name: "floor", precedence: Precedence::None },

    // Trig.
    Operator { name: "sin",   precedence: Precedence::None },
    Operator { name: "cos",   precedence: Precedence::None },
    Operator { name: "tan",   precedence: Precedence::None },
    Operator { name: "sinh",  precedence: Precedence::None },
    Operator { name: "cosh",  precedence: Precedence::None },
    Operator { name: "tanh",  precedence: Precedence::None },
    Operator { name: "asin",  precedence: Precedence::None },
    Operator { name: "acos",  precedence: Precedence::None },
    Operator { name: "atan",  precedence: Precedence::None },

    // Casts.
    Operator { name: "s8",    precedence: Precedence::None },
    Operator { name: "u8",    precedence: Precedence::None },
    Operator { name: "s16",   precedence: Precedence::None },
    Operator { name: "u16",   precedence: Precedence::None },
    Operator { name: "s32",   precedence: Precedence::None },
    Operator { name: "u32",   precedence: Precedence::None },
    Operator { name: "s64",   precedence: Precedence::None },
    Operator { name: "u64",   precedence: Precedence::None },
    Operator { name: "f64",   precedence: Precedence::None },

    // Constants.
    Operator { name: "e",     precedence: Precedence::None },
    Operator { name: "pi",    precedence: Precedence::None },
];


pub fn find_operator(opname: &str) -> Option<&'static Operator> {
    OPERATORS.iter().find(|op| op.name == opname)
}
