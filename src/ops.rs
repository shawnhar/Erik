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
    Invalid

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


// Helpers reduce repetititiveness of filling in the operator table.
fn make_op(name: &'static str, precedence: Precedence, arity: u32, is_right_associative: bool, function: OpFunction) -> Operator {
    Operator { name, precedence, arity, is_right_associative, function }
}

fn make_op_0(name: &'static str, precedence: Precedence, function: fn() -> f64) -> Operator {
    make_op(name, precedence, 0, false, OpFunction::Nullary(function))
}

fn make_op_1(name: &'static str, precedence: Precedence, function: fn(f64) -> f64) -> Operator {
    make_op(name, precedence, 1, false, OpFunction::Unary(function))
}

fn make_op_2(name: &'static str, precedence: Precedence, function: fn(f64, f64) -> f64) -> Operator {
    make_op(name, precedence, 2, false, OpFunction::Binary(function))
}

fn make_op_lazy(name: &'static str, precedence: Precedence, arity: u32, function: fn(f64) -> usize) -> Operator {
    make_op(name, precedence, arity, false, OpFunction::Lazy(function))
}

fn make_op_invalid(name: &'static str, precedence: Precedence, arity: u32, is_right_associative: bool) -> Operator {
    make_op(name, precedence, arity, is_right_associative, OpFunction::Invalid)
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


lazy_static! {
    pub static ref OPERATORS: [Operator; 58] = [
        // Special markers that should never actually be evaluated.
        make_op_invalid("(", Precedence::Brace,   0, false),
        make_op_invalid(")", Precedence::Brace,   0, false),
        make_op_invalid("=", Precedence::Assign,  2, false),

        // Component parts of the ternary ?: operator.
        make_op_invalid("?", Precedence::Ternary, 2, true),
        make_op_invalid(":", Precedence::Ternary, 2, true),

        // Boolean operators.
        make_op_lazy("||", Precedence::LogicalOr,  2, |x: f64| -> usize { if to_bool(x) { 0 } else { 1 } }),
        make_op_lazy("&&", Precedence::LogicalAnd, 2, |x: f64| -> usize { if to_bool(x) { 1 } else { 0 } }),
        make_op_1("!",     Precedence::Unary,         |x: f64| -> f64   { to_float(!to_bool(x))          }),

        // Bitwise operators.
        make_op_2("|",     Precedence::BinaryOr,    |x: f64, y: f64| -> f64 { (to_int(x)  |  to_int(y))        as f64 }),
        make_op_2("^^",    Precedence::BinaryXor,   |x: f64, y: f64| -> f64 { (to_int(x)  ^  to_int(y))        as f64 }),
        make_op_2("&",     Precedence::BinaryAnd,   |x: f64, y: f64| -> f64 { (to_int(x)  &  to_int(y))        as f64 }),
        make_op_2("<<",    Precedence::Shift,       |x: f64, y: f64| -> f64 { (to_int(x)  << (to_int(y) & 31)) as f64 }),
        make_op_2(">>",    Precedence::Shift,       |x: f64, y: f64| -> f64 { (to_uint(x) >> (to_int(y) & 31)) as f64 }),
        make_op_2(">>>",   Precedence::Shift,       |x: f64, y: f64| -> f64 { (to_int(x)  >> (to_int(y) & 31)) as f64 }),
        make_op_1("~",     Precedence::Unary,       |x: f64| -> f64         { !to_int(x)                       as f64 }),

        // Comparisons
        make_op_2("==",    Precedence::CompareEq,   |x: f64, y: f64| -> f64 { to_float(x == y) }),
        make_op_2("!=",    Precedence::CompareEq,   |x: f64, y: f64| -> f64 { to_float(x != y) }),
        make_op_2("<",     Precedence::CompareDiff, |x: f64, y: f64| -> f64 { to_float(x < y)  }),
        make_op_2(">",     Precedence::CompareDiff, |x: f64, y: f64| -> f64 { to_float(x > y)  }),
        make_op_2("<=",    Precedence::CompareDiff, |x: f64, y: f64| -> f64 { to_float(x <= y) }),
        make_op_2(">=",    Precedence::CompareDiff, |x: f64, y: f64| -> f64 { to_float(x >= y) }),

        // Arithmetic.
        make_op_2("+",     Precedence::Addition,    |x: f64, y: f64| -> f64 { x + y }),
        make_op_2("-",     Precedence::Addition,    |x: f64, y: f64| -> f64 { x - y }),
        make_op_2("*",     Precedence::Multiply,    |x: f64, y: f64| -> f64 { x * y }),
        make_op_2("/",     Precedence::Multiply,    |x: f64, y: f64| -> f64 { x / y }),
        make_op_2("%",     Precedence::Multiply,    |x: f64, y: f64| -> f64 { x.rem_euclid(y) }),
        make_op_2("^",     Precedence::Power,       |x: f64, y: f64| -> f64 { x.powf(y) }),

        // Math functions.
        make_op_2("max",   Precedence::None,        |x: f64, y: f64| -> f64 { if x > y { x } else { y } }),
        make_op_2("min",   Precedence::None,        |x: f64, y: f64| -> f64 { if x < y { x } else { y } }),
        make_op_1("sqrt",  Precedence::None,        |x: f64| -> f64 { x.sqrt()  }),
        make_op_1("exp",   Precedence::None,        |x: f64| -> f64 { x.exp()   }),
        make_op_1("ln",    Precedence::None,        |x: f64| -> f64 { x.ln()    }),
        make_op_1("log",   Precedence::None,        |x: f64| -> f64 { x.log10() }),
        make_op_1("log2",  Precedence::None,        |x: f64| -> f64 { x.log2()  }),
        make_op_1("abs",   Precedence::None,        |x: f64| -> f64 { x.abs()   }),
        make_op_1("ceil",  Precedence::None,        |x: f64| -> f64 { x.ceil()  }),
        make_op_1("floor", Precedence::None,        |x: f64| -> f64 { x.floor() }),
        make_op_1("round", Precedence::None,        |x: f64| -> f64 { x.round() }),

        // Trig.
        make_op_1("sin",   Precedence::None,        |x: f64| -> f64 { x.sin()   }),
        make_op_1("cos",   Precedence::None,        |x: f64| -> f64 { x.cos()   }),
        make_op_1("tan",   Precedence::None,        |x: f64| -> f64 { x.tan()   }),
        make_op_1("sinh",  Precedence::None,        |x: f64| -> f64 { x.sinh()  }),
        make_op_1("cosh",  Precedence::None,        |x: f64| -> f64 { x.cosh()  }),
        make_op_1("tanh",  Precedence::None,        |x: f64| -> f64 { x.tanh()  }),
        make_op_1("asin",  Precedence::None,        |x: f64| -> f64 { x.asin()  }),
        make_op_1("acos",  Precedence::None,        |x: f64| -> f64 { x.acos()  }),
        make_op_1("atan",  Precedence::None,        |x: f64| -> f64 { x.atan()  }),
        make_op_1("asinh", Precedence::None,        |x: f64| -> f64 { x.asinh() }),
        make_op_1("acosh", Precedence::None,        |x: f64| -> f64 { x.acosh() }),
        make_op_1("atanh", Precedence::None,        |x: f64| -> f64 { x.atanh() }),

        // Casts.
        make_op_1("i8",    Precedence::None,        |x: f64| -> f64 { (x as i64 as i8)  as f64 }),
        make_op_1("u8",    Precedence::None,        |x: f64| -> f64 { (x as i64 as u8)  as f64 }),
        make_op_1("i16",   Precedence::None,        |x: f64| -> f64 { (x as i64 as i16) as f64 }),
        make_op_1("u16",   Precedence::None,        |x: f64| -> f64 { (x as i64 as u16) as f64 }),
        make_op_1("i32",   Precedence::None,        |x: f64| -> f64 { (x as i64 as i32) as f64 }),
        make_op_1("u32",   Precedence::None,        |x: f64| -> f64 { (x as i64 as u32) as f64 }),

        // Constants.
        make_op_0("e",     Precedence::None,        || -> f64 { f64::consts::E  }),
        make_op_0("pi",    Precedence::None,        || -> f64 { f64::consts::PI }),
    ];


    // Special operators, not accessible by name.
    pub static ref NEGATE:     Operator = make_op_1("-",             Precedence::Unary,         |x: f64| -> f64 { -x });
    pub static ref TERNARY:    Operator = make_op_lazy("?:",         Precedence::Ternary,    3, |x: f64| -> usize { if to_bool(x) { 1 } else { 2 } });
    pub static ref TERMINATOR: Operator = make_op_invalid("{arnie}", Precedence::Terminator, 0, false);
}


pub fn find_operator(opname: &str) -> Option<OperatorRef> {
    // Linear search is fine as there aren't that many operators and their names are short.
    OPERATORS.iter().find(|op| op == opname)
}
