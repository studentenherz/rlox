#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub pos: usize,
    pub len: usize,
}

impl Span {
    pub fn union(start: &Span, end: &Span) -> Self {
        Self {
            line: start.line,
            col: start.col,
            pos: start.pos,
            len: (end.pos + end.len) - start.pos,
        }
    }
}
