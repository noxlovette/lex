use crate::{Expr, RuntimeError, RuntimeResult, TokenType, Value};
pub struct Interpreter;

impl Interpreter {
    pub fn eval(expr: Box<Expr>) -> RuntimeResult<Value> {
        use Expr::*;
        use TokenType::*;
        match *expr {
            Literal { value } => value.into(),
            Grouping { expression } => Ok(Self::eval(expression)?),
            Unary { operator, right } => {
                let right = Self::eval(right)?;
                match operator.token_type {
                    Minus => Ok((-right)?),
                    Bang => Ok(!right),
                    _ => unimplemented!(),
                }
            }
            Binary {
                left,
                operator,
                right,
            } => {
                let left = Self::eval(left)?;
                let right = Self::eval(right)?;
                match operator.token_type {
                    Minus => Ok((left - right)?),
                    Slash => Ok((left / right)?),
                    Star => Ok((left * right)?),
                    Plus => Ok((left + right)?),
                    Greater => Ok((left > right).into()),
                    GreaterEqual => Ok((left >= right).into()),
                    Less => Ok((left < right).into()),
                    LessEqual => Ok((left <= right).into()),
                    BangEqual => Ok((left != right).into()),
                    EqualEqual => Ok((left == right).into()),
                    _ => unreachable!(),
                }
            }
            _ => Err(RuntimeError::TypeError("Not Implemented".to_string())),
        }
    }
}
