use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::classes::LoxClass;
use crate::errors::LoxError;
use crate::expressions::Literal;
use crate::functions::LoxFunction;
use crate::instances::LoxInstance;
use crate::statements::Identifier;

#[derive(Clone)]
pub enum LoxCallable {
    Function(Rc<LoxFunction>),
    Class(Rc<LoxClass>),
}

impl LoxCallable {
    pub fn call(
        &self,
        ctx: &mut crate::interpreter::InterpreterCtx,
        arguments: Vec<Value>,
    ) -> Result<crate::values::Value, LoxError> {
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
    Instance(Rc<RefCell<LoxInstance>>),
    Unassigned,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(this), Self::Boolean(other)) => this == other,
            (Self::Number(this), Self::Number(other)) => this == other,
            (Self::String(this), Self::String(other)) => this == other,
            (
                Self::Callable(LoxCallable::Class(this)),
                Self::Callable(LoxCallable::Class(other)),
            ) => Rc::as_ptr(&this) == Rc::as_ptr(&other),
            (
                Self::Callable(LoxCallable::Function(this)),
                Self::Callable(LoxCallable::Function(other)),
            ) => Rc::as_ptr(&this) == Rc::as_ptr(&other),
            (Self::Instance(this), Self::Instance(other)) => {
                Rc::as_ptr(&this) == Rc::as_ptr(&other)
            }
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
                write!(f, "{}", function)
            }
            Self::Callable(LoxCallable::Class(class)) => {
                write!(f, "{}", class.name())
            }
            Self::Instance(instance) => {
                write!(f, "{} instance", instance.borrow().class.name())
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
                write!(f, "<fn {}>", function.name())
            }
            Self::Callable(LoxCallable::Class(class)) => {
                write!(f, "{}", class.name())
            }
            Self::Instance(instance) => {
                write!(f, "{} instance", instance.borrow().class.name())
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
        Self::Callable(LoxCallable::Function(Rc::new(function)))
    }

    pub fn class(class: LoxClass) -> Self {
        Self::Callable(LoxCallable::Class(Rc::new(class)))
    }

    pub fn instance(instance: LoxInstance) -> Self {
        Self::Instance(Rc::new(RefCell::new(instance)))
    }

    pub fn type_name(&self) -> String {
        match self {
            Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Callable(_) => "callable",
            Self::Unassigned => "unassigned",
            Self::Instance(instance) => return instance.borrow().class.name(),
        }
        .to_string()
    }

    pub fn try_get_callable(&self) -> Result<&LoxCallable, &Self> {
        match self {
            Self::Callable(callable) => Ok(callable),
            _ => Err(self),
        }
    }

    pub fn try_get_function(&self) -> Result<Rc<LoxFunction>, ()> {
        match self {
            Self::Callable(LoxCallable::Function(function)) => Ok(function.clone()),
            _ => Err(()),
        }
    }

    pub fn try_get_property(&self, ident: &Identifier) -> Result<Value, LoxError> {
        match self {
            Self::Instance(instance) => instance.borrow().get(self.clone(), ident),
            _ => Err(LoxError::new_with_span(
                "Only instances have properties.",
                ident.span.clone(),
            )),
        }
    }

    pub fn try_set_property(&self, ident: &Identifier, value: &Value) -> Result<Value, LoxError> {
        match self {
            Self::Instance(instance) => {
                instance.borrow_mut().set(ident, value);
                Ok(value.clone())
            }
            _ => Err(LoxError::new_with_span(
                "Only instances have fields.",
                ident.span.clone(),
            )),
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
