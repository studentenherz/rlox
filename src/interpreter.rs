use std::fmt::Display;

use crate::common::Span;
use crate::environments::{Environment, SharedEnv};
use crate::statements::{Statement, StatementKind};
use crate::{expressions::*, values::Value};

#[derive(Debug, PartialEq)]
enum ErrorKind {
    TypeError,
    NameError,
    UnassignedError,
}

#[derive(Debug)]
pub struct RuntimeError {
    kind: ErrorKind,
    reason: String,
    span: Span,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Line {}] {:?}: {}",
            self.span.line, self.kind, self.reason
        )
    }
}

impl RuntimeError {
    pub fn new_type_error(reason: &str, span: Span) -> Self {
        Self {
            kind: ErrorKind::TypeError,
            reason: reason.to_string(),
            span,
        }
    }

    pub fn new_name_error(reason: &str, span: Span) -> Self {
        Self {
            kind: ErrorKind::NameError,
            reason: reason.to_string(),
            span,
        }
    }

    pub fn new_unassigned_error(reason: &str, span: Span) -> Self {
        Self {
            kind: ErrorKind::UnassignedError,
            reason: reason.to_string(),
            span,
        }
    }
}

pub struct InterpreterCtx {
    pub env: SharedEnv,
}

impl InterpreterCtx {
    pub fn new() -> Self {
        InterpreterCtx {
            env: Environment::new(),
        }
    }
}

pub trait Evaluate {
    type Value;

    fn evaluate(&self, ctx: &mut InterpreterCtx) -> Result<Self::Value, RuntimeError>;
}

impl Evaluate for Expr {
    type Value = Value;

    fn evaluate(&self, ctx: &mut InterpreterCtx) -> Result<Self::Value, RuntimeError> {
        match &self.kind {
            ExprKind::Literal { value } => Ok(Value::from_literal(value)),
            ExprKind::Grouping { expression } => expression.evaluate(ctx),
            ExprKind::Unary { operator, right } => {
                let right_value = right.evaluate(ctx)?;

                match operator {
                    UnaryOperator::Minus => {
                        if let Value::Number(number) = right_value {
                            Ok(Value::Number(-number))
                        } else {
                            Err(RuntimeError::new_type_error(
                                &format!(
                                    "unsupported operand type: {} '{}'",
                                    operator,
                                    right_value.type_name()
                                ),
                                self.span.clone(),
                            ))
                        }
                    }
                    UnaryOperator::Bang => Ok(Value::Boolean(!bool::from(right_value))),
                }
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = left.evaluate(ctx)?;
                let right_value = right.evaluate(ctx)?;

                match operator {
                    BinaryOperator::Comma => Ok(right_value),
                    BinaryOperator::Minus
                    | BinaryOperator::Plus
                    | BinaryOperator::Slash
                    | BinaryOperator::Star => {
                        try_arithmetic(left_value, right_value, operator.clone(), self.span.clone())
                    }
                    BinaryOperator::Less
                    | BinaryOperator::LessEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEqual => {
                        try_compare(left_value, right_value, operator.clone(), self.span.clone())
                    }
                    BinaryOperator::EqualEqual => Ok(Value::Boolean(left_value == right_value)),
                    BinaryOperator::BangEqual => Ok(Value::Boolean(!(left_value == right_value))),
                }
            }
            ExprKind::Ternary {
                left,
                middle,
                right,
            } => {
                let left_value = left.evaluate(ctx)?;

                if bool::from(left_value) {
                    middle.evaluate(ctx)
                } else {
                    right.evaluate(ctx)
                }
            }
            ExprKind::Variable { name } => match ctx.env.borrow().get(name) {
                None => Err(RuntimeError::new_name_error(
                    &format!("undefined variable '{}'", name),
                    self.span.clone(),
                )),
                Some(Value::Unassigned) => Err(RuntimeError::new_unassigned_error(
                    &format!("unassigned variable '{name}'"),
                    self.span.clone(),
                )),
                Some(value) => Ok(value.clone()),
            },
            ExprKind::Assign { name, expr } => {
                let value = expr.evaluate(ctx)?;
                ctx.env
                    .borrow_mut()
                    .assign(name, value.clone())
                    .map_err(|_| {
                        RuntimeError::new_name_error(
                            &format!("undefined variable '{}'", name),
                            self.span.clone(),
                        )
                    })
            }
        }
    }
}

impl Evaluate for Statement {
    type Value = Option<Value>;

    fn evaluate(&self, ctx: &mut InterpreterCtx) -> Result<Self::Value, RuntimeError> {
        match &self.kind {
            StatementKind::Expression { expr, closed } => {
                let val = expr.evaluate(ctx)?;
                if !closed {
                    return Ok(Some(val));
                }
            }
            StatementKind::Print(expr) => {
                let value = expr.evaluate(ctx)?;
                println!("{}", value);
            }
            StatementKind::Var { ident, initializer } => {
                let value = if let Some(expr) = initializer {
                    expr.evaluate(ctx)?
                } else {
                    Value::Unassigned
                };
                ctx.env.borrow_mut().define(&ident.name, value.clone());
            }
            StatementKind::Block(statements) => {
                let scope_env = Environment::new_with_enclosing(ctx.env.clone());
                let mut scope_ctx = InterpreterCtx { env: scope_env };
                for statement in statements {
                    statement.evaluate(&mut scope_ctx)?;
                }
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if condition.evaluate(ctx)?.into() {
                    then_branch.evaluate(ctx)?;
                } else if let Some(statement) = else_branch {
                    statement.evaluate(ctx)?;
                }
            }
        }

        Ok(None)
    }
}

fn try_arithmetic(
    left: Value,
    right: Value,
    operator: BinaryOperator,
    span: Span,
) -> Result<Value, RuntimeError> {
    match (&left, &right) {
        (Value::Number(inner_left), Value::Number(inner_right)) => {
            let result = match operator {
                BinaryOperator::Minus => inner_left - inner_right,
                BinaryOperator::Plus => inner_left + inner_right,
                BinaryOperator::Slash => inner_left / inner_right,
                BinaryOperator::Star => inner_left * inner_right,
                _ => unreachable!(),
            };
            Ok(Value::Number(result))
        }
        (Value::String(_), _) | (_, Value::String(_)) if operator == BinaryOperator::Plus => {
            let left = match left {
                Value::String(string) => string,
                val => val.to_string(),
            };

            let right = match right {
                Value::String(string) => string,
                val => val.to_string(),
            };

            Ok(Value::String(format!("{}{}", left, right)))
        }
        _ => Err(RuntimeError::new_type_error(
            &format!(
                "unsupported operand type(s): '{}' {} '{}'",
                left.type_name(),
                operator,
                right.type_name()
            ),
            span,
        )),
    }
}

fn try_compare(
    left: Value,
    right: Value,
    operator: BinaryOperator,
    span: Span,
) -> Result<Value, RuntimeError> {
    if let (Value::Number(inner_left), Value::Number(inner_right)) = (&left, &right) {
        let result = match operator {
            BinaryOperator::Less => inner_left < inner_right,
            BinaryOperator::LessEqual => inner_left <= inner_right,
            BinaryOperator::Greater => inner_left > inner_right,
            BinaryOperator::GreaterEqual => inner_left >= inner_right,
            _ => unreachable!(),
        };

        Ok(Value::Boolean(result))
    } else {
        Err(RuntimeError::new_type_error(
            &format!(
                "unsupported operand type(s): '{}' {} '{}'",
                left.type_name(),
                operator,
                right.type_name()
            ),
            span,
        ))
    }
}
