use crate::Span;
use crate::{CompilerError, ErrorKind};
use crate::{Token, TokenType};
use branches::{likely, unlikely};
use unicode_ident::{is_xid_continue, is_xid_start};

const CHAR_DEC_DIGIT: u16 = 1; // 0-9
const CHAR_BIN_DIGIT: u16 = 1 << 1; // 0-1
const CHAR_OCT_DIGIT: u16 = 1 << 2; // 0-7
const CHAR_HEX_DIGIT: u16 = 1 << 3; // 0-9 a-f A-F
const CHAR_ASCII_ID_START: u16 = 1 << 4; // a-z A-Z, _
const CHAR_UTF8_START: u16 = 1 << 5; // UTF-8 start byte (0xC0 ~ 0xF4)
const CHAR_SYMBOL: u16 = 1 << 6; // Symbol + - * / % = < > ( ) { } [ ] ...
const CHAR_WHITESPACE: u16 = 1 << 7; // Whitespace
const CHAR_ASCII_ID_CONTINUE: u16 = CHAR_ASCII_ID_START | CHAR_DEC_DIGIT; // a-z A-Z, _, 0-9
const CHAR_QUOTE: u16 = 1 << 8; // ", '
const CHAR_SINGLE_QUOTE: u16 = 1 << 9; // '
const CHAR_DOUBLE_QUOTE: u16 = 1 << 10; // "
const CHAR_ESCAPE: u16 = 1 << 11; // \
const CHAR_NEWLINE: u16 = 1 << 12; // \n
const CHAR_NEWLINE_AND_ESCAPE: u16 = CHAR_NEWLINE | CHAR_ESCAPE; // \, \n

