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
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Print(Expr),
    Var {
        ident: Indentifier,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Statement>,
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

    pub fn new_if(
        span: Span,
        condition: Expr,
        then_branch: Statement,
        else_branch: Option<Statement>,
    ) -> Self {
        Self {
            span,
            kind: StatementKind::If {
                condition,
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            },
        }
    }

    pub fn new_while(span: Span, condition: Expr, body: Statement) -> Self {
        Self {
            span,
            kind: StatementKind::While {
                condition,
                body: Box::new(body),
            },
        }
    }
}
