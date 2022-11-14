use crate::lexer::position::Position;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    // Lexer errors
    UnknownChar(char),
    UnclosedComment,
    UnclosedString,
    UnclosedChar,
    InvalidEscapeSequence(char),

    // Parser errors
    ExpectedTopLevelElement(String),
    UnknownAnnotation(String),
    RedundantAnnotation(String),
    MisplacedAnnotation(String),
    ExpectedToken(String, String),
    ExpectedType(String),
    MisplacedElse,
    ExpectedFactor(String),
    NumberOverflow,
    UnclosedStringTemplate,
    ExpectedIdentifier(String),
    InvalidSuffix(String),
}

impl ParseError {
    pub fn message(&self) -> String {
        match self {
            ParseError::UnknownChar(ch) => {
                format!("unknown character {} (codepoint {}).", ch, *ch as usize)
            }
            ParseError::UnclosedComment => "unclosed comment.".into(),
            ParseError::UnclosedString => "unclosed string.".into(),
            ParseError::UnclosedChar => "unclosed char.".into(),
            ParseError::InvalidEscapeSequence(ch) => format!("unknown escape sequence `\\{}`.", ch),

            // Parser errors
            ParseError::ExpectedTopLevelElement(ref token) => {
                format!(
                    "expected a top-level element (`class`, `struct`, `trait`, `impl`, `enum`, `fun`, `let` or `var`), but got {}.",
                    token
                )
            }
            ParseError::MisplacedAnnotation(ref modifier) => {
                format!("misplaced annotation `{}`.", modifier)
            }
            ParseError::RedundantAnnotation(ref token) => {
                format!("redundant annotation {}.", token)
            }
            ParseError::UnknownAnnotation(ref token) => format!("unknown annotation {}.", token),
            ParseError::ExpectedToken(ref exp, ref got) => {
                format!("expected {} but got {}.", exp, got)
            }
            ParseError::NumberOverflow => "number too large to be represented.".into(),
            ParseError::ExpectedType(ref got) => format!("type expected but got {}.", got),
            ParseError::MisplacedElse => "misplace else.".into(),
            ParseError::ExpectedFactor(ref got) => format!("factor expected but got {}.", got),
            ParseError::UnclosedStringTemplate => "unclosed string template.".into(),
            ParseError::ExpectedIdentifier(ref tok) => {
                format!("identifier expected but got {}.", tok)
            }
            ParseError::InvalidSuffix(ref suffix) => format!("invalid suffix `{}`", suffix),
        }
    }
}

#[derive(Debug)]
pub struct ParseErrorAndPos {
    pub pos: Position,
    pub error: ParseError,
}

impl ParseErrorAndPos {
    pub fn new(pos: Position, error: ParseError) -> ParseErrorAndPos {
        ParseErrorAndPos { pos, error }
    }
}
