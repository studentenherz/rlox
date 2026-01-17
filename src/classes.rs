use std::collections::HashMap;

use crate::instances::LoxInstance;
use crate::values::Value;

#[derive(Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Value>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Value>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn call(
        &self,
        _ctx: &mut crate::interpreter::InterpreterCtx,
        _arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
        let instance = LoxInstance::new(self.clone());

        Ok(Value::instance(instance))
    }

    pub fn arity(&self) -> Option<usize> {
        Some(0)
    }

    pub fn get_method(&self, name: &str) -> Option<Value> {
        self.methods.get(name).cloned()
    }
}