#[rustfmt::skip]
const CHAR_TABLE: [u16; 256] = [
    // 0x00
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0080, 0x1080, 0x0080, 0x0080, 0x0080, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,

    // 0x20
    0x0080, 0x0040, 0x0500, 0x0000, 0x0000, 0x0040, 0x0040, 0x0300,
    0x0040, 0x0040, 0x0040, 0x0040, 0x0040, 0x0040, 0x0040, 0x0040,
    0x000F, 0x000F, 0x000D, 0x000D, 0x000D, 0x000D, 0x000D, 0x000D,
    0x0009, 0x0009, 0x0040, 0x0040, 0x0040, 0x0040, 0x0040, 0x0000,

    // 0x40
    0x0000, 0x0018, 0x0018, 0x0018, 0x0018, 0x0018, 0x0018, 0x0010,
    0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    0x0010, 0x0010, 0x0010, 0x0040, 0x0800, 0x0040, 0x0040, 0x0010,

    // 0x60
    0x0000, 0x0018, 0x0018, 0x0018, 0x0018, 0x0018, 0x0018, 0x0010,
    0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    0x0010, 0x0010, 0x0010, 0x0040, 0x0040, 0x0040, 0x0040, 0x0000,

    // 0x80
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,

    // 0xA0
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,

    // 0xC0
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,

    // 0xE0
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

struct SymbolRule {
    max_len: u8,
    expect1: u8,
    expect2: u8,
    single: TokenType,
    double1: TokenType,
    double2: TokenType,
}

const SYMBOL_LOOKUP: [SymbolRule; 128] = {
    const DEFAULT: SymbolRule = SymbolRule {
        max_len: 0,
        expect1: 0,
        expect2: 0,
        single: TokenType::Unknown,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    let mut array = [DEFAULT; 128];
    array[b'!' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'=',
        expect2: 0,
        single: TokenType::Not,
        double1: TokenType::Ne,
        double2: TokenType::Unknown,
    };

    array[b'%' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Mod,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'&' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'&',
        expect2: 0,
        single: TokenType::Ampersand,
        double1: TokenType::And,
        double2: TokenType::Unknown,
    };

    array[b'(' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::LParen,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b')' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::RParen,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'*' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'*',
        expect2: 0,
        single: TokenType::Star,
        double1: TokenType::Power,
        double2: TokenType::Unknown,
    };

    array[b'+' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Plus,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b',' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Comma,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'-' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Minus,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'.' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Dot,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'/' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Slash,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b':' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Colon,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b';' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Semicolon,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'<' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'=',
        expect2: b'<',
        single: TokenType::Lt,
        double1: TokenType::LtE,
        double2: TokenType::BitwiseShl,
    };

    array[b'=' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'=',
        expect2: 0,
        single: TokenType::Assign,
        double1: TokenType::Eq,
        double2: TokenType::Unknown,
    };

    array[b'>' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'=',
        expect2: b'>',
        single: TokenType::Gt,
        double1: TokenType::GtE,
        double2: TokenType::BitwiseShr,
    };

    array[b'[' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::LBracket,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b']' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::RBracket,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'^' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::BitwiseXor,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'{' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::LBrace,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'|' as usize] = SymbolRule {
        max_len: 2,
        expect1: b'|',
        expect2: 0,
        single: TokenType::Pipe,
        double1: TokenType::Or,
        double2: TokenType::Unknown,
    };

    array[b'}' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::RBrace,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array[b'~' as usize] = SymbolRule {
        max_len: 1,
        expect1: 0,
        expect2: 0,
        single: TokenType::Tilde,
        double1: TokenType::Unknown,
        double2: TokenType::Unknown,
    };

    array
};

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a [u8],
    input_len: usize,
    filename: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    #[inline(always)]
    pub fn new(filename: &'a str, input: &'a [u8]) -> Self {
        Self {
            filename,
            input_len: input.len(),
            input,
            pos: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, CompilerError<'_>> {
        loop {
            let start_pos = self.pos;
            let mut pos = start_pos;
            self.read_bytes(&mut pos, CHAR_WHITESPACE); // skip whitespace

            if unlikely(pos >= self.input_len) {
                return Ok(Token::new(TokenType::Eof, Span::new(pos, pos)));
            }

            let b = self.peek_unlocked(&pos);

            // skip comment
            if b == b'/' {
                advance(&mut pos, 1);
                if self.peek(&pos) == Some(b'/') {
                    advance(&mut pos, 1);
                    self.skip_line_comment();
                } else if unlikely(self.peek(&pos) == Some(b'*')) {
                    advance(&mut pos, 1);
                    self.skip_block_comment();
                } else {
                    return Ok(Token::new(TokenType::Slash, Span::new(start_pos, pos)));
                }

                self.pos = pos;
                continue;
            }

            break {
                let mask = CHAR_TABLE[b as usize];

                if (mask & CHAR_SYMBOL) != 0 {
                    self.read_symbol()
                } else if (mask & CHAR_DEC_DIGIT) != 0 {
                    self.read_number()
                } else if (mask & CHAR_ASCII_ID_START) != 0 {
                    self.read_identifier()
                } else if (mask & CHAR_QUOTE) != 0 {
                    self.read_string()
                } else if unlikely(is_xid_start(self.peek_char_unlocked())) {
                    self.read_identifier()
                } else {
                    if unlikely(true) {
                        self.advance_char();
                        return Err(CompilerError::new(
                            ErrorKind::UnexpectedCharacter,
                            Span::new(start_pos, self.pos),
                            self.input,
                            self.filename,
                        ));
                    }

                    unreachable!()
                }
            };
        }
    }

    fn read_identifier(&mut self) -> Result<Token, CompilerError<'_>> {
        let start_pos = self.pos;
        let mut pos = start_pos;
        let input_len = self.input_len;
        self.advance_char();

        while likely(pos < input_len) {
            let b = self.peek_unlocked(&pos);
            if (CHAR_TABLE[b as usize] & CHAR_ASCII_ID_CONTINUE) != 0 {
                advance(&mut pos, 1);
            } else if (CHAR_TABLE[b as usize] & CHAR_UTF8_START) == 0 {
                break;
            } else {
                if unlikely(true) {
                    let c = self.peek_char();
                    match c {
                        Some(c) => {
                            if is_xid_continue(c) {
                                self.advance_char();
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                unreachable!()
            }
        }

        self.pos = pos;
        Ok(Token::new(TokenType::Identifier, Span::new(start_pos, pos)))
    }

    fn read_string(&mut self) -> Result<Token, CompilerError<'_>> {
        let start_pos = self.pos;
        let mut pos = start_pos;
        let quote_mask: u16 = if self.peek_unlocked(&pos) == b'"' {
            CHAR_DOUBLE_QUOTE
        } else {
            CHAR_SINGLE_QUOTE
        };
        advance(&mut pos, 1);

        let input_len = self.input.len();

        while likely(pos < input_len) {
            let b = self.peek_unlocked(&pos);
            let mask = CHAR_TABLE[b as usize];

            if likely((mask & (quote_mask | CHAR_NEWLINE_AND_ESCAPE)) == 0) {
                advance(&mut pos, 1);
                continue;
            }

            if unlikely((mask & quote_mask) != 0) {
                advance(&mut pos, 1);
                break;
            }

            if unlikely((mask & CHAR_ESCAPE) != 0) {
                advance(&mut pos, 2);
                continue;
            }

            if unlikely((mask & CHAR_NEWLINE) != 0) {
                return Err(CompilerError::new(
                    ErrorKind::UnclosedString,
                    Span::new(start_pos, pos),
                    self.input,
                    self.filename,
                ));
            }
        }

        if unlikely(pos > input_len) {
            return Err(CompilerError::new(
                ErrorKind::UnclosedString,
                Span::new(start_pos, pos),
                self.input,
                self.filename,
            ));
        }

        self.pos = pos;
        Ok(Token::new(TokenType::String, Span::new(start_pos, pos)))
    }

    fn read_symbol(&mut self) -> Result<Token, CompilerError<'_>> {
        let start_pos = self.pos;
        let mut pos = start_pos;
        let b = self.peek_unlocked(&pos);

        advance(&mut pos, 1);

        let symbol_info = &SYMBOL_LOOKUP[b as usize];

        if unlikely(symbol_info.max_len == 0) {
            return Err(CompilerError::new(
                ErrorKind::UnexpectedCharacter,
                Span::new(start_pos, start_pos),
                self.input,
                self.filename,
            ));
        }

        let can_peek_next = unlikely(pos < self.input_len - 1 );
        let mut symbol_type = symbol_info.single;
        let mut symbol_span = Span::new(start_pos, pos);

        if likely(can_peek_next) {
            if self.peek_unlocked(&pos) == symbol_info.expect1 {
                symbol_type = symbol_info.double1;
                symbol_span = Span::new(start_pos, pos);
            } else if unlikely(self.peek_unlocked(&pos) == symbol_info.expect2) {
                symbol_type = symbol_info.double2;
                symbol_span = Span::new(start_pos, pos);
            }
        }

        self.pos = pos;
        Ok(Token::new(symbol_type, symbol_span))
    }

    fn read_number(&mut self) -> Result<Token, CompilerError<'_>> {
        let start_pos = self.pos;
        let mut pos = start_pos;
        let mut num_type = TokenType::Int;

        if self.peek_unlocked(&pos) == b'0' {
            advance(&mut pos, 1);
            let can_peek_next = unlikely(pos < self.input_len - 1 );
            if unlikely(!can_peek_next) {
                return Ok(Token::new(TokenType::Int, Span::new(start_pos, self.pos)));
            }

            match self.peek_unlocked(&pos) {
                b'b' | b'B' => {
                    advance(&mut pos, 1);
                    self.read_bytes(&mut pos, CHAR_BIN_DIGIT);
                }
                b'o' | b'O' => {
                    advance(&mut pos, 1);
                    self.read_bytes(&mut pos, CHAR_OCT_DIGIT);
                }
                b'x' | b'X' => {
                    advance(&mut pos, 1);
                    self.read_bytes(&mut pos, CHAR_HEX_DIGIT);
                }
                _ => {
                    self.read_decimal_digits(&mut num_type);
                    pos = self.pos
                }
            }
        } else {
            self.read_decimal_digits(&mut num_type);
            pos = self.pos;
        }

        self.pos = pos;
        Ok(Token::new(num_type, Span::new(start_pos, self.pos)))
    }

    fn read_bytes(&mut self, pos: &mut usize, mask: u16) {
        let input_len = self.input_len;

        while likely(*pos < input_len) {
            let b = self.peek_unlocked(pos);
            if (CHAR_TABLE[b as usize] & mask) != 0 {
                advance(pos, 1)
            } else {
                break;
            }
        }

        self.pos = *pos;
    }

    fn read_decimal_digits(&mut self, num_type: &mut TokenType) {
        let mut pos = self.pos;
        self.read_bytes(&mut pos, CHAR_DEC_DIGIT);

        if unlikely(self.peek(&pos) == Some(b'.')) {
            *num_type = TokenType::Float;
            advance(&mut pos, 1);
            self.read_bytes(&mut pos, CHAR_DEC_DIGIT);
        }

        if unlikely(matches!(self.peek(&pos), Some(b'e' | b'E'))) {
            *num_type = TokenType::Float;
            advance(&mut pos, 1);
            if matches!(self.peek(&pos), Some(b'+' | b'-')) {
                advance(&mut pos, 1);
            }
            self.read_bytes(&mut pos, CHAR_DEC_DIGIT);
        }

        self.pos = pos;
    }

    fn advance_char(&mut self) {
        let b = self.peek(&self.pos);

        if let Some(byte) = b {
            let len = match byte {
                0x00..=0x7F => 1,
                0xC0..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xFF => 4,
                _ => unreachable!(
                    "Input is guaranteed valid UTF-8 '{}' at {}",
                    byte as char, self.pos
                ),
            };
            advance(&mut self.pos, len);
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        let b = self.peek(&self.pos);

        if b.is_some() {
            Some(self.peek_char_unlocked())
        } else {
            None
        }
    }

    fn peek_char_unlocked(&mut self) -> char {
        let b = self.peek_unlocked(&self.pos);

        let code = match b {
            0x00..=0x7F => b as u32,
            0xC0..=0xDF => {
                let b1 = self.input[self.pos + 1];
                ((b & 0x1F) as u32) << 6 | ((b1 & 0x3F) as u32)
            }
            0xE0..=0xEF => {
                let b1 = self.input[self.pos + 1];
                let b2 = self.input[self.pos + 2];
                ((b & 0x0F) as u32) << 12 | ((b1 & 0x3F) as u32) << 6 | ((b2 & 0x3F) as u32)
            }
            0xF0..=0xFF => {
                let b1 = self.input[self.pos + 1];
                let b2 = self.input[self.pos + 2];
                let b3 = self.input[self.pos + 3];
                ((b & 0x07) as u32) << 18
                    | ((b1 & 0x3F) as u32) << 12
                    | ((b2 & 0x3F) as u32) << 6
                    | ((b3 & 0x3F) as u32)
            }
            _ => unreachable!(
                "Input is guaranteed valid UTF-8 '{}' at {}",
                b as char, self.pos
            ),
        };

        unsafe { char::from_u32_unchecked(code) }
    }

    #[inline(always)]
    fn peek(&self, pos: &usize) -> Option<u8> {
        self.input.get(*pos).copied()
    }

    #[inline(always)]
    fn peek_unlocked(&self, pos: &usize) -> u8 {
        unsafe { *self.input.get_unchecked(*pos) }
    }

    fn skip_block_comment(&mut self) {
        let mut pos = self.pos;
        let input_len = self.input_len;

        while likely(pos < input_len) {
            match self.peek_unlocked(&pos) {
                b'*' => {
                    advance(&mut pos, 1);
                    if unlikely(self.peek(&pos) == Some(b'/')) {
                        advance(&mut pos, 1);
                        break;
                    }
                }
                _ => {
                    advance(&mut pos, 1);
                }
            }
        }
    }

    fn skip_line_comment(&mut self) {
        let mut pos = self.pos;
        let input_len = self.input_len;

        while likely(pos < input_len) {
            match self.peek_unlocked(&pos) {
                b'\n' => break,
                _ => {
                    advance(&mut pos, 1);
                }
            }
        }
    }
}

#[inline(always)]
fn advance(pos: &mut usize, n: usize) {
    *pos += n;
}
