use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::builtins::builtins;
use crate::classes::LoxClass;
use crate::common::Span;
use crate::environments::{Environment, SharedEnv};
use crate::errors::LoxError;
use crate::functions::LoxFunction;
use crate::statements::{Function, Jump, Statement, StatementKind};
use crate::values::LoxCallable;
use crate::{expressions::*, values::Value};

pub struct InterpreterCtx {
    globals: SharedEnv,
    env: SharedEnv,
    pub jump: Rc<RefCell<Option<Jump>>>,
}

impl InterpreterCtx {
    pub fn new() -> Self {
        let globals = Environment::new();
        let builtins = builtins();
        for builtin in builtins {
            globals
                .borrow_mut()
                .define(&builtin.name(), Value::function(builtin));
        }

        InterpreterCtx {
            env: globals.clone(),
            globals,
            jump: Rc::new(RefCell::new(None)),
        }
    }

    pub fn new_explicit(
        globals: SharedEnv,
        env: SharedEnv,
        jump: Rc<RefCell<Option<Jump>>>,
    ) -> Self {
        Self {
            globals: globals.clone(),
            env: env.clone(),
            jump: jump.clone(),
        }
    }

    pub fn new_from_enclosing_ctx(ctx: &mut Self) -> Self {
        let scope_env = Environment::new_with_enclosing(ctx.env.clone());
        Self {
            globals: ctx.globals.clone(),
            env: scope_env,
            jump: ctx.jump.clone(),
        }
    }

    pub fn lookup_env(&mut self, depth: Option<usize>) -> SharedEnv {
        if let Some(depth) = depth {
            let mut env = self.env.clone();
            for _ in 0..depth {
                env = unsafe { env.borrow().enclosing.clone().unwrap_unchecked() };
            }

            env
        } else {
            self.globals.clone()
        }
    }
}

pub trait Evaluate {
    type Value;

    fn evaluate(&self, ctx: &mut InterpreterCtx) -> Result<Self::Value, LoxError>;
}

impl Evaluate for Expr {
    type Value = Value;

