use crate::common::Span;
use crate::expressions::Expr;

#[derive(Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum StatementKind {
    Expression(Expr),
    Print(Expr),
}

impl Statement {
    pub fn expression(span: Span, expr: Expr) -> Self {
        Self {
            span,
            kind: StatementKind::Expression(expr),
        }
    }

    pub fn print(span: Span, expr: Expr) -> Self {
        Self {
            span,
            kind: StatementKind::Print(expr),
        }
    }
}
