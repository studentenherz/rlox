use std::fmt::Display;
use std::iter::Peekable;

use crate::lexer::tokenize;
use crate::{
    common::Span,
    expressions::*,
    lexer::{Token, TokenKind},
};

#[derive(Debug)]
pub struct ParserError {
    reason: String,
    span: Option<Span>,
}

impl ParserError {
    pub fn new(reason: &str) -> Self {
        Self {
            span: None,
            reason: reason.to_string(),
        }
    }

    pub fn new_with_span(reason: &str, span: Span) -> Self {
        Self {
            reason: reason.to_string(),
            span: Some(span),
        }
    }

    pub fn new_with_token(reason: &str, token: Option<&Token>) -> Self {
        if let Some(t) = token {
            return Self::new_with_span(reason, t.span.clone());
        }

        Self::new(reason)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(Span { line, col, .. }) = self.span {
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
            if let Some(p) = &prev {
                if p.kind == TokenKind::Semicolon {
                    return;
                }
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
            let _ = if Self::is_factor_operator(&t) {
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

            Err(ParserError::new_with_span(
                &format!(
                    "operator {} needs a left operand.",
                    &self.input[t.span.pos..(t.span.pos + t.span.len)],
                ),
                t.span,
            ))
        } else {
            self.comma()
        }
    }

    fn comma(&mut self) -> InternalParseResult {
        let mut expr = self.ternary()?;

        while self.matches(|t| t.kind == TokenKind::Comma) {
            let _token = self.next().unwrap();
            let right = self.ternary()?;
            let span = Span::union(&expr.span, &right.span);
            expr = Expr::binary(span, expr, BinaryOperator::Comma, right);
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
                let span = Span::union(&expr.span, &right.span);
                expr = Expr::ternary(span, expr, middle, right);
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
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.comparison()?;
            let span = Span::union(&expr.span, &right.span);
            expr = Expr::binary(span, expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> InternalParseResult {
        let mut expr = self.term()?;

        while self.matches(Self::is_comparison_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.term()?;
            let span = Span::union(&expr.span, &right.span);
            expr = Expr::binary(span, expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> InternalParseResult {
        let mut expr = self.factor()?;

        while self.matches(Self::is_term_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.factor()?;
            let span = Span::union(&expr.span, &right.span);
            expr = Expr::binary(span, expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> InternalParseResult {
        let mut expr = self.unary()?;

        while self.matches(Self::is_factor_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.unary()?;
            let span = Span::union(&expr.span, &right.span);
            expr = Expr::binary(span, expr, operator, right);
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
                let operator = UnaryOperator::try_from(token.kind).unwrap();
                let right = self.unary()?;
                let span = Span::union(&token.span, &right.span);
                Ok(Expr::unary(span, operator, right))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> InternalParseResult {
        self.consume_whitespace();
        let token = self.next();
        match token {
            Some(Token {
                kind: TokenKind::True,
                span,
            }) => Ok(Expr::literal(span, Literal::True)),
            Some(Token {
                kind: TokenKind::False,
                span,
            }) => Ok(Expr::literal(span, Literal::False)),
            Some(Token {
                kind: TokenKind::Nil,
                span,
            }) => Ok(Expr::literal(span, Literal::Nil)),
            Some(Token {
                kind: TokenKind::String(s),
                span,
            }) => Ok(Expr::literal(span, Literal::String(s))),
            Some(Token {
                kind: TokenKind::Number(n),
                span,
            }) => Ok(Expr::literal(span, Literal::Number(n))),

            Some(
                l_paren @ Token {
                    kind: TokenKind::LeftParen,
                    ..
                },
            ) => {
                let expr = self.expression()?;
                self.consume_whitespace();
                match self.next() {
                    Some(
                        r_paren @ Token {
                            kind: TokenKind::RightParen,
                            ..
                        },
                    ) => {
                        let span = Span::union(&l_paren.span, &r_paren.span);
                        Ok(Expr::grouping(span, expr))
                    }
                    Some(t) => Err(ParserError::new_with_span(
                        "Expected ')' after expression",
                        t.span,
                    )),
                    None => Err(ParserError::new("Unexpected EOF")),
                }
            }
            Some(t) => Err(ParserError::new_with_span("Unexpected token", t.span)),
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
                | TokenKind::Plus
                | TokenKind::Star
                | TokenKind::Slash
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper for tests to provide a blank span
    fn ds() -> Span {
        Span {
            line: 0,
            col: 0,
            pos: 0,
            len: 0,
        }
    }

    #[test]
    fn simple_expression() {
        let actual = Parser::parse("1 + 2").unwrap();
        let expected = Expr::binary(
            ds(),
            Expr::literal(ds(), Literal::Number(1.0)),
            BinaryOperator::Plus,
            Expr::literal(ds(), Literal::Number(2.0)),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn complex_expression() {
        let input = "1 + 2 * 3 / (23 + 43) <= 123 == false";
        let actual = Parser::parse(input).unwrap();
        let expected = Expr::binary(
            ds(),
            Expr::binary(
                ds(),
                Expr::binary(
                    ds(),
                    Expr::literal(ds(), Literal::Number(1.0)),
                    BinaryOperator::Plus,
                    Expr::binary(
                        ds(),
                        Expr::binary(
                            ds(),
                            Expr::literal(ds(), Literal::Number(2.0)),
                            BinaryOperator::Star,
                            Expr::literal(ds(), Literal::Number(3.0)),
                        ),
                        BinaryOperator::Slash,
                        Expr::grouping(
                            ds(),
                            Expr::binary(
                                ds(),
                                Expr::literal(ds(), Literal::Number(23.0)),
                                BinaryOperator::Plus,
                                Expr::literal(ds(), Literal::Number(43.0)),
                            ),
                        ),
                    ),
                ),
                BinaryOperator::LessEqual,
                Expr::literal(ds(), Literal::Number(123.0)),
            ),
            BinaryOperator::EqualEqual,
            Expr::literal(ds(), Literal::False),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn comma_expression() {
        let actual = Parser::parse("1 + 2, 3").unwrap();
        let expected = Expr::binary(
            ds(),
            Expr::binary(
                ds(),
                Expr::literal(ds(), Literal::Number(1.0)),
                BinaryOperator::Plus,
                Expr::literal(ds(), Literal::Number(2.0)),
            ),
            BinaryOperator::Comma,
            Expr::literal(ds(), Literal::Number(3.0)),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn ternary_expression() {
        let actual = Parser::parse("false ? 1, 2 : true ? 3 : 4").unwrap();
        let expected = Expr::ternary(
            ds(),
            Expr::literal(ds(), Literal::False),
            Expr::binary(
                ds(),
                Expr::literal(ds(), Literal::Number(1.0)),
                BinaryOperator::Comma,
                Expr::literal(ds(), Literal::Number(2.0)),
            ),
            Expr::ternary(
                ds(),
                Expr::literal(ds(), Literal::True),
                Expr::literal(ds(), Literal::Number(3.0)),
                Expr::literal(ds(), Literal::Number(4.0)),
            ),
        );

        assert_eq!(actual, expected);
    }
}
