use crate::token::Token;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    #[default]
    Nil,
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;
        match self {
            Number(n) => {
                write!(f, "{n}")
            }
            String(s) => {
                write!(f, "{s}")
            }
            Bool(b) => write!(f, "{b}"),
            Nil => write!(f, "nil"),
        }
    }
}

impl Eq for Literal {}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::String(value), Literal::String(other_value)) => value.eq(other_value),
            (Literal::Number(value), Literal::Number(other_value)) => value == other_value,
            (Literal::Bool(value), Literal::Bool(other_value)) => value == other_value,
            (Literal::Nil, Literal::Nil) => true,
            (_, _) => false,
        }
    }
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::String(value) => value.hash(state),
            Literal::Number(value) => value.to_bits().hash(state),
            Literal::Bool(value) => value.hash(state),
            Literal::Nil => "".hash(state),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum Expr {
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Assign {
        id: usize,
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Variable {
        id: usize,
        name: Token,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token, // only used for error reporting
        arguments: Vec<Box<Expr>>,
    },
    Get {
        object: Box<Expr>, // this is the expression on the left of the call, meaning: expr.name
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This {
        id: usize,
        keyword: Token,
    },
    Super {
        id: usize,
        keyword: Token,
        method: Token,
    },
}

impl Expr {
    pub fn id(&self) -> Option<usize> {
        match self {
            Expr::Assign { id, .. }
            | Expr::Variable { id, .. }
            | Expr::This { id, .. }
            | Expr::Super { id, .. } => Some(*id),
            _ => None,
        }
    }

    pub fn as_superclass(&self) -> Option<&Token> {
        match self {
            Expr::Variable { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Consumes the value and wraps it in a Box
    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }
}

/*
Wrapper around Box<Expr> that uses pointer-based equality and hashing.
This allows to use expression references as HashMap keys based on their
identity (where they are in memory) rather than their content.
 */
#[derive(Clone)]
pub struct ExprRef(pub Box<Expr>);

impl ExprRef {
    pub fn new(expr: Expr) -> Self {
        ExprRef(Box::new(expr))
    }

    pub fn from_rc(rc: Box<Expr>) -> Self {
        ExprRef(rc)
    }
}

impl std::ops::Deref for ExprRef {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, &[left, right]),
            Expr::Grouping { expression } => self.parenthesize("group", &[expression]),
            Expr::Literal { value } => format!("{value}"),
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, &[right]),
            _ => "<expr>".to_string(),
        }
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let mut out = String::new();
        out.push('(');
        out.push_str(name);

        for expr in exprs {
            out.push(' ');
            out.push_str(&self.print(expr));
        }

        out.push(')');
        out
    }
}
