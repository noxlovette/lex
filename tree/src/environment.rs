use crate::{RuntimeError, RuntimeResult, Token, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Default, PartialEq)]
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

    pub fn rc(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn with_enclosing(mut self, enclosing: Rc<RefCell<Environment>>) -> Self {
        self.enclosing = Some(enclosing);
        self
    }

    pub fn get_at(&self, distance: usize, lexeme: &str) -> RuntimeResult<Value> {
        if distance == 0 {
            self.values
                .get(lexeme)
                .ok_or(RuntimeError::Undefined {
                    lexeme: lexeme.to_string(),
                })
                .cloned()
        } else {
            self.enclosing
                .as_ref()
                .expect("resolver produced an invalid environment distance")
                .borrow()
                .get_at(distance - 1, lexeme)
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Value) -> RuntimeResult<()> {
        if distance == 0 {
            if self.values.contains_key(&name.lexeme) {
                self.values.insert(name.lexeme.clone(), value);
                Ok(())
            } else {
                Err(RuntimeError::Undefined {
                    lexeme: name.lexeme.clone(),
                })
            }
        } else {
            self.enclosing
                .as_ref()
                .expect("resolver produced an invalid environment distance")
                .borrow_mut()
                .assign_at(distance - 1, name, value)
        }
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
