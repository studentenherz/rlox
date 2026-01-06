use crate::common::Span;
use crate::expressions::Expr;
use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Indentifier {
    pub name: String,
    pub span: Span,
}

impl TryFrom<Token> for Indentifier {
    type Error = ();
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Ident(name) => Ok(Self {
                name,
                span: value.span,
            }),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum StatementKind {
    Block(Vec<Statement>),
    Expression {
        expr: Expr,
        closed: bool,
    },
    Print(Expr),
    Var {
        ident: Indentifier,
        initializer: Option<Expr>,
    },
}

impl Statement {
    pub fn block(span: Span, statements: Vec<Statement>) -> Self {
        Self {
            span,
            kind: StatementKind::Block(statements),
        }
    }

    pub fn expression(span: Span, expr: Expr, closed: bool) -> Self {
        Self {
            span,
            kind: StatementKind::Expression { expr, closed },
        }
    }

    pub fn print(span: Span, expr: Expr) -> Self {
        Self {
            span,
            kind: StatementKind::Print(expr),
        }
    }

    pub fn variable(span: Span, ident: Indentifier, initializer: Option<Expr>) -> Self {
        Self {
            span,
            kind: StatementKind::Var { ident, initializer },
        }
    }
}
