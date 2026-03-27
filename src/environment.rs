use crate::{RuntimeError, RuntimeResult, Token, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_enclosing(mut self, enclosing: Environment) -> Self {
        self.enclosing = Some(Rc::new(RefCell::new(enclosing)));
        self
    }

    pub fn define(&mut self, k: String, v: Option<Value>) {
        self.values.insert(k, v.unwrap_or_default());
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Value> {
        let lexeme = name.lexeme.clone();

        if let Some(value) = self.values.get(&lexeme) {
            Ok(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(RuntimeError::Undefined { lexeme })
        }
    }
    pub fn assign(&mut self, name: &Token, value: &Value) -> RuntimeResult<()> {
        let lexeme = name.lexeme.clone();

        if self.values.contains_key(&lexeme) {
            self.values.entry(lexeme).and_modify(|e| *e = value.clone());
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::Undefined { lexeme })
        }
    }
}
