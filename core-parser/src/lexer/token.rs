use std::fmt;
use std::result::Result;

use crate::lexer::position::{Position, Span};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    // literals
    StringTail(String),
    StringExpr(String),
    LitChar(char),
    LitInt(String, IntBase, IntSuffix),
    LitFloat(String, FloatSuffix),
    Identifier(String),
    True,
    False,
    End,

    // "big" shapes
    Class,
    Enum,
    Value,
    Union,
    Trait,
    Impl,
    Annotation,
    Mod,
    Use,
    Package,

    // "small" shapes
    Fun,
    Let,
    Var,
    Const,

    // control flow
    Return,
    If,
    Else,
    While,
    For,
    In,

    // qualifiers
    This,
    Super,

    // pattern matching
    Is,

    // casting
    As,

    // union variants
    Of,

    // operators – numbers
    Add,
    Sub,
    Mul,
    Div,

    // operators – logic
    Or,
    And,
    Caret,
    AndAnd,
    OrOr,

    // operators – comparisons
    EqEq,
    NotEq,
    EqEqEq,
    NeEqEq,
    Lt,
    Le,
    Gt,
    Ge,

    // basic syntax
    Eq,
    Comma,
    Semicolon,
    Dot,
    DotDotDot,
    Colon,
    ColonColon,
    At,

    // brackets
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // unused
    Type,
    Alias,
    CapitalThis,
    Underscore,
}

impl TokenKind {
    pub fn name(&self) -> &str {
        match *self {
            // literals
            TokenKind::StringTail(_) => "string tail",
            TokenKind::StringExpr(_) => "string epxr",
            TokenKind::LitInt(_, _, suffix) => match suffix {
                IntSuffix::UInt8 => "byte number",
                IntSuffix::Int32 => "int32 number",
                IntSuffix::Int64 => "int64 number",
                IntSuffix::None => "untyped number",
            },
            TokenKind::LitChar(_) => "char",
            TokenKind::LitFloat(_, suffix) => match suffix {
                FloatSuffix::Float32 => "float32 number",
                FloatSuffix::Float64 => "float64 number",
            },
            TokenKind::Identifier(_) => "identifier",
            TokenKind::True => "true",
            TokenKind::False => "false",

            // "big" shapes
            TokenKind::Class => "class",
            TokenKind::Enum => "enum",
            TokenKind::Value => "value",
            TokenKind::Union => "union",
            TokenKind::Trait => "trait",
            TokenKind::Impl => "impl",
            TokenKind::Annotation => "annotation",
            TokenKind::Mod => "mod",
            TokenKind::Use => "use",
            TokenKind::Package => "package",

            // "small" shapes
            TokenKind::Fun => "fun",
            TokenKind::Let => "let",
            TokenKind::Var => "let",
            TokenKind::Const => "const",

            // control flow
            TokenKind::Return => "return",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",

            // qualifiers
            TokenKind::This => "self",
            TokenKind::Super => "super",

            // pattern matching
            TokenKind::Is => "is",

            // casting
            TokenKind::As => "as",

            // union variants
            TokenKind::Of => "of",

            // operators – arithmetic
            TokenKind::Add => "+",
            TokenKind::Sub => "-",
            TokenKind::Mul => "*",
            TokenKind::Div => "/",

            // operators – logic
            TokenKind::Or => "|",
            TokenKind::And => "&",
            TokenKind::Caret => "^",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",

            // operators – comparisons
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::EqEqEq => "===",
            TokenKind::NeEqEq => "!==",
            TokenKind::Lt => "<",
            TokenKind::Le => "<=",
            TokenKind::Gt => ">",
            TokenKind::Ge => ">=",

            // basic syntax
            TokenKind::Eq => "=",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Dot => ".",
            TokenKind::DotDotDot => "...",
            TokenKind::Colon => ":",
            TokenKind::ColonColon => "::",
            TokenKind::At => "@",

            // brackets
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",

            // unused
            TokenKind::Type => "type",
            TokenKind::Alias => "alias",
            TokenKind::CapitalThis => "Self",
            TokenKind::Underscore => "_",

            // end of file
            TokenKind::End => "<<EOF>>",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IntSuffix {
    UInt8,
    Int32,
    Int64,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FloatSuffix {
    Float32,
    Float64,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
    pub span: Span,
}

impl Token {
    pub fn new(tok: TokenKind, pos: Position, span: Span) -> Token {
        Token {
            kind: tok,
            position: pos,
            span,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::End
    }

    pub fn is_identifier(&self) -> bool {
        match self.kind {
            TokenKind::Identifier(_) => true,
            _ => false,
        }
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn name(&self) -> String {
        match self.kind {
            TokenKind::LitInt(ref val, _, suffix) => {
                let suffix = match suffix {
                    IntSuffix::UInt8 => "B",
                    IntSuffix::Int32 => "",
                    IntSuffix::Int64 => "L",
                    IntSuffix::None => "",
                };

                format!("{}{}", val, suffix)
            }

            TokenKind::StringTail(ref val) => format!("\"{}\" tail", &val),
            TokenKind::StringExpr(ref val) => format!("\"{}\" expr", &val),

            TokenKind::Identifier(ref val) => val.clone(),

            _ => self.kind.name().into(),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntBase {
    Bin,
    Dec,
    Hex,
}

impl IntBase {
    pub fn num(self) -> u32 {
        match self {
            IntBase::Bin => 2,
            IntBase::Dec => 10,
            IntBase::Hex => 16,
        }
    }
}
