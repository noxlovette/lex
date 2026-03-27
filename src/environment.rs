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

    pub fn assign(&mut self, name: &Token, value: &Value) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values
                .entry(name.lexeme.clone())
                .and_modify(|e| *e = value.clone());
            Ok(())
        } else {
            Err(RuntimeError::Undefined {
                lexeme: name.lexeme.clone(),
            })
        }
    }
}
