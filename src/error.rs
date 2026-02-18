use crate::Span;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnclosedString,
    UnexpectedCharacter,
}

#[derive(Debug, Clone)]
pub struct CompilerError<'a> {
    pub kind: ErrorKind,
    pub span: Span,
    pub input: &'a [u8],
    pub filename: String,
}

impl<'a> CompilerError<'a> {
    #[cold]
    pub fn new(kind: ErrorKind, span: Span, input: &'a [u8], filename: &str) -> Self {
        Self {
            kind,
            span,
            input,
            filename: filename.to_string(),
        }
    }
}
#[cold]
fn line_col(input: &[u8], pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for &c in &input[..pos] {
        if c == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[cold]
fn snippet(input: &[u8], span: Span) -> String {
    String::from_utf8_lossy(&input[span.start_pos..span.end_pos]).to_string()
}
#[cold]
fn format(kind: &ErrorKind, span: Span, input: &[u8], filename: &String) -> String {
    let (start_line, start_col) = line_col(input, span.start_pos);
    let (end_line, end_col) = line_col(input, span.end_pos);
    let msg = message(kind, span, input);
    format!(
        "{}:{}~{}:{}~{} {:?}: {}",
        filename, start_line, end_line, start_col, end_col, kind, msg
    )
}

#[cold]
fn message(kind: &ErrorKind, span: Span, input: &[u8]) -> String {
    let s = snippet(input, span);

    match kind {
        ErrorKind::UnclosedString => format!(
            "Unclosed string literal '{}'",
            s,
        ),
        ErrorKind::UnexpectedCharacter => {
            let b = if span.start_pos < input.len() {
                input[span.start_pos] as usize
            } else {
                0
            };
            format!("Unexpected character '{}'(0x{})", s, b)
        }
    }
}

impl fmt::Display for ErrorKind {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> Error for CompilerError<'a> {}

impl<'a> fmt::Display for CompilerError<'a> {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format(&self.kind, self.span, self.input, &self.filename)
        )
    }
}

pub type Result<'a, T> = std::result::Result<T, CompilerError<'a>>;
