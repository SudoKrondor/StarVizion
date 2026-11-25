//! StarCalc: parse a NeuroNode expression once, evaluate it every frame against input, window
//! and time variables (and, for a Parametric primitive, the sweep parameter `t`).
//!
//! [`parse`] turns source text into an [`Expr`] syntax tree, or a [`ParseError`] whose [`Span`]
//! marks the text the editor should flag.

#![forbid(unsafe_code)]

mod color;
mod error;
mod lexer;
mod parser;

pub use color::Color;
pub use error::{ParseError, ParseErrorKind, Span};
pub use parser::{BinaryOp, Expr, ExprKind, UnaryOp, parse};
