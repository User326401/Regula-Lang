use crate::Span;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnclosedString,
    UnexpectedCharacter,
    UnclosedParenthesis,
    MissingSemicolon,
    InvalidSyntax,
    ExpectedInteger,
    ExpectedFloat,
    ExpectedIdentifier,
    ExpectedString,
    ExpectedBoolean,
    ExpectedNone,
    ExpectedType,
    MissingTypeAnnotation,
    MissingAssignment,
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub span: Span,
    pub input: Vec<u8>,
    pub filename: String,
}

impl CompilerError {
    #[cold]
    pub fn new(kind: ErrorKind, span: Span, input: &[u8], filename: String) -> Self {
        Self {
            kind,
            span,
            input: input.to_vec(),
            filename,
        }
    }
}

#[cold]
fn line_col(input: &Vec<u8>, pos: usize) -> (usize, usize) {
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
fn snippet(input: &Vec<u8>, span: Span) -> String {
    String::from_utf8_lossy(&input[span.start..span.end]).to_string()
}
#[cold]
fn format(kind: &ErrorKind, span: Span, input: &Vec<u8>, filename: &String) -> String {
    let (start_line, start_col) = line_col(input, span.start);
    let (end_line, end_col) = line_col(input, span.end);
    let msg = message(kind, span, input);
    format!(
        "{}:{}~{}:{}~{} {:?}: {}",
        filename, start_line, end_line, start_col, end_col, kind, msg
    )
}

#[cold]
fn message(kind: &ErrorKind, span: Span, input: &Vec<u8>) -> String {
    let s = snippet(input, span);

    match kind {
        ErrorKind::UnclosedString => format!("unclosed string literal `{}`", s,),
        ErrorKind::UnexpectedCharacter => {
            let b = if span.start < input.len() {
                input[span.start] as usize
            } else {
                0
            };
            format!("unexpected character `{}`(0x{})", s, b)
        }
        ErrorKind::UnclosedParenthesis => {
            String::from("unclosed parenthesis (expected `)` to close this `(`")
        }
        ErrorKind::MissingSemicolon => {
            String::from("missing semicolon (please add a `;` after the expression)")
        }
        ErrorKind::InvalidSyntax => format!("invalid syntax `{}`", s),
        ErrorKind::ExpectedInteger => format!("expected an integer, but found `{}`", s),
        ErrorKind::ExpectedFloat => format!("expected a float, but found `{}`", s),
        ErrorKind::ExpectedIdentifier => format!("expected an identifier, but found `{}`", s),
        ErrorKind::ExpectedString => format!("expected a string, but found `{}`", s),
        ErrorKind::ExpectedBoolean => format!("expected a boolean, but found `{}`", s),
        ErrorKind::ExpectedNone => format!("expected a None, but found `{}`", s),
        ErrorKind::ExpectedType => format!("expected a type after `:`, but found `{}`", s),
        ErrorKind::MissingTypeAnnotation => {
            format!("const need a type annotation, but found `{}`", s)
        }
        ErrorKind::MissingAssignment => format!("need a assignment, but found `{}`", s),
    }
}

impl fmt::Display for ErrorKind {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for CompilerError {}

impl fmt::Display for CompilerError {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format(&self.kind, self.span, &self.input, &self.filename)
        )
    }
}

pub type Result<'a, T> = std::result::Result<T, CompilerError>;
