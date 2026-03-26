use crate::{Literal, RuntimeError, RuntimeResult, Token};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, k: String, v: Literal) {
        self.values.insert(k, v);
    }

    fn get(&self, name: Token) -> RuntimeResult<Literal> {
        let lexeme = name.lexeme;
        Ok(self
            .values
            .get(&lexeme)
            .ok_or_else(|| RuntimeError::Undefined { lexeme })?
            .clone())
    }
}
