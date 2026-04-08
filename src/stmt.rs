use crate::{Expr, Token};

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    Expression {
        expression: Box<Expr>,
    },
    Print {
        expression: Box<Expr>,
    },
    Var {
        name: Token,
        initializer: Box<Option<Expr>>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Box<Expr>,
    },
    Class {
        name: Token,
        super_class: Option<Expr>,
        methods: Vec<Stmt>,
    },
}

impl Stmt {
    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }
}
