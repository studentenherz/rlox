use std::time::{SystemTime, UNIX_EPOCH};

use crate::interpreter::RuntimeError;

pub fn lox_builtin_clock(
    _ctx: &mut crate::interpreter::InterpreterCtx,
    _arguments: &[crate::values::Value],
) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(crate::values::Value::Number(duration.as_secs_f64())),
        Err(err) => Err(RuntimeError::new_system_error(&format!("{}", err))),
    }
}
