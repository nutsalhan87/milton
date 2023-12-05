use crate::util::error_code;

#[derive(Debug)]
pub enum Expression {
    FnDef {
        name: String,
        arguments: Vec<String>,
        expr: Box<Expression>,
    },
    Case {
        condition: Box<Expression>,
        t: Box<Expression>,
        f: Box<Expression>,
    },
    For {
        var: String,
        next_val: Box<Expression>,
        while_expr: Box<Expression>,
        expr: Box<Expression>,
    },
    Fn {
        name: String,
        args: Vec<Expression>,
    },
    VarDef {
        name: String,
        init: Box<Expression>,
        expr: Box<Expression>,
    },
    Var(String),
    Str(String),
    Value(i32),
}

pub enum ExpressionType {
    Char,
    Number,
    Literal,
    Args,
    FnDef,
    For,
    FnOrVar,
    VarDef,
}

impl ExpressionType {
    pub fn to_explained_string(&self, s: &str) -> String {
        match self {
            Self::Char => format!("A character was expected here: {}...", error_code(s)),
            Self::Number => format!("A number was expected here: {}...", error_code(s)),
            Self::Literal => format!("A literal was expected here: {}...", error_code(s)),
            Self::Args => format!("An arguments were expected here: {}...", error_code(s)),
            Self::FnDef => format!(
                "A function definiton was expected here: {}...",
                error_code(s)
            ),
            Self::For => format!("A for cycle was expected here: {}...", error_code(s)),
            Self::FnOrVar => format!(
                "A function call or variable reference was expected here: {}...",
                error_code(s)
            ),
            Self::VarDef => format!(
                "A variable definition was expected here: {}...",
                error_code(s)
            ),
        }
    }
}
