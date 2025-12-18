use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub enum Token {
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
    Class,
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
    Unexpected { line: usize, col: usize },

    // Meaningless lexemes
    Comment(String),
    Whitespace,
}

const EOF_CHAR: char = '\0';

struct Cursor<'a> {
    iter: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    prev: char,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        let iter = input.chars().peekable();
        Self {
            iter,
            line: 1,
            col: 0,
            prev: EOF_CHAR,
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.prev == '\n' {
            self.line += 1;
            self.col = 0;
        }
        self.col += 1;

        let _next = self.iter.next();
        self.prev = _next.unwrap_or(EOF_CHAR);

        _next
    }

    fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    fn next_matches(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(actual) if *actual == expected => {
                self.next();
                true
            }
            _ => false,
        }
    }
    fn advance_token(&mut self) -> Token {
        if let Some(first_char) = self.next() {
            match first_char {
                c if c.is_whitespace() => {
                    self.eat_while(char::is_whitespace);
                    Token::Whitespace
                }
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '-' => Token::Minus,
                '+' => Token::Plus,
                ';' => Token::Semicolon,
                '*' => Token::Star,
                '!' => {
                    if self.next_matches('=') {
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                }
                '=' => {
                    if self.next_matches('=') {
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '<' => {
                    if self.next_matches('=') {
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if self.next_matches('=') {
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '/' => {
                    if self.next_matches('/') {
                        let comment = self.take_while(|c| c != '\n');
                        Token::Comment(comment)
                    } else {
                        Token::Slash
                    }
                }
                '"' => self.string(),
                c if c.is_digit(10) => self.number(c),
                c if c.is_ascii_alphabetic() => self.identifier(c),
                _ => Token::Unknown,
            }
        } else {
            Token::Eof
        }
    }

    fn identifier(&mut self, first_char: char) -> Token {
        let ident = format!(
            "{}{}",
            first_char,
            self.take_while(|c| char::is_ascii_alphanumeric(&c) || c == '_')
        );

        match ident.as_str() {
            "and" => Token::And,
            "class" => Token::Class,
            "else" => Token::Else,
            "false" => Token::False,
            "for" => Token::For,
            "fun" => Token::Fun,
            "if" => Token::If,
            "nil" => Token::Nil,
            "or" => Token::Or,
            "print" => Token::Print,
            "return" => Token::Return,
            "super" => Token::Super,
            "this" => Token::This,
            "true" => Token::True,
            "var" => Token::Var,
            "while" => Token::While,
            _ => Token::Ident(ident),
        }
    }

    fn number(&mut self, first_char: char) -> Token {
        let mut has_dot = false;
        let number = format!(
            "{}{}",
            first_char,
            self.take_while(move |c| {
                if c.is_digit(10) {
                    return true;
                }

                if c == '.' && !has_dot {
                    has_dot = true;
                    return true;
                }
                false
            })
        );
        if let Ok(number) = number.parse::<f64>() {
            return Token::Number(number);
        }

        Token::Unknown
    }

    fn string(&mut self) -> Token {
        let mut escaped = false;
        let string = self.take_while(move |c| {
            let cont = escaped || c != '"';
            escaped = c == '\\';
            cont
        });

        if self.peek() != Some(&'"') {
            return Token::Unexpected {
                line: self.line,
                col: self.col,
            };
        }

        self.next();
        Token::String(string)
    }

    fn take_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> String {
        let mut string = String::new();
        while let Some(second_char) = self.peek() {
            if !predicate(*second_char) {
                break;
            }
            string.push(*second_char);
            self.next();
        }

        string
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while let Some(second_char) = self.peek() {
            if !predicate(*second_char) {
                break;
            }
            self.next();
        }
    }
}

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.advance_token();
        match token {
            Token::Eof => None,
            _ => Some(token),
        }
    })
}
