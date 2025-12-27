use std::fmt::Display;
use std::iter::Peekable;
use std::usize;

use crate::lexer::tokenize;
use crate::{
    lexer::{Token, TokenKind},
    parser::expressions::*,
};

#[derive(Debug)]
struct Location {
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct ParserError {
    loc: Option<Location>,
    reason: String,
}

impl ParserError {
    pub fn new(reason: &str) -> Self {
        Self {
            loc: None,
            reason: reason.to_string(),
        }
    }

    pub fn new_located(line: usize, col: usize, reason: &str) -> Self {
        Self {
            loc: Some(Location { line, col }),
            reason: reason.to_string(),
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(Location { line, col }) = self.loc {
            write!(
                f,
                "Parsing Error: [Line {}, Col {}] {}",
                line, col, self.reason
            )
        } else {
            write!(f, "Parsing Error: {}", self.reason)
        }
    }
}

pub struct Parser<'a> {
    input: &'a str,
    iter: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

type ParseResult = Result<Expr, ParserError>;

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str) -> ParseResult {
        let mut parser = Self::new(input);
        parser._parse()
    }

    fn new(input: &'a str) -> Self {
        let boxed_iter: Box<dyn Iterator<Item = Token> + 'a> = Box::new(tokenize(input));
        let iter = boxed_iter.peekable();

        let mut parser = Self { input, iter };
        parser.consume_whitespace();
        parser
    }

    fn peek(&mut self) -> Option<&Token> {
        self.iter.peek()
    }

    fn next(&mut self) -> Option<Token> {
        self.iter.next()
    }

    fn consume_whitespace(&mut self) {
        while let Some(t) = self.peek() {
            if t.kind != TokenKind::Whitespace {
                break;
            }
            self.next();
        }
    }

    pub fn matches(&mut self, f: impl FnOnce(&Token) -> bool) -> bool {
        self.consume_whitespace();
        self.peek().map_or(false, |t| f(t))
    }

    fn _parse(&mut self) -> ParseResult {
        self.expression()
    }

    fn expression(&mut self) -> ParseResult {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult {
        let mut expr = self.comparison()?;

        while self.matches(|t| matches!(t.kind, TokenKind::BangEqual | TokenKind::EqualEqual)) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult {
        let mut expr = self.term()?;

        while self.matches(|t| {
            matches!(
                t.kind,
                TokenKind::Greater
                    | TokenKind::GreaterEqual
                    | TokenKind::Less
                    | TokenKind::LessEqual
            )
        }) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult {
        let mut expr = self.factor()?;

        while self.matches(|t| matches!(t.kind, TokenKind::Minus | TokenKind::Plus)) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult {
        let mut expr = self.unary()?;

        while self.matches(|t| matches!(t.kind, TokenKind::Star | TokenKind::Slash)) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult {
        self.consume_whitespace();
        match self.peek() {
            Some(Token {
                kind: TokenKind::Bang | TokenKind::Minus,
                ..
            }) => {
                let token = self.next().unwrap();
                let operator = UnaryOperator::try_from(token).unwrap();
                let right = self.unary()?;
                Ok(Expr::unary(operator, right))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> ParseResult {
        self.consume_whitespace();
        match self.next() {
            Some(Token {
                kind: TokenKind::True,
                ..
            }) => Ok(Expr::literal(Literal::True)),
            Some(Token {
                kind: TokenKind::False,
                ..
            }) => Ok(Expr::literal(Literal::False)),
            Some(Token {
                kind: TokenKind::Nil,
                ..
            }) => Ok(Expr::literal(Literal::Nil)),

            Some(Token {
                kind: TokenKind::String(string),
                ..
            }) => Ok(Expr::literal(Literal::String(string))),
            Some(Token {
                kind: TokenKind::Number(number),
                ..
            }) => Ok(Expr::literal(Literal::Number(number))),

            Some(Token {
                kind: TokenKind::LeftParen,
                ..
            }) => {
                let expr = self.expression()?;

                self.consume_whitespace();
                match self.next() {
                    Some(Token {
                        kind: TokenKind::RightParen,
                        ..
                    }) => Ok(Expr::grouping(expr)),
                    Some(t) => Err(ParserError::new_located(
                        t.line,
                        t.col,
                        &format!(
                            "Expected ')' after expression, found {}",
                            &self.input[t.pos..(t.pos + t.len)]
                        ),
                    )),
                    None => Err(ParserError::new("Unexpected EOF")),
                }
            }

            Some(t) => Err(ParserError::new_located(
                t.line,
                t.col,
                &format!("Unexpected token: {}", &self.input[t.pos..(t.pos + t.len)]),
            )),

            None => Err(ParserError::new("Unexpected end of input")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expression() {
        let actual = Parser::parse("1 + 2").unwrap();
        let expected = Expr::binary(
            Expr::literal(Literal::Number(1.0)),
            BinaryOperator::Plus,
            Expr::literal(Literal::Number(2.0)),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn complex_expression() {
        let input = "1 + 2 * 3 / (23 + 43) <= 123 == false";
        let actual = Parser::parse(input).unwrap();
        let expected = Expr::binary(
            // Left side of the '=='
            Expr::binary(
                // Left side of the '<='
                Expr::binary(
                    Expr::literal(Literal::Number(1.0)),
                    BinaryOperator::Plus,
                    // 2 * 3 / (23 + 43)
                    Expr::binary(
                        Expr::binary(
                            Expr::literal(Literal::Number(2.0)),
                            BinaryOperator::Star,
                            Expr::literal(Literal::Number(3.0)),
                        ),
                        BinaryOperator::Slash,
                        Expr::grouping(Expr::binary(
                            Expr::literal(Literal::Number(23.0)),
                            BinaryOperator::Plus,
                            Expr::literal(Literal::Number(43.0)),
                        )),
                    ),
                ),
                BinaryOperator::LessEqual,
                Expr::literal(Literal::Number(123.0)),
            ),
            BinaryOperator::EqualEqual,
            // Right side of the '=='
            Expr::literal(Literal::False),
        );

        assert_eq!(actual, expected);
        assert_eq!(
            actual.to_string(),
            "(== (<= (+ 1 (/ (* 2 3) (group (+ 23 43)))) 123) false)"
        );
    }
}
