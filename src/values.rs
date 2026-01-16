use std::fmt::{Debug, Display};

use crate::classes::LoxClass;
use crate::expressions::Literal;
use crate::functions::LoxFunction;

#[derive(Clone)]
pub enum LoxCallable {
    Function(LoxFunction),
    Class(LoxClass),
}

impl LoxCallable {
    pub fn call(
        &self,
        ctx: &mut crate::interpreter::InterpreterCtx,
        arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, crate::interpreter::RuntimeError> {
        match self {
            Self::Function(function) => function.call(ctx, arguments),
            Self::Class(class) => class.call(ctx, arguments),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Class(class) => class.name(),
            Self::Function(function) => function.name(),
        }
    }

    pub fn arity(&self) -> Option<usize> {
        match self {
            Self::Class(class) => class.arity(),
            Self::Function(function) => function.arity(),
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(LoxCallable),
    Unassigned,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(this), Self::Boolean(other)) => this == other,
            (Self::Number(this), Self::Number(other)) => this == other,
            (Self::String(this), Self::String(other)) => this == other,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(true) => write!(f, "true"),
            Self::Boolean(false) => write!(f, "false"),
            Self::Number(number) => write!(f, "{}", number),
            Self::String(string) => write!(f, "{}", string),
            Self::Callable(LoxCallable::Function(function)) => {
                write!(f, "<function {}>", function.name())
            }
            Self::Callable(LoxCallable::Class(class)) => {
                write!(f, "<class {}>", class.name())
            }
            Self::Unassigned => Ok(()),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(true) => write!(f, "true"),
            Self::Boolean(false) => write!(f, "false"),
            Self::Number(number) => write!(f, "{}", number),
            Self::String(string) => write!(f, "\"{}\"", string),
            Self::Callable(LoxCallable::Function(function)) => {
                write!(f, "<function {}>", function.name())
            }
            Self::Callable(LoxCallable::Class(class)) => {
                write!(f, "<class {}>", class.name())
            }
            Self::Unassigned => Ok(()),
        }
    }
}

impl Value {
    pub fn from_literal(literal: &Literal) -> Self {
        match literal {
            Literal::Nil => Value::Nil,
            Literal::True => Value::Boolean(true),
            Literal::False => Value::Boolean(false),
            Literal::Number(number) => Value::Number(*number),
            Literal::String(string) => Value::String(string.clone()),
        }
    }

    pub fn function(function: LoxFunction) -> Self {
        Self::Callable(LoxCallable::Function(function))
    }

    pub fn class(class: LoxClass) -> Self {
        Self::Callable(LoxCallable::Class(class))
    }

    pub fn type_name(&self) -> String {
        match self {
            Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Callable(_) => "callable",
            Self::Unassigned => "unassigned",
        }
        .to_string()
    }

    pub fn try_get_callable(&self) -> Result<&LoxCallable, &Self> {
        match self {
            Self::Callable(callable) => Ok(callable),
            _ => Err(self),
        }
    }
}

impl From<&Value> for bool {
    fn from(value: &Value) -> Self {
        match value {
            Value::Nil | Value::Boolean(false) => false,
            _ => true,
        }
    }
}
