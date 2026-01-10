use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::expressions::Literal;
use crate::interpreter::LoxCallable;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn LoxCallable>),
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
            Self::Callable(callable) => write!(f, "<function {}>", callable.name()),
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
            Self::Callable(callable) => write!(f, "<function {}>", callable.name()),
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
}

impl From<&Value> for bool {
    fn from(value: &Value) -> Self {
        match value {
            Value::Nil | Value::Boolean(false) => false,
            _ => true,
        }
    }
}
