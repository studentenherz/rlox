use crate::common::Span;
use crate::lexer::TokenKind;
use std::fmt::{Debug, Display};

// --- Operators & Literal Leaf Types ---

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    BangEqual,
    Comma,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Minus,
    Plus,
    Star,
    Slash,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Minus,
    Bang,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    True,
    False,
    Nil,
}

// --- The Expression ---

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(PartialEq, Clone)]
pub enum ExprKind {
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>,
    },
    Ternary {
        left: Box<Expr>,
        middle: Box<Expr>,
        right: Box<Expr>,
    },
}

// --- Implementation of Traits for Expr ---

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Debug for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                write!(
                    f,
                    "({} {:?} {:?})",
                    operator.to_string(),
                    left.kind,
                    right.kind
                )
            }
            ExprKind::Grouping { expression } => {
                write!(f, "(group {:?})", expression.kind)
            }
            ExprKind::Literal { value } => {
                write!(f, "{:?}", value)
            }
            ExprKind::Unary { operator, right } => {
                write!(f, "({:?} {:?})", operator, right.kind)
            }
            ExprKind::Ternary {
                left,
                middle,
                right,
            } => {
                write!(f, "(?: {:?} {:?} {:?})", left.kind, middle.kind, right.kind)
            }
        }
    }
}

impl Expr {
    pub fn binary(span: Span, left: Expr, operator: BinaryOperator, right: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
        }
    }

    pub fn grouping(span: Span, expression: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Grouping {
                expression: Box::new(expression),
            },
        }
    }

    pub fn literal(span: Span, literal: Literal) -> Self {
        Self {
            span,
            kind: ExprKind::Literal { value: literal },
        }
    }

    pub fn unary(span: Span, operator: UnaryOperator, right: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Unary {
                operator,
                right: Box::new(right),
            },
        }
    }

    pub fn ternary(span: Span, left: Expr, middle: Expr, right: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Ternary {
                left: Box::new(left),
                middle: Box::new(middle),
                right: Box::new(right),
            },
        }
    }
}

// --- Boilerplate implementations (TryFrom & Display for Operators) ---

impl TryFrom<TokenKind> for BinaryOperator {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::BangEqual => Ok(Self::BangEqual),
            TokenKind::Comma => Ok(Self::Comma),
            TokenKind::EqualEqual => Ok(Self::EqualEqual),
            TokenKind::Greater => Ok(Self::Greater),
            TokenKind::GreaterEqual => Ok(Self::GreaterEqual),
            TokenKind::Less => Ok(Self::Less),
            TokenKind::LessEqual => Ok(Self::LessEqual),
            TokenKind::Minus => Ok(Self::Minus),
            TokenKind::Plus => Ok(Self::Plus),
            TokenKind::Star => Ok(Self::Star),
            TokenKind::Slash => Ok(Self::Slash),
            _ => Err(()),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BangEqual => "!=",
            Self::Comma => ",",
            Self::EqualEqual => "==",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Minus => "-",
            Self::Plus => "+",
            Self::Star => "*",
            Self::Slash => "/",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<TokenKind> for UnaryOperator {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Minus => Ok(Self::Minus),
            TokenKind::Bang => Ok(Self::Bang),
            _ => Err(()),
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if matches!(self, Self::Minus) {
                "-"
            } else {
                "!"
            }
        )
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
        }
    }
}
