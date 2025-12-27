use crate::lexer::{Token, TokenKind};
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

// --- The Unified Expression Enum ---

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
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
}

// --- Implementation of Traits for Expr ---

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator, left, right)
            }
            Expr::Grouping { expression } => {
                write!(f, "(group {})", expression)
            }
            Expr::Literal { value } => {
                write!(f, "{}", value)
            }
            Expr::Unary { operator, right } => {
                write!(f, "({} {})", operator, right)
            }
        }
    }
}

impl Expr {
    pub fn binary(left: Expr, operator: BinaryOperator, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn grouping(expression: Expr) -> Self {
        Expr::Grouping {
            expression: Box::new(expression),
        }
    }

    pub fn literal(literal: Literal) -> Self {
        Expr::Literal { value: literal }
    }

    pub fn unary(operator: UnaryOperator, right: Expr) -> Self {
        Expr::Unary {
            operator,
            right: Box::new(right),
        }
    }
}

// --- Boilerplate implementations (TryFrom & Display for Operators) ---

impl TryFrom<Token> for BinaryOperator {
    type Error = ();
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.kind {
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

impl TryFrom<Token> for UnaryOperator {
    type Error = ();
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.kind {
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

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_print() {
        let expr = Expr::binary(
            Expr::literal(Literal::Number(1.0)),
            BinaryOperator::Minus,
            Expr::grouping(Expr::binary(
                Expr::literal(Literal::Number(2.0)),
                BinaryOperator::Plus,
                Expr::literal(Literal::Number(3.0)),
            )),
        );

        assert_eq!(expr.to_string(), "(- 1 (group (+ 2 3)))");
    }
}
