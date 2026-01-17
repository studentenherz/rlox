use std::collections::HashMap;

use crate::classes::LoxClass;
use crate::interpreter::RuntimeError;
use crate::statements::Identifier;
use crate::values::Value;

#[derive(Clone)]
pub struct LoxInstance {
    pub props: HashMap<String, Value>,
    pub class: LoxClass,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            props: HashMap::new(),
        }
    }

    pub fn get(&self, ident: &Identifier) -> Result<Value, RuntimeError> {
        if let Some(value) = self.props.get(&ident.name) {
            return Ok(value.clone());
        }

        Err(RuntimeError::new_attr_error(
            &format!("undefined property '{}'.", &ident.name),
            ident.span.clone(),
        ))
    }

    pub fn set(&mut self, ident: &Identifier, value: &Value) {
        self.props.insert(ident.name.clone(), value.clone());
    }
}
