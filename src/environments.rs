use std::collections::HashMap;

use crate::values::Value;

pub struct Environment {
    vars: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value, ()> {
        if let Some(var) = self.vars.get_mut(name) {
            *var = value.clone();
            Ok(value)
        } else {
            Err(())
        }
    }
}
