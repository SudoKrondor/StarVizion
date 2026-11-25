//! Errors from parsing an expression, each pointing at the text that caused it.

use std::fmt;

/// A byte range in an expression's source text: where the editor draws its error marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub(crate) fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    /// The span from the start of `self` to the end of `other`.
    pub(crate) fn to(self, other: Span) -> Span {
        Span::new(self.start, other.end)
    }
}

/// Why an expression could not be parsed, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: Span,
}

impl ParseError {
    pub(crate) fn new(kind: ParseErrorKind, span: Span) -> ParseError {
        ParseError { kind, span }
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl std::error::Error for ParseError {}

/// The kinds of parse error. Each displays as a message the editor can show beside the field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The expression has nothing in it but whitespace.
    EmptyExpression,
    /// A character that has no meaning in StarCalc.
    UnexpectedCharacter(char),
    /// Half of a two-character operator, such as `&` for `&&`.
    LoneOperator {
        found: char,
        suggestion: &'static str,
    },
    /// `//` or `/*`: StarCalc has no comments yet.
    CommentNotSupported,
    /// Digits that do not form a number, such as `1.`, `1e` or `2x`.
    MalformedNumber,
    /// A number too large to represent.
    NumberTooLarge,
    /// A `#` not followed by exactly 6 or 8 hex digits.
    InvalidColor,
    /// Text with no closing quote.
    UnterminatedText,
    /// A raw line break inside quoted text.
    LineBreakInText,
    /// A backslash escape other than `\"`, `\\` or `\n`.
    UnknownEscape(char),
    /// A token where something else was expected.
    UnexpectedToken {
        found: &'static str,
        expected: &'static str,
    },
    /// The expression ended where something else was expected.
    UnexpectedEnd { expected: &'static str },
    /// Nesting deeper than StarCalc allows.
    TooDeep,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExpression => write!(f, "the expression is empty"),
            Self::UnexpectedCharacter(c) => write!(f, "`{c}` is not part of StarCalc"),
            Self::LoneOperator { found, suggestion } => {
                write!(
                    f,
                    "`{found}` is not an operator; did you mean `{suggestion}`?"
                )
            }
            Self::CommentNotSupported => write!(f, "comments are not supported"),
            Self::MalformedNumber => write!(f, "this is not a valid number"),
            Self::NumberTooLarge => write!(f, "this number is too large"),
            Self::InvalidColor => {
                write!(
                    f,
                    "a color is `#` followed by 6 or 8 hex digits, like `#FF8800`"
                )
            }
            Self::UnterminatedText => write!(f, "text is missing its closing `\"`"),
            Self::LineBreakInText => {
                write!(f, "text cannot contain a line break; use `\\n` instead")
            }
            Self::UnknownEscape(c) => {
                write!(f, "`\\{c}` is not an escape; use `\\\"`, `\\\\` or `\\n`")
            }
            Self::UnexpectedToken { found, expected } => {
                write!(f, "expected {expected}, found {found}")
            }
            Self::UnexpectedEnd { expected } => {
                write!(f, "expected {expected}, found the end of the expression")
            }
            Self::TooDeep => write!(f, "the expression is nested too deeply"),
        }
    }
}
