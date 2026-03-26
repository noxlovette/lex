use crate::{RuntimeError, RuntimeResult, Token, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, k: String, v: Option<Value>) {
        self.values.insert(k, v.unwrap_or_default());
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Value> {
        let lexeme = name.lexeme.clone();
        Ok(self
            .values
            .get(&lexeme)
            .ok_or_else(|| RuntimeError::Undefined { lexeme })?
            .clone())
    }
}
