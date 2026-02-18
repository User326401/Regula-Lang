use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    Unknown,

    Identifier,
    Int,
    Float,
    String,

    Assign, // =
    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /
    Mod,    // %
    Power,  // **

    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    Comma,     // ,
    Semicolon, // ;
    Colon,     // :
    Dot,       // .

    Ne,  // !=
    Lt,  // <
    Gt,  // >
    LtE, // <=
    GtE, // >=
    Eq,  // ==

    Not, // !
    And, // &&
    Or,  // ||

    Ampersand,  // &
    Pipe,       // |
    Tilde,      // ~
    BitwiseXor, // ^
    BitwiseShl, // <<
    BitwiseShr, // >>

    Let,    // let
    Const,  // const
    Struct, // struct
    Impl,   // impl
    Enum,   // enum
    Mut,    // mut

    If,       // if
    Else,     // else
    While,    // while
    For,      // for
    In,       // in
    Func,     // func
    Return,   // return
    Loop,     // loop
    Break,    // break
    Continue, // continue
    Match,    // match

    Import, // import

    True,  // true
    False, // false

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

impl Token {
    #[inline(always)]
    pub fn new(token_type: TokenType, span: Span) -> Self {
        Self { token_type, span }
    }
}
