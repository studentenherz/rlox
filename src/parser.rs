use std::iter::Peekable;

use crate::lexer::tokenize;
use crate::{
    common::Span,
    constants::MAXIMUM_ARGUMETN_COUNT,
    errors::{LoxError, LoxErrorSet},
    expressions::*,
    lexer::{Token, TokenKind},
    statements::*,
};

type ExpressionParseResult = Result<Expr, LoxError>;
type StatementParseResult = Result<Statement, LoxError>;
type ParseResult = Result<Vec<Statement>, LoxErrorSet>;

pub struct Parser<'a> {
    input: &'a str,
    iter: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
    repl: bool,
    inside_loop: bool,
    errors: Vec<LoxError>,
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
                        parser.consume_whitespace();
                        if parser.peek().is_some() {
                            if let Err(err) = parser._parse() {
                                parser.errors.push(err);
                            }
                        }
                    }
                }
            }
        }

        if parser.errors.is_empty() {
            Ok(statements)
        } else {
            Err(parser.errors)
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
    ) -> Result<Token, LoxError> {
        if self.matches(f) {
            unsafe { Ok(self.next().unwrap_unchecked()) }
        } else {
            Err(LoxError::new_with_token(message, self.peek()))
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
                "Expect class name.",
            )?
            .try_into()
            .unwrap_unchecked()
        };

        let mut superclass = None;
        if self.matches(|t| t.kind == TokenKind::Less) {
            self.next();
            self.consume_whitespace();
            match self.next() {
                Some(Token {
                    span,
                    kind: TokenKind::Ident(name),
                }) => superclass = Some(Expr::variable(span, name)),
                _ => {
                    return Err(LoxError::new_with_token(
                        "Expect superclass name.",
                        self.peek(),
                    ));
                }
            }
        }

        self.consume(
            |t| t.kind == TokenKind::LeftBrace,
            "Expect '{' before class body.",
        )?;

        let mut methods = Vec::new();
        while !self.matches(|t| t.kind == TokenKind::RightBrace) {
            match self.function("method") {
                Ok((_, fun)) => methods.push(fun),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        let right_brace = self.consume(
            |t| t.kind == TokenKind::RightBrace,
            "Expect '}' after class body.",
        )?;

        Ok(Statement::new_class(
            Span::union(&class_token.span, &right_brace.span),
            name,
            superclass,
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
    fn function(&mut self, kind: &str) -> Result<(Span, Function), LoxError> {
        let name = self.consume(
            |t| matches!(t.kind, TokenKind::Ident(_)),
            &format!("Expect {kind} name."),
        )?;

        self.consume(
            |t| t.kind == TokenKind::LeftParen,
            &format!("Expect '(' after {kind} name."),
        )?;
        let mut parameters = vec![];
        if !self.matches(|t| t.kind == TokenKind::RightParen) {
            loop {
                let param = self.consume(
                    |t| matches!(t.kind, TokenKind::Ident(_)),
                    "Expect parameter name.",
                )?;
                if parameters.len() >= MAXIMUM_ARGUMETN_COUNT {
                    self.errors.push(LoxError::new_with_span(
                        &format!("Can't have more than {MAXIMUM_ARGUMETN_COUNT} parameters."),
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
            "Expect ')' after parameters.",
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
            Err(LoxError::new_with_token(
                &format!("Expect '{{' before {kind} body."),
                self.peek(),
            ))
        }
    }

    fn var_declaration(&mut self) -> StatementParseResult {
        let var = unsafe { self.next().unwrap_unchecked() };
        self.consume_whitespace();
        let ident = self.consume(
            |t| matches!(t.kind, TokenKind::Ident(_)),
            "Expect variable name.",
        )?;

        let mut initializer = None;
        if self.matches(|t| t.kind == TokenKind::Equal) {
            self.next();
            initializer = Some(self.expression()?);
        }

        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
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
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(
            |t| t.kind == TokenKind::Semicolon,
            &format!("Expect ';' after return value."),
        )?;

        Ok(Statement {
            span: token.span,
            kind: StatementKind::Return(ret_val),
        })
    }

    fn jump_statement(&mut self) -> StatementParseResult {
        let jump = unsafe { self.next().unwrap_unchecked() };
        let jump_str = &self.input[jump.span.pos..(jump.span.pos + jump.span.len)];

        if !self.inside_loop {
            return Err(LoxError::new_with_span(
                &format!("Unexpected '{}' statement outside a loop.", jump_str),
                jump.span,
            ));
        }

        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            &format!("Expect ';' after '{}'", jump_str),
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
            "Expect '(' after 'for'.",
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
            "Expect ';' after loop condition.",
        )?;

        let increment = if self.matches(|t| t.kind == TokenKind::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "Expect ')' after for clauses.",
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
            "Expect '(' after 'while'.",
        )?;
        let condition = self.expression()?;
        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "Expect ')' after if condition.",
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
        self.consume(|t| t.kind == TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(
            |t| t.kind == TokenKind::RightParen,
            "Expect ')' after if condition.",
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

    fn block(&mut self) -> Result<(Span, Vec<Statement>), LoxError> {
        let left_brace = unsafe { self.next().unwrap_unchecked() };
        let mut statements = Vec::<Statement>::new();

        while self.matches(|t| !matches!(t.kind, TokenKind::RightBrace | TokenKind::Eof)) {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        let right_brace = self.consume(
            |t| t.kind == TokenKind::RightBrace,
            "Expect closing '}' after block.",
        )?;

        Ok((Span::union(&left_brace.span, &right_brace.span), statements))
    }

    fn expression_statement(&mut self) -> StatementParseResult {
        let expr = self.expression()?;
        let semicolon = self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "Expect ';' after expression",
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
        let end_span = match self.consume(
            |t| t.kind == TokenKind::Semicolon,
            "Expect ';' after expression",
        ) {
            Ok(t) => Some(t.span),
            Err(err) => {
                let span = err.span.clone();
                self.errors.push(err);
                span
            }
        };

        let span = if let Some(end_span) = end_span {
            Span::union(&print.span, &end_span)
        } else {
            print.span
        };

        Ok(Statement::print(span, expr))
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

            Err(LoxError::new_with_span(
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

        // Ternary oprator
        if self.matches(|t| t.kind == TokenKind::Question) {
            self.next();
            let middle = self.expression()?;
            if self.matches(|t| t.kind == TokenKind::Colon) {
                self.next();
                let right = self.assignment()?;
                expr = Expr::ternary(expr, middle, right);
            } else {
                return Err(LoxError::new_with_token("Expected ':'", self.peek()));
            }
        }
        // Assignment & set
        else if self.matches(|t| t.kind == TokenKind::Equal) {
            let equal_token = unsafe { self.next().unwrap_unchecked() };
            let value = self.assignment()?;
            let span = Span::union(&expr.span, &value.span);

            match expr.kind {
                ExprKind::Variable { name } => {
                    expr = Expr::assign(span, name, value);
                }
                ExprKind::Get { object, name } => {
                    expr = Expr::set(span, *object, name, value);
                }
                _ => {
                    self.errors.push(LoxError::new_with_span(
                        "Invalid assignment target.",
                        equal_token.span,
                    ));
                }
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

        loop {
            self.consume_whitespace();
            match self.peek() {
                Some(Token {
                    kind: TokenKind::LeftParen,
                    ..
                }) => {
                    self.next();
                    let mut arguments = Vec::<Expr>::new();
                    if !self.matches(|t| t.kind == TokenKind::RightParen) {
                        arguments.push(self.assignment()?);
                        while self.matches(|t| t.kind == TokenKind::Comma) {
                            self.next();
                            let argument = self.assignment()?;
                            if arguments.len() >= MAXIMUM_ARGUMETN_COUNT {
                                self.errors.push(LoxError::new_with_span(
                                    &format!(
                                        "Can't have more than {MAXIMUM_ARGUMETN_COUNT} arguments."
                                    ),
                                    argument.span.clone(),
                                ));
                            }
                            arguments.push(argument);
                        }
                    }

                    let closing_paren = self.consume(
                        |t| t.kind == TokenKind::RightParen,
                        "Expect ')' after arguments.",
                    )?;

                    expr = Expr::call(
                        Span::union(&expr.span, &closing_paren.span),
                        expr,
                        arguments,
                    );
                }
                Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) => {
                    self.next();
                    let ident: Identifier = unsafe {
                        self.consume(
                            |t| matches!(t.kind, TokenKind::Ident(_)),
                            "Expect property name after '.'.",
                        )?
                        .try_into()
                        .unwrap_unchecked()
                    };
                    expr = Expr::get(expr, ident);
                }
                _ => break,
            }
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
            Some(Token {
                kind: TokenKind::This,
                span,
            }) => Ok(Expr::this(span)),
            Some(Token {
                kind: TokenKind::Super,
                span,
            }) => {
                self.consume(|t| t.kind == TokenKind::Dot, "Expect '.' after 'super'.")?;

                let method = unsafe {
                    self.consume(
                        |t| matches!(t.kind, TokenKind::Ident(_)),
                        "Expect superclass method name.",
                    )?
                    .try_into()
                    .unwrap_unchecked()
                };

                Ok(Expr {
                    span,
                    kind: ExprKind::Super { method },
                    resolved_depth: None,
                })
            }
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
                    Some(t) => Err(LoxError::new_with_span(
                        "Expect ')' after expression",
                        t.span,
                    )),
                    None => Err(LoxError::new("Unexpected EOF")),
                }
            }
            Some(Token {
                span,
                kind: TokenKind::Unexpected(reason),
            }) => Err(LoxError::new_with_span(
                &format!("Unexpected token: {reason}",),
                span,
            )),
            Some(t) => Err(LoxError::new_with_span("Expect expression.", t.span)),
            None => Ok(Expr::eof()),
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
