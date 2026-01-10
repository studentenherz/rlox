use crate::interpreter::LoxCallable;

#[derive(PartialEq)]
pub struct PrintBuiltin {}

impl PrintBuiltin {
    pub fn new() -> Self {
        Self {}
    }
}

impl LoxCallable for PrintBuiltin {
    fn name(&self) -> String {
        "__builtin_print".to_string()
    }

    fn arity(&self) -> Option<usize> {
        None
    }

    fn call(
        &self,
        _ctx: &mut crate::interpreter::InterpreterCtx,
        arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
        let string_args: Vec<String> = arguments.iter().map(|v| v.to_string()).collect();
        println!("{}", string_args.join(" "));

        Ok(crate::values::Value::Nil)
    }
}
