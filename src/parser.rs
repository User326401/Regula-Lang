use crate::Span;
use crate::ast::{BinOp, Expr, Program, Type};
use crate::ast::{Literal, UnaryOp};
use crate::error::{CompilerError, ErrorKind};
use crate::token::{Token, TokenType};
use branches::likely;

pub struct Parser<'a> {
    tokens: &'a [Token],
    input: &'a [u8],
    filename: String,
    pos: usize,
}

impl<'a> Parser<'a> {
    #[inline(always)]
    pub fn new(tokens: &'a [Token], input: &'a [u8], filename: String) -> Self {
        Self {
            tokens,
            input,
            filename,
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, CompilerError> {
        let start = self.peek_unlocked().span;
        let mut exprs = Vec::new();

        while likely(self.peek_kind() != TokenType::Eof) {
            exprs.push(self.parse_expr()?);
            if self.eat(TokenType::Semicolon).is_none() {
                return Err(CompilerError::new(
                    ErrorKind::MissingSemicolon,
                    self.peek_unlocked().span,
                    self.input,
                    self.filename.clone(),
                ));
            };
        }

        let span = start.merge(self.eat(TokenType::Eof).unwrap().span);
        Ok(Program {
            body: Expr::Block { exprs, span },
            filename: self.filename.clone(),
            span,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, CompilerError> {
        match self.peek_kind() {
            TokenType::Let => self.parse_let(),
            TokenType::Const => self.parse_const(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, CompilerError> {
        let start = self.eat(TokenType::Let).unwrap().span;

        let target = self.parse_expr()?;
        let mut end = *target.span();
        let mut type_ = Type::Auto;

        if self.eat(TokenType::Colon).is_some() {
            type_ = self.parse_type()?;
            end = self.peek_unlocked().span;
        }

        let init = if self.eat(TokenType::Assign).is_none() {
            None
        } else {
            let expr = self.parse_expr()?;
            Some(Box::new(expr))
        };

        if let Some(init_expr) = &init {
            end = *init_expr.span();
        }

        Ok(Expr::Let {
            target: Box::new(target),
            kind: type_,
            init,
            span: start.merge(end),
        })
    }

    fn parse_const(&mut self) -> Result<Expr, CompilerError> {
        let start = self.eat(TokenType::Const).unwrap().span;

        let target = self.parse_expr()?;
        let type_;

        if self.eat(TokenType::Colon).is_some() {
            type_ = self.parse_type()?;
        } else {
            return Err(CompilerError::new(
                ErrorKind::MissingTypeAnnotation,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            ));
        }

        if self.eat(TokenType::Assign).is_none() {
            return Err(CompilerError::new(
                ErrorKind::MissingAssignment,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            ));
        }

        let expr = self.parse_expr()?;
        let end = *expr.span();

        Ok(Expr::Const {
            target: Box::new(target),
            kind: type_,
            value: Box::new(expr),
            span: start.merge(end),
        })
    }

    fn parse_type(&mut self) -> Result<Type, CompilerError> {
        let type_ = match self.peek_kind() {
            TokenType::TypeInt => Ok(Type::Int),
            TokenType::TypeFloat => Ok(Type::Float),
            TokenType::TypeString => Ok(Type::String),
            TokenType::TypeBoolean => Ok(Type::Bool),
            _ => Err(CompilerError::new(
                ErrorKind::ExpectedType,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
        };

        self.advance();
        type_
    }

    fn parse_or(&mut self) -> Result<Expr, CompilerError> {
        let mut left = self.parse_and()?;
        let start = *left.span();

        while matches!(self.peek_kind(), TokenType::Or) {
            let op = BinOp::Or;
            self.advance();
            let right = self.parse_and()?;
            let end = *right.span();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: start.merge(end),
            }
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, CompilerError> {
        let mut left = self.parse_add()?;
        let start = *left.span();

        while matches!(self.peek_kind(), TokenType::And) {
            let op = BinOp::And;
            self.advance();
            let right = self.parse_add()?;
            let end = *right.span();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: start.merge(end),
            }
        }

        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr, CompilerError> {
        let mut left = self.parse_mul()?;
        let start = *left.span();

        while matches!(self.peek_kind(), TokenType::Plus | TokenType::Minus) {
            let op = match self.peek_kind() {
                TokenType::Plus => BinOp::Add,
                TokenType::Minus => BinOp::Sub,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_mul()?;
            let end = *right.span();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: start.merge(end),
            }
        }

        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, CompilerError> {
        let mut left = self.parse_pow()?;
        let start = *left.span();

        while matches!(
            self.peek_kind(),
            TokenType::Star | TokenType::Slash | TokenType::Mod
        ) {
            let op = match self.peek_kind() {
                TokenType::Star => BinOp::Mul,
                TokenType::Slash => BinOp::Div,
                TokenType::Mod => BinOp::Mod,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_pow()?;
            let end = *right.span();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: start.merge(end),
            }
        }

        Ok(left)
    }

    fn parse_pow(&mut self) -> Result<Expr, CompilerError> {
        let mut left = self.parse_unary()?;
        let start = *left.span();

        if matches!(self.peek_kind(), TokenType::Power) {
            let op = BinOp::Pow;
            self.advance();
            let right = self.parse_pow()?;
            let end = *right.span();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: start.merge(end),
            }
        };

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, CompilerError> {
        if matches!(
            self.peek_kind(),
            TokenType::Plus | TokenType::Minus | TokenType::Not
        ) {
            let start = self.peek_unlocked().span;
            let op = match self.peek_kind() {
                TokenType::Plus => {
                    self.advance();
                    return self.parse_unary();
                }
                TokenType::Minus => UnaryOp::Neg,
                TokenType::Not => UnaryOp::Not,
                _ => unreachable!(),
            };

            self.advance();
            let expr = self.parse_unary()?;
            let end = *expr.span();
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span: start.merge(end),
            });
        };

        self.parse_primary()
    }
    fn parse_primary(&mut self) -> Result<Expr, CompilerError> {
        match self.peek_kind() {
            TokenType::LParen => {
                let left_paren_span = self.peek_unlocked().span;
                self.advance();
                let expr = self.parse_expr()?;
                match self.eat(TokenType::RParen) {
                    None => Err(CompilerError::new(
                        ErrorKind::UnclosedParenthesis,
                        left_paren_span,
                        self.input,
                        self.filename.clone(),
                    )),
                    Some(_) => Ok(expr),
                }
            }
            TokenType::String => self.parse_string(),
            TokenType::Int => self.parse_int(),
            TokenType::Float => self.parse_float(),
            TokenType::Identifier => self.parse_ident(),
            TokenType::True | TokenType::False => self.parse_bool(),
            TokenType::None => self.parse_none(),
            _ => Err(CompilerError::new(
                ErrorKind::InvalidSyntax,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
        }
    }

    fn parse_int(&mut self) -> Result<Expr, CompilerError> {
        let token = self.eat(TokenType::Int);

        match token {
            None => Err(CompilerError::new(
                ErrorKind::ExpectedInteger,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
            Some(tok) => {
                let bytes = self.slice(&tok.span);

                let s = std::str::from_utf8(bytes).unwrap();
                let value = s.parse::<i64>().unwrap();

                Ok(Expr::Literal {
                    lit: Literal::Int(value),
                    span: tok.span,
                })
            }
        }
    }

    fn parse_float(&mut self) -> Result<Expr, CompilerError> {
        let token = self.eat(TokenType::Float);

        match token {
            None => Err(CompilerError::new(
                ErrorKind::ExpectedFloat,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
            Some(tok) => {
                let bytes = self.slice(&tok.span);

                let s = std::str::from_utf8(bytes).unwrap();
                let value = s.parse::<f64>().unwrap();

                Ok(Expr::Literal {
                    lit: Literal::Float(value),
                    span: tok.span,
                })
            }
        }
    }

    fn parse_ident(&mut self) -> Result<Expr, CompilerError> {
        let token = self.eat(TokenType::Identifier);

        match token {
            None => Err(CompilerError::new(
                ErrorKind::ExpectedIdentifier,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
            Some(tok) => {
                let name = std::str::from_utf8(self.slice(&tok.span))
                    .unwrap()
                    .to_string();

                Ok(Expr::Ident {
                    name,
                    span: tok.span,
                })
            }
        }
    }

    fn parse_string(&mut self) -> Result<Expr, CompilerError> {
        let token = self.eat(TokenType::String);

        match token {
            None => Err(CompilerError::new(
                ErrorKind::ExpectedString,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
            Some(tok) => {
                let bytes = self.slice(&tok.span);
                let content_bytes = &bytes[1..bytes.len() - 1];
                let content = std::str::from_utf8(content_bytes).unwrap().to_string();

                Ok(Expr::Literal {
                    lit: Literal::String(content),
                    span: tok.span,
                })
            }
        }
    }

    fn parse_bool(&mut self) -> Result<Expr, CompilerError> {
        let token = self.peek_unlocked();

        match token.token_type {
            TokenType::True | TokenType::False => {
                let value = match self.peek_kind() {
                    TokenType::True => true,
                    TokenType::False => false,
                    _ => unreachable!(),
                };
                self.advance();

                Ok(Expr::Literal {
                    lit: Literal::Bool(value),
                    span: token.span,
                })
            }
            _ => Err(CompilerError::new(
                ErrorKind::ExpectedBoolean,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
        }
    }

    fn parse_none(&mut self) -> Result<Expr, CompilerError> {
        let token = self.eat(TokenType::None);

        match token {
            None => Err(CompilerError::new(
                ErrorKind::ExpectedBoolean,
                self.peek_unlocked().span,
                self.input,
                self.filename.clone(),
            )),
            Some(tok) => Ok(Expr::Literal {
                lit: Literal::None,
                span: tok.span,
            }),
        }
    }

    #[inline(always)]
    fn slice(&self, span: &Span) -> &'a [u8] {
        &self.input[span.start..span.end]
    }

    #[inline(always)]
    fn peek_unlocked(&self) -> &'a Token {
        unsafe { self.tokens.get_unchecked(self.pos) }
    }

    #[inline(always)]
    fn peek_kind(&self) -> TokenType {
        self.peek_unlocked().token_type
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.pos += 1;
    }

    #[inline(always)]
    fn eat(&mut self, token_type: TokenType) -> Option<&'a Token> {
        if likely(self.peek_kind() == token_type) {
            let tok = self.peek_unlocked();
            self.advance();
            Some(tok)
        } else {
            None
        }
    }
}
