use crate::common::Span;
use crate::expressions::Expr;
use crate::lexer::{Token, TokenKind};
use crate::values::Value;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl TryFrom<Token> for Identifier {
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

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Jump {
    Break,
    Continue,
    Return(Value),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Block(Vec<Statement>),
    Class {
        name: Identifier,
        methods: Vec<Function>,
    },
    Function(Function),
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
        ident: Identifier,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Statement>,
    },
    For {
        initializer: Option<Box<Statement>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Vec<Statement>,
    },
    Jump(Jump),
    Return(Option<Expr>),
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

    pub fn function(
        span: Span,
        name: Identifier,
        parameters: Vec<Identifier>,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            span,
            kind: StatementKind::Function(Function {
                name,
                parameters,
                body,
            }),
        }
    }

    pub fn print(span: Span, expr: Expr) -> Self {
        Self {
            span,
            kind: StatementKind::Print(expr),
        }
    }

    pub fn variable(span: Span, ident: Identifier, initializer: Option<Expr>) -> Self {
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

    pub fn new_for(
        span: Span,
        initializer: Option<Statement>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            span,
            kind: StatementKind::For {
                initializer: initializer.map(Box::new),
                condition,
                increment,
                body,
            },
        }
    }

    pub fn new_break(span: Span) -> Self {
        Self {
            span,
            kind: StatementKind::Jump(Jump::Break),
        }
    }

    pub fn new_continue(span: Span) -> Self {
        Self {
            span,
            kind: StatementKind::Jump(Jump::Continue),
        }
    }

    pub fn new_class(span: Span, name: Identifier, methods: Vec<Function>) -> Self {
        Self {
            span,
            kind: StatementKind::Class { name, methods },
        }
    }
}
