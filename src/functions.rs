use crate::environments::Environment;
use crate::interpreter::{Evaluate, InterpreterCtx, LoxCallable};
use crate::statements::{Identifier, Statement};
use crate::values::Value;

pub struct LoxFunction {
    name: Identifier,
    parameters: Vec<Identifier>,
    body: Vec<Statement>,
}

impl LoxFunction {
    pub fn new(name: Identifier, parameters: Vec<Identifier>, body: Vec<Statement>) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        ctx: &mut crate::interpreter::InterpreterCtx,
        arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
        let env = Environment::new();
        let mut function_ctx = InterpreterCtx {
            globals: ctx.globals.clone(),
            env,
            jump: ctx.jump.clone(),
        };

        for (param, arg) in self.parameters.iter().zip(arguments) {
            function_ctx
                .env
                .borrow_mut()
                .define(&param.name, arg.clone());
        }

        for statement in &self.body {
            statement.evaluate(&mut function_ctx)?;
        }

        Ok(Value::Nil)
    }

    fn name(&self) -> String {
        self.name.name.clone()
    }

    fn arity(&self) -> Option<usize> {
        Some(self.parameters.len())
    }
}
