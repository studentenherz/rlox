use std::fmt::Display;
use std::iter::Peekable;

use crate::lexer::tokenize;
use crate::{
    common::Span,
    constants::MAXIMUM_ARGUMETN_COUNT,
    expressions::*,
    lexer::{Token, TokenKind},
    statements::*,
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

type ExpressionParseResult = Result<Expr, ParserError>;
type StatementParseResult = Result<Statement, ParserError>;
type ParseResult = Result<Vec<Statement>, ParserErrorSet>;

pub struct Parser<'a> {
    input: &'a str,
    iter: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
    repl: bool,
    inside_loop: bool,
    errors: Vec<ParserError>,
}

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str, repl: bool) -> ParseResult {
        let mut parser = Self::new(input, repl);
        let mut statements = Vec::<Statement>::new();

        while parser.matches(|t| t.kind != TokenKind::Eof) {
            match parser._parse() {
                Ok(stmt) => {
                    statements.push(stmt);
                }
                Err(err) => {
                    parser.errors.push(err);
                    while parser.peek().is_some() {
                        parser.synchronize();
                        if let Err(err) = parser._parse() {
                            parser.errors.push(err);
                        }
                    }
                }
            }
        }

        if parser.errors.is_empty() {
            Ok(statements)
        } else {
            Err(ParserErrorSet {
                errors: parser.errors,
            })
        }
    }

    fn new(input: &'a str, repl: bool) -> Self {
        let boxed_iter: Box<dyn Iterator<Item = Token> + 'a> = Box::new(tokenize(input));
        let iter = boxed_iter.peekable();

        let mut parser = Self {
            input,
            iter,
            repl,
            inside_loop: false,
            errors: vec![],
        };
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
            match t.kind {
                TokenKind::Whitespace
                | TokenKind::SingleLineComment(_)
                | TokenKind::MultiLineComment(_) => {
                    self.next();
                }
                _ => {
                    break;
                }
            }
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

    fn matches(&mut self, f: impl FnOnce(&Token) -> bool) -> bool {
        self.consume_whitespace();
        self.peek().map_or(false, |t| f(t))
    }

    fn consume(
        &mut self,
        f: impl FnOnce(&Token) -> bool,
        message: &str,
    ) -> Result<Token, ParserError> {
        if self.matches(f) {
            unsafe { Ok(self.next().unwrap_unchecked()) }
        } else {
            Err(ParserError::new_with_token(message, self.peek()))
        }
    }

    fn _parse(&mut self) -> StatementParseResult {
        self.declaration()
    }

    fn declaration(&mut self) -> StatementParseResult {
        self.consume_whitespace();
        match self.peek() {
            Some(Token {
                kind: TokenKind::Fun,
                ..
            }) => self.fun_declaration(),
            Some(Token {
                kind: TokenKind::Class,
                ..
            }) => self.class_declaration(),
            Some(Token {
                kind: TokenKind::Var,
                ..
            }) => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn class_declaration(&mut self) -> StatementParseResult {
        let class_token = unsafe { self.next().unwrap_unchecked() };
        let name: Identifier = unsafe {
            self.consume(
                |t| matches!(t.kind, TokenKind::Ident(_)),
                "expect class name.",
            )?
            .try_into()
            .unwrap_unchecked()
        };

        self.consume(
            |t| t.kind == TokenKind::LeftBrace,
            "expect '{' before class body.",
        )?;

        let mut methods = Vec::new();
        while !self.matches(|t| t.kind == TokenKind::RightBrace) {
            methods.push(self.function("method")?.1);
        }

        let right_brace = self.consume(
            |t| t.kind == TokenKind::RightBrace,
            "expect '}' after class body.",
        )?;

        Ok(Statement::new_class(
            Span::union(&class_token.span, &right_brace.span),
            name,
            methods,
        ))
    }

    fn fun_declaration(&mut self) -> StatementParseResult {
        let fun_token = unsafe { self.next().unwrap_unchecked() };
        let (
            end_span,
            Function {
                name,
                parameters,
                body,
            },
        ) = self.function("function")?;

        Ok(Statement::function(
            Span::union(&fun_token.span, &end_span),
            name,
            parameters,
            body,
        ))
    }
    fn function(&mut self, kind: &str) -> Result<(Span, Function), ParserError> {
        let name = self.consume(
            |t| matches!(t.kind, TokenKind::Ident(_)),
            &format!("expect {kind} name."),
        )?;

        self.consume(
            |t| t.kind == TokenKind::LeftParen,
            &format!("expect '(' after {kind} name."),
        )?;
        let mut parameters = vec![];
        if !self.matches(|t| t.kind == TokenKind::RightParen) {
            loop {
                let param = self.consume(
                    |t| matches!(t.kind, TokenKind::Ident(_)),
                    "expect parameter name.",
                )?;
                if parameters.len() >= MAXIMUM_ARGUMETN_COUNT {
                    self.errors.push(ParserError::new_with_span(
                        &format!("can't have more than {MAXIMUM_ARGUMETN_COUNT} parameters."),
                        param.span.clone(),
                    ));
                }

                parameters.push(unsafe { param.try_into().unwrap_unchecked() });
                if self.matches(|t| t.kind == TokenKind::Comma) {
                    self.next();
                } else {
                    break;
                }
            }
        }

        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "expect ')' after parameters.",
        )?;

        if self.matches(|t| t.kind == TokenKind::LeftBrace) {
            let (span, block) = self.block()?;

            Ok((
                span,
                Function {
                    name: unsafe { name.try_into().unwrap_unchecked() },
                    parameters,
                    body: block,
                },
            ))
        } else {
            Err(ParserError::new_with_token(
                &format!("expect '{{' before {kind} body."),
                self.peek(),
            ))
        }
    }

    fn var_declaration(&mut self) -> StatementParseResult {
        let var = unsafe { self.next().unwrap_unchecked() };
        self.consume_whitespace();
        let ident = self.consume(
            |t| matches!(t.kind, TokenKind::Ident(_)),
            "expected variable name.",
        )?;

        let mut initializer = None;
        if self.matches(|t| t.kind == TokenKind::Equal) {
            self.next();
            initializer = Some(self.expression()?);
        }

        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "expected ';' after variable declaration.",
        )?;

        Ok(Statement::variable(
            Span::union(&var.span, &semicolon.span),
            unsafe { ident.try_into().unwrap_unchecked() },
            initializer,
        ))
    }

    fn statement(&mut self) -> StatementParseResult {
        self.consume_whitespace();
        match self.peek() {
            Some(Token {
                kind: TokenKind::Print,
                ..
            }) => self.print_statement(),
            Some(Token {
                kind: TokenKind::LeftBrace,
                ..
            }) => {
                let (span, statements) = self.block()?;
                Ok(Statement::block(span, statements))
            }
            Some(Token {
                kind: TokenKind::If,
                ..
            }) => self.if_statement(),
            Some(Token {
                kind: TokenKind::While,
                ..
            }) => self.while_statement(),
            Some(Token {
                kind: TokenKind::For,
                ..
            }) => self.for_statement(),
            Some(Token {
                kind: TokenKind::Break | TokenKind::Continue,
                ..
            }) => self.jump_statement(),
            Some(Token {
                kind: TokenKind::Return,
                ..
            }) => self.return_statement(),
            _ => self.expression_statement(),
        }
    }

    fn return_statement(&mut self) -> StatementParseResult {
        let token = unsafe { self.next().unwrap_unchecked() };

        let ret_val = if self.matches(|t| t.kind == TokenKind::Semicolon) {
            Expr::literal(Span::dumb(), Literal::Nil)
        } else {
            self.expression()?
        };

        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            &format!("expect ';' after return value."),
        )?;

        Ok(Statement {
            span: Span::union(&token.span, &semicolon.span),
            kind: StatementKind::Return(ret_val),
        })
    }

    fn jump_statement(&mut self) -> StatementParseResult {
        let jump = unsafe { self.next().unwrap_unchecked() };
        let jump_str = &self.input[jump.span.pos..(jump.span.pos + jump.span.len)];

        if !self.inside_loop {
            return Err(ParserError::new_with_span(
                &format!("unexpected '{}' statement outside a loop.", jump_str),
                jump.span,
            ));
        }

        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            &format!("expect ';' after '{}'", jump_str),
        )?;
        let span = Span::union(&jump.span, &semicolon.span);

        Ok(match jump.kind {
            TokenKind::Break => Statement::new_break(span),
            TokenKind::Continue => Statement::new_continue(span),
            _ => unreachable!(),
        })
    }

    fn for_statement(&mut self) -> StatementParseResult {
        let for_token = unsafe { self.next().unwrap_unchecked() };
        self.consume(
            |t| t.kind == TokenKind::LeftParen,
            "expect '(' after 'for'.",
        )?;

        self.consume_whitespace();
        let initializer = match self.peek() {
            Some(Token {
                kind: TokenKind::Semicolon,
                ..
            }) => {
                self.next();
                None
            }
            Some(Token {
                kind: TokenKind::Var,
                ..
            }) => Some(self.var_declaration()?),
            _ => Some(self.expression_statement()?),
        };

        let condition = if self.matches(|t| t.kind == TokenKind::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "expect ';' after loop condition.",
        )?;

        let increment = if self.matches(|t| t.kind == TokenKind::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "expect ')' after for clauses.",
        )?;

        let prev_inside_loop = self.inside_loop;
        self.inside_loop = true;
        let body = self.statement()?;
        let end_span = body.span.clone();
        self.inside_loop = prev_inside_loop;

        let body = match body.kind {
            StatementKind::Block(statements) => statements,
            _ => vec![body],
        };

        Ok(Statement::new_for(
            Span::union(&for_token.span, &end_span),
            initializer,
            condition,
            increment,
            body,
        ))
    }

    fn while_statement(&mut self) -> StatementParseResult {
        let while_token = unsafe { self.next().unwrap_unchecked() };

        self.consume(
            |t| t.kind == TokenKind::LeftParen,
            "expect '(' after 'while'.",
        )?;
        let condition = self.expression()?;
        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "expected ')' after if condition.",
        )?;

        let prev_inside_loop = self.inside_loop;
        self.inside_loop = true;
        let body = self.statement()?;
        self.inside_loop = prev_inside_loop;

        Ok(Statement::new_while(
            Span::union(&while_token.span, &body.span),
            condition,
            body,
        ))
    }

    fn if_statement(&mut self) -> StatementParseResult {
        let if_token = unsafe { self.next().unwrap_unchecked() };
        self.consume(
            |t| t.kind == TokenKind::LeftParen,
            "expected '(' after 'if'.",
        )?;
        let condition = self.expression()?;
        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "expected ')' after if condition.",
        )?;

        let then_branch = self.statement()?;
        let else_branch = if self.matches(|t| t.kind == TokenKind::Else) {
            self.next();
            Some(self.statement()?)
        } else {
            None
        };

        let span = if let Some(statement) = &else_branch {
            Span::union(&if_token.span, &statement.span)
        } else {
            Span::union(&if_token.span, &then_branch.span)
        };

        Ok(Statement::new_if(span, condition, then_branch, else_branch))
    }

    fn block(&mut self) -> Result<(Span, Vec<Statement>), ParserError> {
        let left_brace = unsafe { self.next().unwrap_unchecked() };
        let mut statements = Vec::<Statement>::new();

        while self.matches(|t| !matches!(t.kind, TokenKind::RightBrace | TokenKind::Eof)) {
            statements.push(self.declaration()?);
        }

        let right_brace = self.consume(
            |t| t.kind == TokenKind::RightBrace,
            "expected closing '}' after block.",
        )?;

        Ok((Span::union(&left_brace.span, &right_brace.span), statements))
    }

    fn expression_statement(&mut self) -> StatementParseResult {
        let expr = self.expression()?;
        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "expected ';' after expression",
        );
        let closed = semicolon.is_ok();

        if !self.repl {
            if let Err(err) = semicolon {
                return Err(err);
            }
        }

        let span = if self.repl {
            expr.span.clone()
        } else {
            let semicolon = unsafe { semicolon.unwrap_unchecked() };
            Span::union(&expr.span, &semicolon.span)
        };

        Ok(Statement::expression(span, expr, closed))
    }

    fn print_statement(&mut self) -> StatementParseResult {
        let print = unsafe { self.next().unwrap_unchecked() };
        let expr = self.expression()?;
        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "expected ';' after expression",
        )?;

        Ok(Statement::print(
            Span::union(&print.span, &semicolon.span),
            expr,
        ))
    }

    fn expression(&mut self) -> ExpressionParseResult {
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
                self.assignment()
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

    fn comma(&mut self) -> ExpressionParseResult {
        let mut expr = self.assignment()?;

        while self.matches(|t| t.kind == TokenKind::Comma) {
            let _token = self.next().unwrap();
            let right = self.assignment()?;
            expr = Expr::binary(expr, BinaryOperator::Comma, right);
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> ExpressionParseResult {
        let mut expr = self.or()?;

        // Ternaru oprator
        if self.matches(|t| t.kind == TokenKind::Question) {
            self.next();
            let middle = self.expression()?;
            if self.matches(|t| t.kind == TokenKind::Colon) {
                self.next();
                let right = self.assignment()?;
                expr = Expr::ternary(expr, middle, right);
            } else {
                return Err(ParserError::new_with_token("Expected ':'", self.peek()));
            }
        }
        // Assignment
        else if self.matches(|t| t.kind == TokenKind::Equal) {
            self.next();
            let value = self.assignment()?;

            if let ExprKind::Variable { name } = expr.kind {
                expr = Expr::assign(Span::union(&expr.span, &value.span), name, value);
            } else {
                return Err(ParserError::new_with_token(
                    "Invalid assignment target",
                    self.peek(),
                ));
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> ExpressionParseResult {
        let mut expr = self.and()?;

        while self.matches(|t| t.kind == TokenKind::Or) {
            let operator = unsafe {
                self.next()
                    .unwrap_unchecked()
                    .kind
                    .try_into()
                    .unwrap_unchecked()
            };
            let right = self.and()?;
            expr = Expr::logical(expr, operator, right)
        }

        Ok(expr)
    }

    fn and(&mut self) -> ExpressionParseResult {
        let mut expr = self.equality()?;

        while self.matches(|t| t.kind == TokenKind::And) {
            let operator = unsafe {
                self.next()
                    .unwrap_unchecked()
                    .kind
                    .try_into()
                    .unwrap_unchecked()
            };
            let right = self.equality()?;
            expr = Expr::logical(expr, operator, right)
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ExpressionParseResult {
        let mut expr = self.comparison()?;

        while self.matches(Self::is_equality_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ExpressionParseResult {
        let mut expr = self.term()?;

        while self.matches(Self::is_comparison_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> ExpressionParseResult {
        let mut expr = self.factor()?;

        while self.matches(Self::is_term_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ExpressionParseResult {
        let mut expr = self.unary()?;

        while self.matches(Self::is_factor_operator) {
            let token = self.next().unwrap();
            let operator = BinaryOperator::try_from(token.kind).unwrap();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ExpressionParseResult {
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
            _ => self.call(),
        }
    }

    fn call(&mut self) -> ExpressionParseResult {
        let mut expr = self.primary()?;
        let initial_span = expr.span.clone();

        while self.matches(|t| t.kind == TokenKind::LeftParen) {
            self.next();
            let mut arguments = Vec::<Expr>::new();
            if !self.matches(|t| t.kind == TokenKind::RightParen) {
                arguments.push(self.assignment()?);
                while self.matches(|t| t.kind == TokenKind::Comma) {
                    self.next();
                    let argument = self.assignment()?;
                    if arguments.len() >= MAXIMUM_ARGUMETN_COUNT {
                        self.errors.push(ParserError::new_with_span(
                            &format!("can't have more than {MAXIMUM_ARGUMETN_COUNT} arguments."),
                            argument.span.clone(),
                        ));
                    }
                    arguments.push(argument);
                }
            }

            let closing_paren = self.consume(
                |t| t.kind == TokenKind::RightParen,
                "expect ')' after arguments.",
            )?;

            expr = Expr::call(
                Span::union(&initial_span, &closing_paren.span),
                expr,
                arguments,
            );
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ExpressionParseResult {
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
            Some(Token {
                kind: TokenKind::Ident(name),
                span,
            }) => Ok(Expr::variable(span, name)),

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
            Some(Token {
                span,
                kind: TokenKind::Unexpected(reason),
            }) => Err(ParserError::new_with_span(
                &format!("Unexpected token: {reason}",),
                span,
            )),
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