    fn evaluate(&self, ctx: &mut InterpreterCtx) -> Result<Self::Value, LoxError> {
        match &self.kind {
            ExprKind::Literal { value } => Ok(value.clone()),
            ExprKind::Grouping { expression } => expression.evaluate(ctx),
            ExprKind::Unary { operator, right } => {
                let right_value = right.evaluate(ctx)?;

                match operator {
                    UnaryOperator::Minus => {
                        if let Value::Number(number) = right_value {
                            Ok(Value::Number(-number))
                        } else {
                            Err(LoxError::new_with_span(
                                "Operand must be a number.",
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
                        try_arithmetic(left_value, right_value, &operator, &self.span)
                    }
                    BinaryOperator::Less
                    | BinaryOperator::LessEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEqual => {
                        try_compare(left_value, right_value, &operator, &self.span)
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
            ExprKind::Variable { name } => {
                match ctx.lookup_env(self.resolved_depth).borrow().get(name) {
                    None => Err(LoxError::new_with_span(
                        &format!("Undefined variable '{}'.", name),
                        self.span.clone(),
                    )),
                    Some(Value::Unassigned) => Err(LoxError::new_with_span(
                        &format!("Unassigned variable '{name}'."),
                        self.span.clone(),
                    )),
                    Some(value) => Ok(value.clone()),
                }
            }
            ExprKind::This => {
                let name = "this";
                match ctx.lookup_env(self.resolved_depth).borrow().get(name) {
                    None => Err(LoxError::new_with_span(
                        &format!("Undefined variable '{}'.", name),
                        self.span.clone(),
                    )),
                    Some(Value::Unassigned) => Err(LoxError::new_with_span(
                        &format!("Unassigned variable '{name}'."),
                        self.span.clone(),
                    )),
                    Some(value) => Ok(value.clone()),
                }
            }
            ExprKind::Super { method } => {
                let name = "super";
                let superclass = match ctx.lookup_env(self.resolved_depth).borrow().get(name) {
                    None => Err(LoxError::new_with_span(
                        &format!("Undefined variable '{}'.", name),
                        self.span.clone(),
                    )),
                    Some(Value::Unassigned) => Err(LoxError::new_with_span(
                        &format!("Unassigned variable '{name}'."),
                        self.span.clone(),
                    )),
                    Some(Value::Callable(LoxCallable::Class(class))) => Ok(class.clone()),
                    _ => unreachable!(),
                }?;

                let name = "this";
                let object = match ctx
                    .lookup_env(self.resolved_depth.map(|v| v - 1))
                    .borrow()
                    .get(name)
                {
                    None => Err(LoxError::new_with_span(
                        &format!("Undefined variable '{}'.", name),
                        self.span.clone(),
                    )),
                    Some(Value::Unassigned) => Err(LoxError::new_with_span(
                        &format!("Unassigned variable '{name}'."),
                        self.span.clone(),
                    )),
                    Some(value) => Ok(value.clone()),
                }?;

                if let Some(Value::Callable(LoxCallable::Function(method))) =
                    superclass.get_method(&method.name)
                {
                    Ok(Value::function(method.bind(object)))
                } else {
                    Err(LoxError::new_with_span(
                        &format!("Undefined property '{}'.", method.name.clone()),
                        self.span.clone(),
                    ))
                }
            }
            ExprKind::Assign { name, expr } => {
                let value = expr.evaluate(ctx)?;
                ctx.lookup_env(self.resolved_depth)
                    .borrow_mut()
                    .assign(name, value.clone())
                    .map_err(|_| {
                        LoxError::new_with_span(
                            &format!("Undefined variable '{}'.", name),
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
                    LogicalOperator::And if !is_left_truthy => Ok(left),
                    _ => right.evaluate(ctx),
                }
            }
            ExprKind::Call { callee, arguments } => {
                let callee = callee.evaluate(ctx)?;

                let mut args = Vec::<Value>::new();
                for arg in arguments {
                    args.push(arg.evaluate(ctx)?);
                }

                let callable = callee.try_get_callable().map_err(|_| {
                    LoxError::new_with_span(
                        "Can only call functions and classes.",
                        self.span.clone(),
                    )
                })?;

                if let Some(arity) = callable.arity() {
                    let argc = args.len();
                    if argc != arity {
                        return Err(LoxError::new_with_span(
                            &format!("Expected {arity} arguments but got {argc}.",),
                            self.span.clone(),
                        ));
                    }
                }

                callable.call(ctx, args)
            }
            ExprKind::Get { object, name } => {
                let object = object.evaluate(ctx)?;
                object.try_get_property(name)
            }
            ExprKind::Set {
                object,
                name,
                value,
            } => {
                let object = object.evaluate(ctx)?;
                let value = value.evaluate(ctx)?;
                object.try_set_property(name, &value)
            }
            ExprKind::Eof => Ok(Value::Nil),
        }
    }
}

impl Evaluate for Statement {
    type Value = Option<Value>;

    fn evaluate(&self, ctx: &mut InterpreterCtx) -> Result<Self::Value, LoxError> {
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
                let mut scope_ctx = InterpreterCtx::new_from_enclosing_ctx(ctx);
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
                    if let Some(jump) = ctx.jump.borrow().clone() {
                        match jump {
                            Jump::Break => {
                                *ctx.jump.borrow_mut() = None;
                                break;
                            }
                            Jump::Return(_) => break,
                            Jump::Continue => {
                                *ctx.jump.borrow_mut() = None;
                            }
                        }
                    }
                }
            }
            StatementKind::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                let mut scope_ctx = InterpreterCtx::new_from_enclosing_ctx(ctx);
                if let Some(initializer) = initializer {
                    initializer.evaluate(&mut scope_ctx)?;
                }

                let condition = condition
                    .clone()
                    .unwrap_or(Expr::literal(Span::dumb(), Literal::True));

                while bool::from(&condition.evaluate(&mut scope_ctx)?) {
                    body.evaluate(&mut scope_ctx)?;
                    if let Some(jump) = scope_ctx.jump.borrow().clone() {
                        match jump {
                            Jump::Break => {
                                *ctx.jump.borrow_mut() = None;
                                break;
                            }
                            Jump::Return(_) => break,
                            Jump::Continue => {
                                *ctx.jump.borrow_mut() = None;
                            }
                        }
                    }

                    if let Some(increment) = increment {
                        increment.evaluate(&mut scope_ctx)?;
                    }
                }
            }
            StatementKind::Jump(jump) => *ctx.jump.borrow_mut() = Some(jump.clone()),
            StatementKind::Function(Function {
                name,
                parameters,
                body,
            }) => {
                let function = LoxFunction::new_user_defined(
                    name.clone(),
                    parameters.clone(),
                    body.to_vec(),
                    ctx.env.clone(),
                    false,
                );
                ctx.env
                    .borrow_mut()
                    .define(&name.name, Value::function(function));
            }
            StatementKind::Return(expr) => {
                let value = if let Some(expr) = expr {
                    expr.evaluate(ctx)?
                } else {
                    Value::from_literal(&Literal::Nil)
                };

                *ctx.jump.borrow_mut() = Some(Jump::Return(value));
            }
            StatementKind::Class {
                name,
                superclass,
                methods: methods_expr,
            } => {
                let superclass = if let Some(expr) = superclass {
                    let value = expr.evaluate(ctx)?;
                    match value {
                        Value::Callable(crate::values::LoxCallable::Class(class)) => {
                            Some(class.clone())
                        }
                        _ => {
                            return Err(LoxError::new_with_span(
                                "Superclass must be a class.",
                                self.span.clone(),
                            ));
                        }
                    }
                } else {
                    None
                };

                ctx.env.borrow_mut().define(&name.name, Value::Unassigned);

                let env = if let Some(superclass) = &superclass {
                    let env = Environment::new_with_enclosing(ctx.env.clone());
                    env.borrow_mut().define(
                        "super",
                        Value::Callable(LoxCallable::Class(superclass.clone())),
                    );
                    env
                } else {
                    ctx.env.clone()
                };

                let mut methods = HashMap::new();
                for Function {
                    name,
                    parameters,
                    body,
                } in methods_expr
                {
                    let function = Value::function(LoxFunction::new_user_defined(
                        name.clone(),
                        parameters.clone(),
                        body.clone(),
                        env.clone(),
                        name.name == "init",
                    ));
                    methods.insert(name.name.clone(), function);
                }

                let class = LoxClass::new(&name.name, superclass, methods);
                ctx.env
                    .borrow_mut()
                    .assign(&name.name, Value::class(class))
                    .map_err(|_| {
                        LoxError::new_with_span(
                            &format!("undefined variable '{}'", name.name),
                            self.span.clone(),
                        )
                    })?;
            }
        }

        Ok(None)
    }
}

fn try_arithmetic(
    left: Value,
    right: Value,
    operator: &BinaryOperator,
    span: &Span,
) -> Result<Value, LoxError> {
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
        (Value::String(_), _) | (_, Value::String(_)) if *operator == BinaryOperator::Plus => {
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
        _ => {
            if *operator == BinaryOperator::Plus {
                Err(LoxError::new_with_span(
                    "Operands must be two numbers or two strings.",
                    span.clone(),
                ))
            } else {
                Err(LoxError::new_with_span(
                    "Operands must be numbers.",
                    span.clone(),
                ))
            }
        }
    }
}

fn try_compare(
    left: Value,
    right: Value,
    operator: &BinaryOperator,
    span: &Span,
) -> Result<Value, LoxError> {
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
        Err(LoxError::new_with_span(
            "Operands must be numbers.",
            span.clone(),
        ))
    }
}
