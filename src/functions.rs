use std::rc::Rc;

use crate::environments::{Environment, SharedEnv};
use crate::interpreter::{Evaluate, InterpreterCtx, RuntimeError};
use crate::statements::{Identifier, Jump, Statement};
use crate::values::Value;

#[derive(Clone)]
pub enum LoxFunction {
    Builtin {
        function: fn(ctx: &mut InterpreterCtx, arguments: &[Value]) -> Result<Value, RuntimeError>,
        name: String,
        arity: Option<usize>,
    },
    UserDefined {
        name: Identifier,
        parameters: Rc<Vec<Identifier>>,
        body: Rc<Vec<Statement>>,
        closure: SharedEnv,
    },
}

impl LoxFunction {
    pub fn new_user_defined(
        name: Identifier,
        parameters: Vec<Identifier>,
        body: Vec<Statement>,
        closure: SharedEnv,
    ) -> Self {
        Self::UserDefined {
            name,
            parameters: Rc::new(parameters),
            body: Rc::new(body),
            closure,
        }
    }

    pub fn new_builtin(
        name: String,
        arity: Option<usize>,
        function: fn(ctx: &mut InterpreterCtx, arguments: &[Value]) -> Result<Value, RuntimeError>,
    ) -> Self {
        Self::Builtin {
            function,
            name,
            arity,
        }
    }

    pub fn call(
        &self,
        ctx: &mut crate::interpreter::InterpreterCtx,
        arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
        match self {
            Self::UserDefined {
                parameters,
                body,
                closure,
                ..
            } => {
                let env = Environment::new_with_enclosing(closure.clone());
                let mut function_ctx =
                    InterpreterCtx::new_explicit(ctx.lookup_env(None), env, ctx.jump.clone());

                for (param, arg) in parameters.iter().zip(arguments) {
                    function_ctx
                        .lookup_env(Some(0))
                        .borrow_mut()
                        .define(&param.name, arg.clone());
                }

                for statement in body.iter() {
                    statement.evaluate(&mut function_ctx)?;

                    let jump_value = function_ctx.jump.borrow().clone();
                    if let Some(Jump::Return(value)) = jump_value {
                        *function_ctx.jump.borrow_mut() = None;
                        return Ok(value);
                    }
                }

                Ok(Value::Nil)
            }
            Self::Builtin { function, .. } => function(ctx, arguments),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::UserDefined { name, .. } => name.name.clone(),
            Self::Builtin { name, .. } => name.clone(),
        }
    }

    pub fn arity(&self) -> Option<usize> {
        match self {
            Self::UserDefined { parameters, .. } => Some(parameters.len()),
            Self::Builtin { arity, .. } => *arity,
        }
    }
}
