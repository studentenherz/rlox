use std::time::{SystemTime, UNIX_EPOCH};
use std::usize;

use crate::interpreter::{LoxCallable, RuntimeError};

#[derive(PartialEq)]
pub struct ClockBuiltin {}

impl ClockBuiltin {
    pub fn new() -> Self {
        Self {}
    }
}

impl LoxCallable for ClockBuiltin {
    fn name(&self) -> String {
        "clock".to_string()
    }

    fn arity(&self) -> Option<usize> {
        Some(0)
    }

    fn call(
        &self,
        _ctx: &mut crate::interpreter::InterpreterCtx,
        _arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
        let now = SystemTime::now();
        match now.duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(crate::values::Value::Number(duration.as_secs_f64())),
            Err(err) => Err(RuntimeError::new_system_error(&format!("{}", err))),
        }
    }
}
