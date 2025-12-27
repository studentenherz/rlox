use std::fmt::Display;
use std::iter::Peekable;

use crate::lexer::tokenize;
use crate::{lexer::Token, parser::expressions::*};

pub struct Parser<'a> {
    pub iter: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

#[derive(Debug)]
pub struct ParserError {
    reason: String,
}

impl ParserError {
    pub fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

type ParseResult = Result<Box<dyn Expr>, ParserError>;

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str) -> ParseResult {
        let mut parser = Self::new(input);
        parser._parse()
    }

    fn new(input: &'a str) -> Self {
        let boxed_iter: Box<dyn Iterator<Item = Token> + 'a> = Box::new(tokenize(input));
        let mut iter = boxed_iter.peekable();

        while iter.peek() == Some(&Token::Whitespace) {
            iter.next();
        }

        Self { iter }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.iter.peek()
    }

    fn next(&mut self) -> Option<Token> {
        self.iter.next()
    }

    fn consume_whitespace(&mut self) {
        while matches!(self.peek(), Some(Token::Whitespace)) {
            self.next();
        }
    }

    pub fn matches(&mut self, f: impl FnOnce(&Token) -> bool) -> bool {
        self.consume_whitespace();
        self.iter.peek().map_or(false, |t| f(t))
    }

    fn _parse(&mut self) -> ParseResult {
        self.expression()
    }

    fn expression(&mut self) -> ParseResult {
        self.equality()
    }
    fn equality(&mut self) -> ParseResult {
        let mut expr = self.comparison()?;

        self.consume_whitespace();
        while self.matches(|t| matches!(t, Token::BangEqual | Token::EqualEqual)) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.comparison()?;
            expr = Box::new(BinaryExpr::new(expr, operator, right));
        }

        Ok(expr)
    }
    fn comparison(&mut self) -> ParseResult {
        let mut expr = self.term()?;

        while self.matches(|t| {
            matches!(
                t,
                Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual
            )
        }) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.term()?;
            expr = Box::new(BinaryExpr::new(expr, operator, right));
        }

        Ok(expr)
    }
    fn term(&mut self) -> ParseResult {
        let mut expr = self.factor()?;

        while self.matches(|t| matches!(t, Token::Minus | Token::Plus)) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.factor()?;
            expr = Box::new(BinaryExpr::new(expr, operator, right));
        }

        Ok(expr)
    }
    fn factor(&mut self) -> ParseResult {
        let mut expr = self.unary()?;

        while self.matches(|t| matches!(t, Token::Star | Token::Slash)) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.unary()?;
            expr = Box::new(BinaryExpr::new(expr, operator, right));
        }

        Ok(expr)
    }
    fn unary(&mut self) -> ParseResult {
        self.consume_whitespace();
        match self.peek() {
            Some(Token::Bang | Token::Minus) => {
                let token = self.next().unwrap();
                let operator = UnaryOperator::try_from(token).unwrap();
                let right = self.unary()?;
                Ok(Box::new(UnaryExpr::new(operator, right)))
            }
            _ => self.primary(),
        }
    }
    fn primary(&mut self) -> ParseResult {
        self.consume_whitespace();
        match self.next() {
            Some(Token::True) => Ok(Box::new(LiteralExpr::bool_true())),
            Some(Token::False) => Ok(Box::new(LiteralExpr::bool_false())),
            Some(Token::Nil) => Ok(Box::new(LiteralExpr::nil())),
            Some(Token::String(string)) => Ok(Box::new(LiteralExpr::string(&string))),
            Some(Token::Number(number)) => Ok(Box::new(LiteralExpr::number(number))),
            Some(Token::LeftParen) => {
                let expr = self.expression()?;
                match self.next() {
                    Some(Token::RightParen) => Ok(Box::new(GroupingExpr::new(expr))),
                    _ => Err(ParserError::new("Expected ')' after expression")),
                }
            }
            token => Err(ParserError::new(&format!(
                "Unexpected token while parsing while parsing {:?}",
                token
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expression() {
        let actual = Parser::parse("1 + 2 * 3 / (23 + 43) <= 123 == false").unwrap();

        println!("{}", actual);
        println!("{:?}", actual);

        assert_eq!(
            actual.to_string(),
            "(== (<= (+ 1 (/ (* 2 3) (group (+ 23 43)))) 123) false)"
        )
    }
}
