use crate::token::Token;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Default for Literal {
    fn default() -> Self {
        Self::Nil
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

#[derive(Clone)]
pub enum Expr {
    Unary {
        operator: Token,
        right: Rc<Expr>,
    },
    Assign {
        name: Token,
        value: Rc<Expr>,
    },
    Binary {
        left: Rc<Expr>,
        operator: Token,
        right: Rc<Expr>,
    },
    Grouping {
        expression: Rc<Expr>,
    },
    Literal {
        value: Literal,
    },
    Variable {
        name: Token,
    },
    Logical {
        left: Rc<Expr>,
        operator: Token,
        right: Rc<Expr>,
    },
    Call {
        callee: Rc<Expr>,
        paren: Token, // only used for error reporting
        arguments: Vec<Rc<Expr>>,
    },
    Get {
        object: Rc<Expr>, // this is the expression on the left of the call, meaning: expr.name
        name: Token,
    },
    Set {
        object: Rc<Expr>,
        name: Token,
        value: Rc<Expr>,
    },
    This {
        keyword: Token,
    },
    Super {
        keyword: Token,
        method: Token,
    },
}

impl Expr {
    pub fn as_superclass(&self) -> Option<&Token> {
        match self {
            Expr::Variable { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Consumes the value and wraps it in an RC
    pub fn rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

/*
Wrapper around Rc<Expr> that uses pointer-based equality and hashing.
This allows to use expression references as HashMap keys based on their
identity (where they are in memory) rather than their content.
 */
#[derive(Clone)]
pub struct ExprRef(pub Rc<Expr>);

impl ExprRef {
    pub fn new(expr: Expr) -> Self {
        ExprRef(Rc::new(expr))
    }

    pub fn from_rc(rc: Rc<Expr>) -> Self {
        ExprRef(rc)
    }
}

impl std::ops::Deref for ExprRef {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Pointer-based equality: two ExprRefs are equal if they point to the same allocation
impl PartialEq for ExprRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ExprRef {}

// Pointer-based hashing: hash the pointer address, not the content
impl Hash for ExprRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Rc::as_ptr(&self.0), state)
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

    fn parenthesize(&self, name: &str, exprs: &[&Rc<Expr>]) -> String {
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
