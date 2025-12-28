use std::str::Chars;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
    pub pos: usize,
    pub len: usize,
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
    Unexpected,

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
                    _ => TokenKind::Unknown,
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
            line,
            col,
            pos,
            len: self.pos - pos,
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
            let mut comment = String::new();
            if let Some(mut first_char) = self.peek_first() {
                while let Some(second_char) = self.peek_second() {
                    if first_char == '*' && second_char == '/' {
                        self.bump();
                        self.bump();
                        break;
                    }

                    comment.push(first_char);
                    first_char = second_char;
                    self.bump();
                }
            }

            TokenKind::MultiLineComment(comment)
        } else {
            self.bump();
            TokenKind::Slash
        }
    }

    fn identifier(&mut self) -> TokenKind {
        let ident = self.take_while(Self::is_alphanumeric);

        match ident.as_str() {
            "and" => TokenKind::And,
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
            return TokenKind::Unexpected;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(
        mut actual: impl Iterator<Item = Token>,
        expected: impl IntoIterator<Item = TokenKind>,
    ) {
        for (i, expected_item) in expected.into_iter().enumerate() {
            assert_eq!(
                actual.next().map(|t| t.kind),
                Some(expected_item),
                "comparing item {}",
                i
            );
        }
        assert_eq!(actual.next(), None, "comparing last item");
    }

    #[test]
    fn single_character_tokens() {
        let source = r#"({}),.-+;*/?:"#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::LeftParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::RightParen,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Minus,
                TokenKind::Plus,
                TokenKind::Semicolon,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Question,
                TokenKind::Colon,
            ],
        );
    }

    #[test]
    fn one_or_two_character_tokens() {
        let source = r#"!
        !=
        =
        ==
        >
        >=
        <
        <="#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::Bang,
                TokenKind::Whitespace,
                TokenKind::BangEqual,
                TokenKind::Whitespace,
                TokenKind::Equal,
                TokenKind::Whitespace,
                TokenKind::EqualEqual,
                TokenKind::Whitespace,
                TokenKind::Greater,
                TokenKind::Whitespace,
                TokenKind::GreaterEqual,
                TokenKind::Whitespace,
                TokenKind::Less,
                TokenKind::Whitespace,
                TokenKind::LessEqual,
            ],
        );
    }

    #[test]
    fn idents() {
        let source = r#"variable1 variable_2 cammelCaseVariable _undescore_first"#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::Ident("variable1".to_string()),
                TokenKind::Whitespace,
                TokenKind::Ident("variable_2".to_string()),
                TokenKind::Whitespace,
                TokenKind::Ident("cammelCaseVariable".to_string()),
                TokenKind::Whitespace,
                TokenKind::Ident("_undescore_first".to_string()),
            ],
        );
    }

    #[test]
    fn strings() {
        let source = r#""Valid string even if keywords in"
"Escaped \"string\""
"Invalid string not terminated"#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::String("Valid string even if keywords in".to_string()),
                TokenKind::Whitespace,
                TokenKind::String("Escaped \\\"string\\\"".to_string()),
                TokenKind::Whitespace,
                TokenKind::Unexpected,
            ],
        );
    }

    #[test]
    fn keywords() {
        let source = r#"and
class
else
false
fun
for
if
nil
or
print
super
return
this
true
var
while"#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::And,
                TokenKind::Whitespace,
                TokenKind::Class,
                TokenKind::Whitespace,
                TokenKind::Else,
                TokenKind::Whitespace,
                TokenKind::False,
                TokenKind::Whitespace,
                TokenKind::Fun,
                TokenKind::Whitespace,
                TokenKind::For,
                TokenKind::Whitespace,
                TokenKind::If,
                TokenKind::Whitespace,
                TokenKind::Nil,
                TokenKind::Whitespace,
                TokenKind::Or,
                TokenKind::Whitespace,
                TokenKind::Print,
                TokenKind::Whitespace,
                TokenKind::Super,
                TokenKind::Whitespace,
                TokenKind::Return,
                TokenKind::Whitespace,
                TokenKind::This,
                TokenKind::Whitespace,
                TokenKind::True,
                TokenKind::Whitespace,
                TokenKind::Var,
                TokenKind::Whitespace,
                TokenKind::While,
            ],
        );
    }

    #[test]
    fn comments() {
        let source = r#"// comment! no var/if keyword
/* This is a multi-
line comment */"#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::SingleLineComment(" comment! no var/if keyword".to_string()),
                TokenKind::Whitespace,
                TokenKind::MultiLineComment(" This is a multi-\nline comment ".to_string()),
            ],
        );
    }

    #[test]
    fn fibonacci() {
        let source = r#"fun fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}

print fib(8); // expect: 21"#;
        let actual = tokenize(source);

        assert_tokens(
            actual,
            vec![
                TokenKind::Fun,
                TokenKind::Whitespace,
                TokenKind::Ident("fib".to_string()),
                TokenKind::LeftParen,
                TokenKind::Ident("n".to_string()),
                TokenKind::RightParen,
                TokenKind::Whitespace,
                TokenKind::LeftBrace,
                TokenKind::Whitespace,
                TokenKind::If,
                TokenKind::Whitespace,
                TokenKind::LeftParen,
                TokenKind::Ident("n".to_string()),
                TokenKind::Whitespace,
                TokenKind::Less,
                TokenKind::Whitespace,
                TokenKind::Number(2f64),
                TokenKind::RightParen,
                TokenKind::Whitespace,
                TokenKind::Return,
                TokenKind::Whitespace,
                TokenKind::Ident("n".to_string()),
                TokenKind::Semicolon,
                TokenKind::Whitespace,
                TokenKind::Return,
                TokenKind::Whitespace,
                TokenKind::Ident("fib".to_string()),
                TokenKind::LeftParen,
                TokenKind::Ident("n".to_string()),
                TokenKind::Whitespace,
                TokenKind::Minus,
                TokenKind::Whitespace,
                TokenKind::Number(1f64),
                TokenKind::RightParen,
                TokenKind::Whitespace,
                TokenKind::Plus,
                TokenKind::Whitespace,
                TokenKind::Ident("fib".to_string()),
                TokenKind::LeftParen,
                TokenKind::Ident("n".to_string()),
                TokenKind::Whitespace,
                TokenKind::Minus,
                TokenKind::Whitespace,
                TokenKind::Number(2f64),
                TokenKind::RightParen,
                TokenKind::Semicolon,
                TokenKind::Whitespace,
                TokenKind::RightBrace,
                TokenKind::Whitespace,
                TokenKind::Print,
                TokenKind::Whitespace,
                TokenKind::Ident("fib".to_string()),
                TokenKind::LeftParen,
                TokenKind::Number(8f64),
                TokenKind::RightParen,
                TokenKind::Semicolon,
                TokenKind::Whitespace,
                TokenKind::SingleLineComment(" expect: 21".to_string()),
            ],
        );
    }
}
