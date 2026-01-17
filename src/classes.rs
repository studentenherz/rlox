use crate::instances::LoxInstance;
use crate::values::Value;

#[derive(Clone)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
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
}
