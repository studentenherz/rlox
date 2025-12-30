use std::fmt::Display;
use std::iter::Peekable;
use std::usize;

use crate::lexer::tokenize;
use crate::{
    expressions::*,
    lexer::{Token, TokenKind},
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

    pub fn new_with_token(reason: &str, token: Option<&Token>) -> Self {
        if let Some(t) = token {
            return Self::new_located(t.line, t.col, reason);
        }

        Self::new(reason)
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

#[derive(Debug)]
pub struct ParserErrorSet {
    errors: Vec<ParserError>,
}

impl Display for ParserErrorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors: Vec<String> = self.errors.iter().map(|err| err.to_string()).collect();
        write!(f, "{}", errors.join("\n"))
    }
}

type InternalParseResult = Result<Expr, ParserError>;
type ParseResult = Result<Expr, ParserErrorSet>;

pub struct Parser<'a> {
    input: &'a str,
    iter: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str) -> ParseResult {
        let mut parser = Self::new(input);

        match parser._parse() {
            Ok(expr) => Ok(expr),
            Err(err) => {
                let mut errors = vec![err];
                while parser.peek().is_some() {
                    parser.synchronize();
                    if let Err(err) = parser._parse() {
                        errors.push(err);
                    }
                }

                Err(ParserErrorSet { errors })
            }
        }
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

    fn synchronize(&mut self) {
        let mut prev = self.next();

        while let Some(t) = self.peek() {
            if prev.unwrap().kind == TokenKind::Semicolon {
                return;
            }

            if matches!(
                t.kind,
                TokenKind::If
                    | TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return
            ) {
                return;
            }

            prev = self.next();
        }
    }

    pub fn matches(&mut self, f: impl FnOnce(&Token) -> bool) -> bool {
        self.consume_whitespace();
        self.peek().map_or(false, |t| f(t))
    }

    fn _parse(&mut self) -> InternalParseResult {
        self.expression()
    }

    fn expression(&mut self) -> InternalParseResult {
        if self.peek().map_or(false, Self::is_binary_operator) {
            let t = self.next().unwrap();
            let _discarded_right_operand = if Self::is_factor_operator(&t) {
                self.unary()
            } else if Self::is_term_operator(&t) {
                self.factor()
            } else if Self::is_comparison_operator(&t) {
                self.term()
            } else if Self::is_equality_operator(&t) {
                self.comparison()
            } else {
                self.ternary()
            };

            Err(ParserError::new_located(
                t.line,
                t.col,
                &format!(
                    "operator {} needs a left operand.",
                    &self.input[t.pos..(t.pos + t.len)],
                ),
            ))
        } else {
            self.comma()
        }
    }

    fn comma(&mut self) -> InternalParseResult {
        let mut expr = self.ternary()?;

        while self.matches(|t| t.kind == TokenKind::Comma) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.ternary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> InternalParseResult {
        let mut expr = self.equality()?;

        if self.matches(|t| t.kind == TokenKind::Question) {
            self.next();
            let middle = self.expression()?;
            if self.matches(|t| t.kind == TokenKind::Colon) {
                self.next();
                let right = self.ternary()?;
                expr = Expr::ternary(expr, middle, right);
            } else {
                return Err(ParserError::new_with_token("Expected ':'", self.peek()));
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> InternalParseResult {
        let mut expr = self.comparison()?;

        while self.matches(Self::is_equality_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> InternalParseResult {
        let mut expr = self.term()?;

        while self.matches(Self::is_comparison_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> InternalParseResult {
        let mut expr = self.factor()?;

        while self.matches(Self::is_term_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> InternalParseResult {
        let mut expr = self.unary()?;

        while self.matches(Self::is_factor_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token).unwrap();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> InternalParseResult {
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

    fn primary(&mut self) -> InternalParseResult {
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

    fn is_factor_operator(token: &Token) -> bool {
        matches!(token.kind, TokenKind::Star | TokenKind::Slash)
    }

    fn is_term_operator(token: &Token) -> bool {
        matches!(token.kind, TokenKind::Minus | TokenKind::Plus)
    }

    fn is_comparison_operator(token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual
        )
    }

    fn is_equality_operator(token: &Token) -> bool {
        matches!(token.kind, TokenKind::BangEqual | TokenKind::EqualEqual)
    }

    fn is_binary_operator(token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::BangEqual
                | TokenKind::Comma
                | TokenKind::EqualEqual
                | TokenKind::Greater
                | TokenKind::GreaterEqual
                | TokenKind::Less
                | TokenKind::LessEqual
                | TokenKind::Minus
                | TokenKind::Plus
                | TokenKind::Star
                | TokenKind::Slash
        )
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

    #[test]
    fn comma_expression() {
        let actual = Parser::parse("1 + 2, 3").unwrap();
        let expected = Expr::binary(
            Expr::binary(
                Expr::literal(Literal::Number(1.0)),
                BinaryOperator::Plus,
                Expr::literal(Literal::Number(2.0)),
            ),
            BinaryOperator::Comma,
            Expr::literal(Literal::Number(3f64)),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn ternary_expression() {
        let actual = Parser::parse("false ? 1, 2 : true ? 3 : 4").unwrap();
        let expected = Expr::ternary(
            Expr::literal(Literal::False),
            Expr::binary(
                Expr::literal(Literal::Number(1f64)),
                BinaryOperator::Comma,
                Expr::literal(Literal::Number(2f64)),
            ),
            Expr::ternary(
                Expr::literal(Literal::True),
                Expr::literal(Literal::Number(3f64)),
                Expr::literal(Literal::Number(4f64)),
            ),
        );

        assert_eq!(actual, expected);
    }
}
