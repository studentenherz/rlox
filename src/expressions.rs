use crate::common::Span;
use crate::lexer::TokenKind;
use crate::statements::Identifier;
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
pub enum LogicalOperator {
    And,
    Or,
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
    pub resolved_depth: Option<usize>,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Identifier,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Set {
        object: Box<Expr>,
        name: Identifier,
        value: Box<Expr>,
    },
    Ternary {
        left: Box<Expr>,
        middle: Box<Expr>,
        right: Box<Expr>,
    },
    This,
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>,
    },
    Variable {
        name: String,
    },
    Assign {
        name: String,
        expr: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: LogicalOperator,
        right: Box<Expr>,
    },
}

// --- Implementation of Traits for Expr ---

impl Expr {
    pub fn binary(left: Expr, operator: BinaryOperator, right: Expr) -> Self {
        let span = Span::union(&left.span, &right.span);
        Self {
            span,
            kind: ExprKind::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
            resolved_depth: None,
        }
    }

    pub fn get(object: Expr, name: Identifier) -> Self {
        Self {
            span: Span::union(&object.span, &name.span),
            kind: ExprKind::Get {
                object: Box::new(object),
                name,
            },
            resolved_depth: None,
        }
    }

    pub fn grouping(span: Span, expression: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Grouping {
                expression: Box::new(expression),
            },
            resolved_depth: None,
        }
    }

    pub fn literal(span: Span, literal: Literal) -> Self {
        Self {
            span,
            kind: ExprKind::Literal { value: literal },
            resolved_depth: None,
        }
    }

    pub fn unary(span: Span, operator: UnaryOperator, right: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Unary {
                operator,
                right: Box::new(right),
            },
            resolved_depth: None,
        }
    }

    pub fn ternary(left: Expr, middle: Expr, right: Expr) -> Self {
        let span = Span::union(&left.span, &right.span);
        Self {
            span,
            kind: ExprKind::Ternary {
                left: Box::new(left),
                middle: Box::new(middle),
                right: Box::new(right),
            },
            resolved_depth: None,
        }
    }

    pub fn variable(span: Span, name: String) -> Self {
        Self {
            span,
            kind: ExprKind::Variable { name },
            resolved_depth: None,
        }
    }

    pub fn assign(span: Span, name: String, expr: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Assign {
                name,
                expr: Box::new(expr),
            },
            resolved_depth: None,
        }
    }

    pub fn logical(left: Expr, operator: LogicalOperator, right: Expr) -> Self {
        let span = Span::union(&left.span, &right.span);
        Self {
            span,
            kind: ExprKind::Logical {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
            resolved_depth: None,
        }
    }

    pub fn call(span: Span, callee: Expr, arguments: Vec<Expr>) -> Self {
        Self {
            span,
            kind: ExprKind::Call {
                callee: Box::new(callee),
                arguments,
            },
            resolved_depth: None,
        }
    }

    pub fn set(span: Span, object: Expr, name: Identifier, value: Expr) -> Self {
        Self {
            span,
            kind: ExprKind::Set {
                object: Box::new(object),
                name,
                value: Box::new(value),
            },
            resolved_depth: None,
        }
    }

    pub fn this(span: Span) -> Self {
        Self {
            span,
            kind: ExprKind::This,
            resolved_depth: None,
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

impl TryFrom<TokenKind> for LogicalOperator {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::And => Ok(LogicalOperator::And),
            TokenKind::Or => Ok(LogicalOperator::Or),
            _ => Err(()),
        }
    }
}

impl Display for LogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogicalOperator::And => "and",
            LogicalOperator::Or => "or",
        };
        write!(f, "{}", s)
    }
}
