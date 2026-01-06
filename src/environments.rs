use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::values::Value;

pub type SharedEnv = Rc<RefCell<Environment>>;

pub struct Environment {
    vars: HashMap<String, Value>,
    pub enclosing: Option<SharedEnv>,
}

impl Environment {
    pub fn new() -> SharedEnv {
        Rc::new(RefCell::new(Self {
            vars: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_with_enclosing(env: SharedEnv) -> SharedEnv {
        Rc::new(RefCell::new(Self {
            vars: HashMap::new(),
            enclosing: Some(env),
        }))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.vars.get(name) {
            return Some(value.clone());
        }

        self.enclosing
            .as_ref()
            .and_then(|encl| encl.borrow().get(name))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value, ()> {
        if let Some(var) = self.vars.get_mut(name) {
            *var = value.clone();
            Ok(value)
        } else {
            if let Some(encl) = &mut self.enclosing {
                encl.borrow_mut().assign(name, value)
            } else {
                Err(())
            }
        }
    }
}
