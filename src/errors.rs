use crate::common::Span;
use crate::lexer::Token;

#[derive(Debug)]
pub struct LoxError {
    pub reason: String,
    pub span: Option<Span>,
}

pub type LoxErrorSet = Vec<LoxError>;

impl LoxError {
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

pub struct LoxErrorLogger<'a> {
    input: &'a str,
}

impl<'a> LoxErrorLogger<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub fn runtime_error(&self, err: &LoxError) {
        eprintln!("{}", err.reason);
        if let Some(span) = &err.span {
            eprintln!("[line {}]", span.line);
        }
    }

    pub fn error(&self, err: &LoxError) {
        if let Some(span) = &err.span {
            let token = if span.pos < self.input.len() {
                format!("'{}'", &self.input[span.pos..(span.pos + span.len)])
            } else {
                "end".to_string()
            };
            eprintln!("[line {}] Error at {}: {}", span.line, token, err.reason);
        } else {
            eprintln!("Error at end: {}", err.reason);
        }
    }
}
