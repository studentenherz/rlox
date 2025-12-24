use std::fmt::{Debug, Display};

use crate::lexer::Token;

#[derive(Debug)]
pub enum BinaryOperator {
    BangEqual,
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

impl TryFrom<Token> for BinaryOperator {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::BangEqual => Ok(Self::BangEqual),
            Token::EqualEqual => Ok(Self::EqualEqual),
            Token::Greater => Ok(Self::Greater),
            Token::GreaterEqual => Ok(Self::GreaterEqual),
            Token::Less => Ok(Self::Less),
            Token::LessEqual => Ok(Self::LessEqual),
            Token::Minus => Ok(Self::Minus),
            Token::Plus => Ok(Self::Plus),
            Token::Star => Ok(Self::Star),
            Token::Slash => Ok(Self::Slash),
            _ => Err(()),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BangEqual => write!(f, "!="),
            Self::EqualEqual => write!(f, "=="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Minus => write!(f, "-"),
            Self::Plus => write!(f, "+"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
        }
    }
}

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
    Bang,
}

impl TryFrom<Token> for UnaryOperator {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Minus => Ok(Self::Minus),
            Token::Bang => Ok(Self::Bang),
            _ => Err(()),
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minus => write!(f, "-"),
            Self::Bang => write!(f, "!"),
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    Ident(String),
    String(String),
    Number(f64),
    True,
    False,
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => write!(f, "{}", ident),
            Self::String(string) => write!(f, "\"{}\"", string),
            Self::Number(number) => write!(f, "{}", number),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl TryFrom<Token> for Literal {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Ident(string) => Ok(Self::Ident(string)),
            Token::String(string) => Ok(Self::String(string)),
            Token::Number(number) => Ok(Self::Number(number)),
            Token::True => Ok(Self::True),
            Token::False => Ok(Self::False),
            Token::Nil => Ok(Self::Nil),
            _ => Err(()),
        }
    }
}

pub trait Expr: Display {}

pub struct BinaryExpr<'a> {
    left: &'a dyn Expr,
    operator: BinaryOperator,
    right: &'a dyn Expr,
}

impl<'a> Expr for BinaryExpr<'a> {}

pub struct GroupingExpr<'a> {
    inner_expr: &'a dyn Expr,
}

impl<'a> Expr for GroupingExpr<'a> {}

pub struct LiteralExpr {
    literal: Literal,
}

impl Expr for LiteralExpr {}

pub struct UnaryExpr<'a> {
    operator: UnaryOperator,
    right: &'a dyn Expr,
}

impl<'a> Expr for UnaryExpr<'a> {}

impl<'a> Display for BinaryExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.operator, self.left, self.right)
    }
}

impl<'a> Display for GroupingExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(group {})", self.inner_expr)
    }
}

impl Display for LiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.literal)
    }
}

impl<'a> Display for UnaryExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        let expr = BinaryExpr {
            left: &LiteralExpr {
                literal: Literal::Number(1f64),
            },
            operator: BinaryOperator::Minus,
            right: &GroupingExpr {
                inner_expr: &BinaryExpr {
                    left: &LiteralExpr {
                        literal: Literal::Number(2f64),
                    },
                    operator: BinaryOperator::Plus,
                    right: &LiteralExpr {
                        literal: Literal::Number(3f64),
                    },
                },
            },
        };

        assert_eq!(expr.to_string(), "(- 1 (group (+ 2 3)))");
    }
}
