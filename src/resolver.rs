use std::collections::HashMap;
use std::fmt::Display;

use crate::common::Span;
use crate::expressions::{Expr, ExprKind};
use crate::statements::{Identifier, Statement, StatementKind};

#[derive(Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    current_function: FunctionType,
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<ResolverError>,
}

pub struct ResolverError {
    span: Option<Span>,
    reason: String,
}

impl Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            if let Some(span) = &self.span {
                format!("[Line {} Col {}] ", span.line, span.col)
            } else {
                "".to_string()
            },
            self.reason
        )
    }
}

impl ResolverError {
    pub fn new(reason: String, span: Span) -> Self {
        Self {
            reason,
            span: Some(span),
        }
    }
}

pub struct ResolverErrorSet {
    errors: Vec<ResolverError>,
}

impl Display for ResolverErrorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors: Vec<String> = self.errors.iter().map(|err| err.to_string()).collect();
        write!(f, "{}", errors.join("\n"))
    }
}

impl Resolver {
    pub fn resolve(&mut self, statements: &mut Vec<Statement>) -> Result<(), ResolverErrorSet> {
        for stmt in statements {
            self.resolve_statement(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            let errors = std::mem::replace(&mut self.errors, vec![]);
            Err(ResolverErrorSet { errors })
        }
    }

    pub fn new() -> Self {
        Self {
            scopes: vec![],
            current_function: FunctionType::None,
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
                self.errors.push(ResolverError::new(
                    "Already a variable with this name in this scope".to_string(),
                    ident.span.clone(),
                ));
            }
            scope.insert(ident.name.clone(), false);
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

    fn resolve_statement(&mut self, stmt: &mut Statement) {
        match &mut stmt.kind {
            StatementKind::Block(statements) => {
                self.begin_scope();
                for statement in statements.iter_mut() {
                    self.resolve_statement(statement);
                }
                self.end_scope();
            }
            StatementKind::Var { ident, initializer } => {
                self.declare(ident);
                if let Some(init) = initializer {
                    self.resolve_expression(init);
                }
                self.define(ident.name.clone());
            }
            StatementKind::Function {
                name,
                parameters,
                body,
            } => {
                self.declare(name);
                self.define(name.name.clone());

                let enclosing_function = self.current_function.clone();

                self.current_function = FunctionType::Function;
                self.begin_scope();
                for param in parameters {
                    self.declare(param);
                    self.define(param.name.clone());
                }
                for statement in body.iter_mut() {
                    self.resolve_statement(statement);
                }
                self.end_scope();

                self.current_function = enclosing_function;
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
                    self.errors.push(ResolverError::new(
                        "Can't return from top-level code.".to_string(),
                        stmt.span.clone(),
                    ));
                }
                self.resolve_expression(expr);
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
                for stmt in body {
                    self.resolve_statement(stmt);
                }
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
                            self.errors.push(ResolverError::new(
                                "Can't read local variable in its own initializer.".to_string(),
                                expr.span.clone(),
                            ));
                            return;
                        }
                    }
                }

                expr.resolved_depth = self.calculate_depth(name);
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
        }
    }
}
