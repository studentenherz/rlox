use std::collections::HashMap;

use crate::classes::LoxClass;
use crate::errors::LoxError;
use crate::statements::Identifier;
use crate::values::Value;

#[derive(Clone)]
pub struct LoxInstance {
    pub fields: HashMap<String, Value>,
    pub class: LoxClass,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, this: Value, ident: &Identifier) -> Result<Value, LoxError> {
        if let Some(value) = self.fields.get(&ident.name) {
            return Ok(value.clone());
        }

        if let Some(method) = self.class.get_method(&ident.name) {
            let method = unsafe { method.try_get_function().unwrap_unchecked() };
            return Ok(Value::function(method.bind(this)));
        }

        Err(LoxError::new_with_span(
            &format!("Undefined property '{}'.", &ident.name),
            ident.span.clone(),
        ))
    }

    pub fn set(&mut self, ident: &Identifier, value: &Value) {
        self.fields.insert(ident.name.clone(), value.clone());
    }
}
