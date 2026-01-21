use crate::errors::LoxError;

pub fn lox_builtin_print(
    _ctx: &mut crate::interpreter::InterpreterCtx,
    arguments: &[crate::values::Value],
) -> Result<crate::values::Value, LoxError> {
    let string_args: Vec<String> = arguments.iter().map(|v| v.to_string()).collect();
    println!("{}", string_args.join(" "));

    Ok(crate::values::Value::Nil)
}
