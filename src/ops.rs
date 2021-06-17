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


lazy_static! {
    pub static ref OPERATORS: [Operator; 52] = [
        // Special markers that should never actually be evaluated.
        make_op_invalid("(", Precedence::Brace,   0, false),
        make_op_invalid(")", Precedence::Brace,   0, false),
        make_op_invalid("=", Precedence::Assign,  2, false),

        // Component parts of the ternary ?: operator.
        make_op_invalid("?", Precedence::Ternary, 2, true),
        make_op_invalid(":", Precedence::Ternary, 2, true),

        // Logical operators use short circuit evaluation.
        make_op_lazy("||", Precedence::LogicalOr,  2, |_x: f64| -> usize { 0 }),
        make_op_lazy("&&", Precedence::LogicalAnd, 2, |_x: f64| -> usize { 0 }),

        // Binary operators.
        make_op_2("|",     Precedence::BinaryOr,    |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("^^",    Precedence::BinaryXor,   |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("&",     Precedence::BinaryAnd,   |_x: f64, _y: f64| -> f64 { 0.0 }),

        // Comparisons
        make_op_2("==",    Precedence::CompareEq,   |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("!=",    Precedence::CompareEq,   |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("<",     Precedence::CompareDiff, |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2(">",     Precedence::CompareDiff, |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("<=",    Precedence::CompareDiff, |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2(">=",    Precedence::CompareDiff, |_x: f64, _y: f64| -> f64 { 0.0 }),

        // Shifts.
        make_op_2("<<",    Precedence::Shift,       |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2(">>",    Precedence::Shift,       |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2(">>>",   Precedence::Shift,       |_x: f64, _y: f64| -> f64 { 0.0 }),

        // Arithmetic.
        make_op_2("+",     Precedence::Addition,    |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("-",     Precedence::Addition,    |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("*",     Precedence::Multiply,    |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("/",     Precedence::Multiply,    |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("%",     Precedence::Multiply,    |_x: f64, _y: f64| -> f64 { 0.0 }),

        // Negation.
        make_op_1("!",     Precedence::Unary,       |_x: f64| -> f64 { 0.0 }),
        make_op_1("~",     Precedence::Unary,       |_x: f64| -> f64 { 0.0 }),

        // Raise to a power.
        make_op_2("^",     Precedence::Power,       |_x: f64, _y: f64| -> f64 { 0.0 }),

        // Math functions.
        make_op_2("max",   Precedence::None,        |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_2("min",   Precedence::None,        |_x: f64, _y: f64| -> f64 { 0.0 }),
        make_op_1("sqrt",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("exp",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("ln",    Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("log",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("ceil",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("floor", Precedence::None,        |_x: f64| -> f64 { 0.0 }),

        // Trig.
        make_op_1("sin",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("cos",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("tan",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("sinh",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("cosh",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("tanh",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("asin",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("acos",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("atan",  Precedence::None,        |_x: f64| -> f64 { 0.0 }),

        // Casts.
        make_op_1("s8",    Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("u8",    Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("s16",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("u16",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("s32",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),
        make_op_1("u32",   Precedence::None,        |_x: f64| -> f64 { 0.0 }),

        // Constants.
        make_op_0("e",     Precedence::None,        || -> f64 { 0.0 }),
        make_op_0("pi",    Precedence::None,        || -> f64 { 0.0 }),
    ];


    // Special operators, not accessible by name.
    pub static ref NEGATE:     Operator = make_op_1("-",             Precedence::Unary,         |_x: f64| -> f64 { 0.0 });
    pub static ref TERNARY:    Operator = make_op_lazy("?:",         Precedence::Ternary,    3, |_x: f64| -> usize { 0 });
    pub static ref TERMINATOR: Operator = make_op_invalid("{arnie}", Precedence::Terminator, 0, false);
}


pub fn find_operator(opname: &str) -> Option<OperatorRef> {
    // Linear search is fine as there aren't that many operators and their names are short.
    OPERATORS.iter().find(|op| op == opname)
}
