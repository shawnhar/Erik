use std::f64;


// Ordering of these enum values determines parser behavior.
#[derive(Clone, Copy, Debug)]
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


// Operators can be implemented using functions that take various numbers of parameters.
#[derive(Debug)]
pub enum OpFunction {
    Nullary(fn()         -> f64),
    Unary  (fn(f64)      -> f64),
    Binary (fn(f64, f64) -> f64),
    Lazy   (fn(f64)      -> usize),
    Invalid,

    // Lazy operators take the value of their first parameter, and return the index of which
    // other parameter should be evaluated and used as the result of the expression. This
    // provides short circuit evaluation for logical || and && plus ternary ?: operators.
}


// Implementation of an operator or builtin function.
#[derive(Debug)]
pub struct Operator {
    pub name:                 &'static str,
    pub precedence:           Precedence,
    pub arity:                u32,
    pub is_right_associative: bool,
    pub function:             OpFunction,
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


// Macros reduce repetitititiveness of filling in the operator table.
macro_rules! operators {
    ($($element:tt),*) => {
        [ $(operator! $element),* ]
    };
}


macro_rules! operator {
    // Matches a nullary function.
    ($name:literal, || $expression:expr) => {
        Operator { name: $name, precedence: Precedence::None, arity: 0, is_right_associative: false, function: OpFunction::Nullary(|| -> f64 { $expression }) }
    };

    // Matches a unary function.
    ($name:literal, |$x:ident| $expression:expr) => {
        Operator { name: $name, precedence: Precedence::None, arity: 1, is_right_associative: false, function: OpFunction::Unary(|$x: f64| -> f64 { $expression }) }
    };

    // Matches a unary operator.
    ($name:literal, $precedence:expr, |$x:ident| $expression:expr) => {
        Operator { name: $name, precedence: $precedence, arity: 1, is_right_associative: false, function: OpFunction::Unary(|$x: f64| -> f64 { $expression }) }
    };

    // Matches a binary function.
    ($name:literal, |$x:ident, $y:ident| $expression:expr) => {
        Operator { name: $name, precedence: Precedence::None, arity: 2, is_right_associative: false, function: OpFunction::Binary(|$x: f64, $y: f64| -> f64 { $expression }) }
    };

    // Matches a binary operator.
    ($name:literal, $precedence:expr, |$x:ident, $y:ident| $expression:expr) => {
        Operator { name: $name, precedence: $precedence, arity: 2, is_right_associative: false, function: OpFunction::Binary(|$x: f64, $y: f64| -> f64 { $expression }) }
    };

    // Matches a lazily evaluated operator, identified by "lazy" marker keyword.
    ($name:literal, $precedence:expr, $arity:literal, lazy |$x:ident| $expression:expr) => {
        Operator { name: $name, precedence: $precedence, arity: $arity, is_right_associative: false, function: OpFunction::Lazy(|$x: f64| -> usize { $expression }) }
    };

    // Matches a special operator that does not have any evaluation function.
    ($name:literal, $precedence:expr, $arity:literal, $is_right_associative:literal) => {
        Operator { name: $name, precedence: $precedence, arity: $arity, is_right_associative: $is_right_associative, function: OpFunction::Invalid }
    };
}


// Helpers for converting values between different types.
fn to_bool(x: f64) -> bool {
    x != 0.0
}

fn to_float(x: bool) -> f64 {
    if x { 1.0 } else { 0.0 }
}

fn to_int(x: f64) -> i32 {
    x as i64 as i32
}

fn to_uint(x: f64) -> u32 {
    x as i64 as u32
}


pub static OPERATORS: [Operator; 27] = operators![
    // Special markers that should never actually be evaluated.
    { "(",   Precedence::Brace,      0, false },
    { ")",   Precedence::Brace,      0, false },
    { "=",   Precedence::Assign,     2, false },

    // Component parts of the ternary ?: operator.
    { "?",   Precedence::Ternary,    2, true },
    { ":",   Precedence::Ternary,    2, true },

    // Boolean operators.
    { "||",  Precedence::LogicalOr,  2, lazy |x| if to_bool(x) {0} else {1} },
    { "&&",  Precedence::LogicalAnd, 2, lazy |x| if to_bool(x) {1} else {0} },
    { "!",   Precedence::Unary,         |x| to_float(!to_bool(x)) },

    // Bitwise operators.
    { "|",   Precedence::BinaryOr,      |x, y| (to_int(x)  |  to_int(y))        as f64 },
    { "^^",  Precedence::BinaryXor,     |x, y| (to_int(x)  ^  to_int(y))        as f64 },
    { "&",   Precedence::BinaryAnd,     |x, y| (to_int(x)  &  to_int(y))        as f64 },
    { "<<",  Precedence::Shift,         |x, y| (to_int(x)  << (to_int(y) & 31)) as f64 },
    { ">>",  Precedence::Shift,         |x, y| (to_uint(x) >> (to_int(y) & 31)) as f64 },
    { ">>>", Precedence::Shift,         |x, y| (to_int(x)  >> (to_int(y) & 31)) as f64 },
    { "~",   Precedence::Unary,         |x|    !to_int(x)                       as f64 },

    // Comparisons
    { "==",  Precedence::CompareEq,     |x, y| to_float(x == y) },
    { "!=",  Precedence::CompareEq,     |x, y| to_float(x != y) },
    { "<",   Precedence::CompareDiff,   |x, y| to_float(x < y)  },
    { ">",   Precedence::CompareDiff,   |x, y| to_float(x > y)  },
    { "<=",  Precedence::CompareDiff,   |x, y| to_float(x <= y) },
    { ">=",  Precedence::CompareDiff,   |x, y| to_float(x >= y) },

    // Arithmetic.
    { "+",   Precedence::Addition,      |x, y| x + y },
    { "-",   Precedence::Addition,      |x, y| x - y },
    { "*",   Precedence::Multiply,      |x, y| x * y },
    { "/",   Precedence::Multiply,      |x, y| x / y },
    { "%",   Precedence::Multiply,      |x, y| x.rem_euclid(y) },
    { "^",   Precedence::Power,         |x, y| x.powf(y) }
];


pub static FUNCTIONS: [Operator; 31] = operators![
    // Math functions.
    { "max",   |x, y| if x > y {x} else {y} },
    { "min",   |x, y| if x < y {x} else {y} },

    { "sqrt",  |x| x.sqrt()  },
    { "exp",   |x| x.exp()   },
    { "ln",    |x| x.ln()    },
    { "log",   |x| x.log10() },
    { "log2",  |x| x.log2()  },
    { "abs",   |x| x.abs()   },
    { "ceil",  |x| x.ceil()  },
    { "floor", |x| x.floor() },
    { "round", |x| x.round() },

    // Trig.
    { "sin",   |x| x.sin()   },
    { "cos",   |x| x.cos()   },
    { "tan",   |x| x.tan()   },
    { "sinh",  |x| x.sinh()  },
    { "cosh",  |x| x.cosh()  },
    { "tanh",  |x| x.tanh()  },
    { "asin",  |x| x.asin()  },
    { "acos",  |x| x.acos()  },
    { "atan",  |x| x.atan()  },
    { "asinh", |x| x.asinh() },
    { "acosh", |x| x.acosh() },
    { "atanh", |x| x.atanh() },

    // Casts.
    { "i8",    |x| x as i64 as i8  as f64 },
    { "u8",    |x| x as i64 as u8  as f64 },
    { "i16",   |x| x as i64 as i16 as f64 },
    { "u16",   |x| x as i64 as u16 as f64 },
    { "i32",   |x| x as i64 as i32 as f64 },
    { "u32",   |x| x as i64 as u32 as f64 },

    // Constants.
    { "e",     || f64::consts::E  },
    { "pi",    || f64::consts::PI }
];


// Special operators, not accessible by name.
pub static NEGATE:     Operator = operator!{ "-",       Precedence::Unary,         |x| -x };
pub static TERNARY:    Operator = operator!{ "?:",      Precedence::Ternary,    3, lazy |x| if to_bool(x) {1} else {2} };
pub static TERMINATOR: Operator = operator!{ "{arnie}", Precedence::Terminator, 0, false };


pub fn find_operator(opname: &str) -> Option<OperatorRef> {
    OPERATORS.iter().find(|op| op == opname)
}

pub fn find_function(opname: &str) -> Option<OperatorRef> {
    FUNCTIONS.iter().find(|op| op == opname)
}
