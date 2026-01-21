use std::str::Chars;

use crate::common::Span;

#[derive(Debug)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    // Singe-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Question,
    Colon,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literlas
    Ident(String),
    String(String),
    Number(f64),

    // Keywords
    And,
    Break,
    Class,
    Continue,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Super,
    Return,
    This,
    True,
    Var,
    While,

    Eof,
    Unknown,
    Unexpected(String),

    SingleLineComment(String),
    MultiLineComment(String),
    Whitespace,
}

const EOF_CHAR: char = '\0';

struct Cursor<'a> {
    iter: Chars<'a>,
    line: usize,
    col: usize,
    prev: char,
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        let iter = input.chars();
        Self {
            iter,
            line: 1,
            col: 1,
            prev: EOF_CHAR,
            pos: 0,
        }
    }

    fn bump(&mut self) {
        if self.prev == '\n' {
            self.line += 1;
            self.col = 1;
        }
        self.col += 1;
        self.pos += 1;

        let _next = self.iter.next();
        self.prev = _next.unwrap_or(EOF_CHAR);
    }

    fn peek_first(&mut self) -> Option<char> {
        self.iter.clone().next()
    }

    fn peek_second(&mut self) -> Option<char> {
        let mut iter = self.iter.clone();
        iter.next();
        iter.next()
    }

    fn second_matches(&mut self, expected: char) -> bool {
        if let Some(second_char) = self.peek_second() {
            if second_char == expected {
                return true;
            }
        }

        false
    }

    fn advance_token(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let pos = self.pos;

        let token_kind = if let Some(first_char) = self.peek_first() {
            let token = match first_char {
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                '-' => TokenKind::Minus,
                '+' => TokenKind::Plus,
                ';' => TokenKind::Semicolon,
                '*' => TokenKind::Star,
                '?' => TokenKind::Question,
                ':' => TokenKind::Colon,
                '!' => {
                    if self.second_matches('=') {
                        self.bump();
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    }
                }
                '=' => {
                    if self.second_matches('=') {
                        self.bump();
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    }
                }
                '<' => {
                    if self.second_matches('=') {
                        self.bump();
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    }
                }
                '>' => {
                    if self.second_matches('=') {
                        self.bump();
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    }
                }
                _ => TokenKind::Unknown,
            };

            match token {
                TokenKind::Unknown => match first_char {
                    '"' => self.string(),
                    '/' => self.comment_or_slash(),
                    c if c.is_digit(10) => self.number(),
                    c if Self::is_alpha(c) => self.identifier(),
                    c if c.is_whitespace() => {
                        self.eat_while(char::is_whitespace);
                        TokenKind::Whitespace
                    }
                    _ => {
                        self.bump();
                        TokenKind::Unknown
                    }
                },
                _ => {
                    self.bump();
                    token
                }
            }
        } else {
            TokenKind::Eof
        };

        Token {
            kind: token_kind,
            span: Span {
                line,
                col,
                pos,
                len: self.pos - pos,
            },
        }
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        Self::is_alpha(c) || c.is_digit(10)
    }

    fn comment_or_slash(&mut self) -> TokenKind {
        if self.second_matches('/') {
            self.bump();
            self.bump();
            let comment = self.take_while(|c| c != '\n');
            TokenKind::SingleLineComment(comment)
        } else if self.second_matches('*') {
            self.bump();
            self.bump();
            let mut openning_comments = 1usize;
            let mut comment = String::new();
            if let Some(mut first_char) = self.peek_first() {
                while let Some(second_char) = self.peek_second() {
                    if first_char == '/' && second_char == '*' {
                        openning_comments += 1;
                    }
                    if first_char == '*' && second_char == '/' {
                        self.bump();
                        self.bump();
                        openning_comments -= 1;
                        if openning_comments == 0 {
                            break;
                        }
                    }

                    comment.push(first_char);
                    first_char = second_char;
                    self.bump();
                }
            }

            if openning_comments == 0 {
                TokenKind::MultiLineComment(comment)
            } else {
                TokenKind::Unexpected("expect closing '*/' for multiline comment.".to_string())
            }
        } else {
            self.bump();
            TokenKind::Slash
        }
    }

    fn identifier(&mut self) -> TokenKind {
        let ident = self.take_while(Self::is_alphanumeric);

        match ident.as_str() {
            "and" => TokenKind::And,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Ident(ident),
        }
    }

    fn number(&mut self) -> TokenKind {
        let mut has_dot = false;
        let number = self.take_while(move |c| {
            if c.is_digit(10) {
                return true;
            }

            if c == '.' && !has_dot {
                has_dot = true;
                return true;
            }
            false
        });

        if let Ok(number) = number.parse::<f64>() {
            return TokenKind::Number(number);
        }

        TokenKind::Unknown
    }

    fn string(&mut self) -> TokenKind {
        self.bump();
        let mut escaped = false;
        let string = self.take_while(move |c| {
            let cont = escaped || c != '"';
            escaped = c == '\\';
            cont
        });

        if self.peek_first() != Some('"') {
            return TokenKind::Unexpected("expect closing '\"' for string.".to_string());
        }

        self.bump();
        TokenKind::String(string)
    }

    fn take_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> String {
        let mut string = String::new();
        while let Some(second_char) = self.peek_first() {
            if !predicate(second_char) {
                break;
            }
            string.push(second_char);
            self.bump();
        }

        string
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while let Some(second_char) = self.peek_first() {
            if !predicate(second_char) {
                break;
            }
            self.bump();
        }
    }
}

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.advance_token();
        match token.kind {
            TokenKind::Eof => None,
            _ => Some(token),
        }
    })
}
