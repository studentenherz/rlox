use std::collections::HashMap;

use crate::errors::{LoxError, LoxErrorSet};
use crate::expressions::{Expr, ExprKind};
use crate::statements::{Function, Identifier, Statement, StatementKind};

#[derive(Clone, PartialEq)]
enum FunctionType {
    Function,
    Initializer,
    Method,
    None,
}

#[derive(Clone, PartialEq)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver {
    current_function: FunctionType,
    current_class: ClassType,
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<LoxError>,
}

impl Resolver {
    pub fn resolve(&mut self, statements: &mut Vec<Statement>) -> Result<(), LoxErrorSet> {
        for stmt in statements {
            self.resolve_statement(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            let errors = std::mem::replace(&mut self.errors, vec![]);
            Err(errors)
        }
    }

    pub fn new() -> Self {
        Self {
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
            errors: vec![],
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, ident: &Identifier) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&ident.name) {
                self.errors.push(LoxError::new_with_span(
                    "Already a variable with this name in this scope.",
                    ident.span.clone(),
                ));
            } else {
                scope.insert(ident.name.clone(), false);
            }
        }
    }

    fn define(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, true);
        }
    }

    fn calculate_depth(&mut self, name: &str) -> Option<usize> {
        let mut depth = 0;
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return Some(depth);
            }
            depth += 1;
        }
        None
    }

    fn resolve_function(&mut self, function: &mut Function, fn_type: FunctionType) {
        let enclosing_function = self.current_function.clone();
        self.current_function = fn_type;
        self.begin_scope();
        for param in &function.parameters {
            self.declare(param);
            self.define(param.name.clone());
        }
        for statement in function.body.iter_mut() {
            self.resolve_statement(statement);
        }
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn resolve_statement(&mut self, stmt: &mut Statement) {
        match &mut stmt.kind {
            StatementKind::Block(statements) => {
                self.begin_scope();
                for statement in statements.iter_mut() {
                    self.resolve_statement(statement);
                }
                self.end_scope();
            }
            StatementKind::Class {
                name,
                superclass,
                methods,
            } => {
                let prev_class_type = self.current_class.clone();
                self.current_class = ClassType::Class;
                self.declare(&name);
                self.define(name.name.clone());

                if let Some(Expr {
                    kind:
                        ExprKind::Variable {
                            name: superclass_name,
                        },
                    span,
                    ..
                }) = superclass
                {
                    if name.name == *superclass_name {
                        self.errors.push(LoxError::new_with_span(
                            "A class can't inherit from itself.",
                            span.clone(),
                        ))
                    }
                }

                if let Some(superclass) = superclass {
                    self.current_class = ClassType::Subclass;
                    self.resolve_expression(superclass);
                    self.begin_scope();
                    unsafe {
                        self.scopes
                            .last_mut()
                            .unwrap_unchecked()
                            .insert("super".to_string(), true);
                    }
                }

                self.begin_scope();
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert("this".to_string(), true);
                }

                for method in methods {
                    let fn_type = if method.name.name == "init" {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };
                    self.resolve_function(method, fn_type);
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = prev_class_type;
            }
            StatementKind::Var { ident, initializer } => {
                self.declare(ident);
                if let Some(init) = initializer {
                    self.resolve_expression(init);
                }
                self.define(ident.name.clone());
            }
            StatementKind::Function(function) => {
                self.declare(&function.name);
                self.define(function.name.name.clone());

                self.resolve_function(function, FunctionType::Function);
            }
            StatementKind::Expression { expr, closed: _ } => {
                self.resolve_expression(expr);
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            StatementKind::Print(expr) => {
                self.resolve_expression(expr);
            }
            StatementKind::Return(expr) => {
                if self.current_function == FunctionType::None {
                    self.errors.push(LoxError::new_with_span(
                        "Can't return from top-level code.",
                        stmt.span.clone(),
                    ));
                }

                if let Some(expr) = expr {
                    if self.current_function == FunctionType::Initializer {
                        self.errors.push(LoxError::new_with_span(
                            "Can't return a value from an initializer.",
                            stmt.span.clone(),
                        ));
                    }

                    self.resolve_expression(expr);
                }
            }
            StatementKind::While { condition, body } => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
            StatementKind::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                self.begin_scope();
                if let Some(initializer) = initializer {
                    self.resolve_statement(initializer);
                }
                if let Some(condition) = condition {
                    self.resolve_expression(condition);
                }
                if let Some(increment) = increment {
                    self.resolve_expression(increment);
                }

                self.resolve_statement(body);
                self.end_scope();
            }
            StatementKind::Jump(_) => {}
        }
    }

    fn resolve_expression<'a>(&mut self, expr: &'a mut Expr) {
        match &mut expr.kind {
            ExprKind::Variable { name } => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(defined) = scope.get(name) {
                        if !*defined {
                            self.errors.push(LoxError::new_with_span(
                                "Can't read local variable in its own initializer.",
                                expr.span.clone(),
                            ));
                            return;
                        }
                    }
                }

                expr.resolved_depth = self.calculate_depth(name);
            }
            ExprKind::Super { .. } => {
                if self.current_class == ClassType::None {
                    self.errors.push(LoxError::new_with_span(
                        "Can't use 'super' outside of a class.",
                        expr.span.clone(),
                    ));
                } else if self.current_class != ClassType::Subclass {
                    self.errors.push(LoxError::new_with_span(
                        "Can't use 'super' in a class with no superclass.",
                        expr.span.clone(),
                    ));
                }

                expr.resolved_depth = self.calculate_depth("super");
            }
            ExprKind::This => {
                if self.current_class == ClassType::None {
                    self.errors.push(LoxError::new_with_span(
                        "Can't use 'this' outside of a class.",
                        expr.span.clone(),
                    ));
                }

                expr.resolved_depth = self.calculate_depth("this")
            }
            ExprKind::Assign {
                name,
                expr: assign_expr,
            } => {
                self.resolve_expression(assign_expr);
                expr.resolved_depth = self.calculate_depth(name)
            }
            ExprKind::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            ExprKind::Get { object, .. } => {
                self.resolve_expression(object);
            }
            ExprKind::Call { callee, arguments } => {
                self.resolve_expression(callee);

                for arg in arguments {
                    self.resolve_expression(arg);
                }
            }
            ExprKind::Grouping { expression } => {
                self.resolve_expression(expression);
            }
            ExprKind::Literal { value: _ } => {}
            ExprKind::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            ExprKind::Set { object, value, .. } => {
                self.resolve_expression(object);
                self.resolve_expression(value);
            }
            ExprKind::Unary { operator: _, right } => {
                self.resolve_expression(right);
            }
            ExprKind::Ternary {
                left,
                middle,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(middle);
                self.resolve_expression(right);
            }
            ExprKind::Eof => {}
        }
    }
}
