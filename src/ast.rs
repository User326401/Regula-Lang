use crate::span::Span;

// Everything is an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        lit: Literal,
        span: Span
    },

    Ident {
        name: String,
        span: Span
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span
    },

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span
    },

    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span
    },

    Block {
        exprs: Vec<Expr>,
        span: Span
    },

    Let {
        target: Box<Expr>,
        kind: Option<Type>,
        init: Option<Box<Expr>>,
        span: Span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    Eq,
    Lt,
    Gt,
    Lte,
    Gte,
    LtE,
    GtE,
    EqE,
    Neq,

    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg, // -
    Not, // !

    // Pos // + (unary plus) -Omitted, as it is semantically redundant (+x = x),
    // and has no effect on the expression value
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    None,
}