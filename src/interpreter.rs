use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::common::Span;
use crate::environments::{Environment, SharedEnv};
use crate::statements::{Jump, Statement, StatementKind};
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
    pub jump: Rc<RefCell<Option<Jump>>>,
}

impl InterpreterCtx {
    pub fn new() -> Self {
        InterpreterCtx {
            env: Environment::new(),
            jump: Rc::new(RefCell::new(None)),
        }
    }

    pub fn new_from_ctx(ctx: &mut Self) -> Self {
        let scope_env = Environment::new_with_enclosing(ctx.env.clone());
        Self {
            env: scope_env,
            jump: ctx.jump.clone(),
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
                    UnaryOperator::Bang => Ok(Value::Boolean(!bool::from(&right_value))),
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

                if bool::from(&left_value) {
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
            ExprKind::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(ctx)?;
                let is_left_truthy = bool::from(&left);

                match operator {
                    LogicalOperator::Or if is_left_truthy => Ok(left),
                    LogicalOperator::And if is_left_truthy => Ok(left),
                    _ => right.evaluate(ctx),
                }
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
                let mut scope_ctx = InterpreterCtx::new_from_ctx(ctx);
                for statement in statements {
                    if scope_ctx.jump.borrow().is_some() {
                        break;
                    }
                    statement.evaluate(&mut scope_ctx)?;
                }
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if bool::from(&condition.evaluate(ctx)?) {
                    then_branch.evaluate(ctx)?;
                } else if let Some(statement) = else_branch {
                    statement.evaluate(ctx)?;
                }
            }
            StatementKind::While { condition, body } => {
                while bool::from(&condition.evaluate(ctx)?) {
                    body.evaluate(ctx)?;
                    if matches!(*ctx.jump.borrow(), Some(Jump::Break)) {
                        break;
                    }
                    *ctx.jump.borrow_mut() = None;
                }
            }
            StatementKind::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                let mut scope_ctx = InterpreterCtx::new_from_ctx(ctx);
                if let Some(initializer) = initializer {
                    initializer.evaluate(&mut scope_ctx)?;
                }

                let condition = condition
                    .clone()
                    .unwrap_or(Expr::literal(Span::dumb(), Literal::True));

                while bool::from(&condition.evaluate(&mut scope_ctx)?) {
                    body.evaluate(&mut scope_ctx)?;
                    if matches!(*scope_ctx.jump.borrow(), Some(Jump::Break)) {
                        break;
                    }
                    *scope_ctx.jump.borrow_mut() = None;

                    if let Some(increment) = increment {
                        increment.evaluate(&mut scope_ctx)?;
                    }
                }
            }
            StatementKind::Jump(jump) => *ctx.jump.borrow_mut() = Some(jump.clone()),
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
