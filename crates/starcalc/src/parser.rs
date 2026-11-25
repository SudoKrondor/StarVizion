//! Builds an expression's syntax tree from its tokens, following the precedence table in the design
//! doc's Language Rules.

use crate::color::Color;
use crate::error::{ParseError, ParseErrorKind, Span};
use crate::lexer::{self, Token, TokenKind};

/// How deep the parser may recurse: each parenthesis, call argument, conditional branch, unary
/// operator and exponent counts as one level.
// WHY: Vizlet files can be imported from other people, and unbounded nesting such as `((((...` would
// overflow the stack and take StarVizion down. No hand-written expression comes near this limit.
const MAX_DEPTH: usize = 64;

/// Parses one StarCalc expression into its syntax tree. Names are not resolved here.
///
/// # Errors
///
/// Returns a [`ParseError`] whose span marks the offending text when `source` is empty, contains
/// text StarCalc does not recognise, is not one complete expression, or nests too deeply.
pub fn parse(source: &str) -> Result<Expr, ParseError> {
    let tokens = lexer::tokenize(source)?;
    if tokens.is_empty() {
        let whole = Span::new(0, source.len());
        return Err(ParseError::new(ParseErrorKind::EmptyExpression, whole));
    }
    let mut parser = Parser {
        tokens,
        pos: 0,
        depth: 0,
        end: source.len(),
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(unexpected(
            token,
            "an operator or the end of the expression",
        ));
    }
    Ok(expr)
}

/// A parsed expression and the part of the source text it covers.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Number(f64),
    Bool(bool),
    Color(Color),
    Text(String),
    /// A binding, local, constant, or window or time variable. Which one is resolved later.
    Name(String),
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `condition ? if_true : if_false`
    Conditional {
        condition: Box<Expr>,
        if_true: Box<Expr>,
        if_false: Box<Expr>,
    },
    Call {
        name: String,
        name_span: Span,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Power,
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

type ParseResult = Result<Expr, ParseError>;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
    /// The source length, where an "unexpected end" error points.
    end: usize,
}

impl Parser {
    fn expression(&mut self) -> ParseResult {
        self.nested(Self::conditional)
    }

    fn conditional(&mut self) -> ParseResult {
        let condition = self.or()?;
        if !self.eat(&TokenKind::Question) {
            return Ok(condition);
        }
        let if_true = self.expression()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        let if_false = self.expression()?;
        Ok(Expr {
            span: condition.span.to(if_false.span),
            kind: ExprKind::Conditional {
                condition: Box::new(condition),
                if_true: Box::new(if_true),
                if_false: Box::new(if_false),
            },
        })
    }

    fn or(&mut self) -> ParseResult {
        self.left_associative(Self::and, |kind| match kind {
            TokenKind::OrOr => Some(BinaryOp::Or),
            _ => None,
        })
    }

    fn and(&mut self) -> ParseResult {
        self.left_associative(Self::equality, |kind| match kind {
            TokenKind::AndAnd => Some(BinaryOp::And),
            _ => None,
        })
    }

    fn equality(&mut self) -> ParseResult {
        self.left_associative(Self::comparison, |kind| match kind {
            TokenKind::EqualEqual => Some(BinaryOp::Equal),
            TokenKind::BangEqual => Some(BinaryOp::NotEqual),
            _ => None,
        })
    }

    fn comparison(&mut self) -> ParseResult {
        self.left_associative(Self::additive, |kind| match kind {
            TokenKind::Less => Some(BinaryOp::Less),
            TokenKind::LessEqual => Some(BinaryOp::LessOrEqual),
            TokenKind::Greater => Some(BinaryOp::Greater),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterOrEqual),
            _ => None,
        })
    }

    fn additive(&mut self) -> ParseResult {
        self.left_associative(Self::multiplicative, |kind| match kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Subtract),
            _ => None,
        })
    }

    fn multiplicative(&mut self) -> ParseResult {
        self.left_associative(Self::unary, |kind| match kind {
            TokenKind::Star => Some(BinaryOp::Multiply),
            TokenKind::Slash => Some(BinaryOp::Divide),
            TokenKind::Percent => Some(BinaryOp::Remainder),
            _ => None,
        })
    }

    /// One precedence level of left-associative binary operators: `operand (op operand)*`.
    fn left_associative(
        &mut self,
        operand: fn(&mut Self) -> ParseResult,
        operator: fn(&TokenKind) -> Option<BinaryOp>,
    ) -> ParseResult {
        let mut left = operand(self)?;
        while let Some(op) = self.peek().and_then(|token| operator(&token.kind)) {
            self.pos += 1;
            let right = operand(self)?;
            left = binary(op, left, right);
        }
        Ok(left)
    }

    fn unary(&mut self) -> ParseResult {
        let op = match self.peek().map(|token| &token.kind) {
            Some(TokenKind::Minus) => UnaryOp::Negate,
            Some(TokenKind::Bang) => UnaryOp::Not,
            _ => return self.power(),
        };
        let op_span = self.tokens[self.pos].span;
        self.pos += 1;
        let operand = self.nested(Self::unary)?;
        Ok(Expr {
            span: op_span.to(operand.span),
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
        })
    }

    /// `^` binds tighter than unary minus and is right-associative: `-2^2` is `-(2^2)`, `2^3^2` is
    /// `2^(3^2)`, and the exponent may carry its own sign, as in `2^-1`.
    fn power(&mut self) -> ParseResult {
        let base = self.primary()?;
        if !self.eat(&TokenKind::Caret) {
            return Ok(base);
        }
        let exponent = self.nested(Self::unary)?;
        Ok(binary(BinaryOp::Power, base, exponent))
    }

    fn primary(&mut self) -> ParseResult {
        let Some(token) = self.tokens.get(self.pos).cloned() else {
            return Err(self.unexpected_end("a value"));
        };
        self.pos += 1;
        let kind = match token.kind {
            TokenKind::Number(value) => ExprKind::Number(value),
            TokenKind::Bool(value) => ExprKind::Bool(value),
            TokenKind::Color(color) => ExprKind::Color(color),
            TokenKind::Text(text) => ExprKind::Text(text),
            TokenKind::Name(name) if self.at(&TokenKind::LeftParen) => {
                return self.call(name, token.span);
            }
            TokenKind::Name(name) => ExprKind::Name(name),
            TokenKind::LeftParen => return self.parenthesized(token.span),
            _ => return Err(unexpected(&token, "a value")),
        };
        Ok(Expr {
            kind,
            span: token.span,
        })
    }

    fn parenthesized(&mut self, open: Span) -> ParseResult {
        let inner = self.expression()?;
        let close = self.expect(&TokenKind::RightParen, "`)`")?;
        Ok(Expr {
            span: open.to(close),
            ..inner
        })
    }

    fn call(&mut self, name: String, name_span: Span) -> ParseResult {
        self.pos += 1;
        let mut args = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            loop {
                args.push(self.expression()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(&TokenKind::RightParen, "`,` or `)`")?;
        Ok(Expr {
            span: name_span.to(close),
            kind: ExprKind::Call {
                name,
                name_span,
                args,
            },
        })
    }

    /// Runs `parse` one nesting level deeper, failing instead once the limit is reached.
    fn nested(&mut self, parse: fn(&mut Self) -> ParseResult) -> ParseResult {
        if self.depth == MAX_DEPTH {
            let span = self
                .peek()
                .map_or(Span::new(self.end, self.end), |token| token.span);
            return Err(ParseError::new(ParseErrorKind::TooDeep, span));
        }
        self.depth += 1;
        let result = parse(self);
        self.depth -= 1;
        result
    }

    fn expect(&mut self, kind: &TokenKind, expected: &'static str) -> Result<Span, ParseError> {
        let Some(token) = self.peek() else {
            return Err(self.unexpected_end(expected));
        };
        if &token.kind != kind {
            return Err(unexpected(token, expected));
        }
        let span = token.span;
        self.pos += 1;
        Ok(span)
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        self.pos += 1;
        true
    }

    fn at(&self, kind: &TokenKind) -> bool {
        self.peek().is_some_and(|token| &token.kind == kind)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn unexpected_end(&self, expected: &'static str) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnexpectedEnd { expected },
            Span::new(self.end, self.end),
        )
    }
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr {
        span: left.span.to(right.span),
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}

fn unexpected(token: &Token, expected: &'static str) -> ParseError {
    let found = token.kind.describe();
    ParseError::new(
        ParseErrorKind::UnexpectedToken { found, expected },
        token.span,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tree as an S-expression, so precedence reads at a glance.
    fn show(expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Number(value) => value.to_string(),
            ExprKind::Bool(value) => value.to_string(),
            ExprKind::Color(c) => format!("#{:02X}{:02X}{:02X}{:02X}", c.r, c.g, c.b, c.a),
            ExprKind::Text(text) => format!("{text:?}"),
            ExprKind::Name(name) => name.clone(),
            ExprKind::Unary { op, operand } => format!("({} {})", unary_symbol(*op), show(operand)),
            ExprKind::Binary { op, left, right } => {
                format!("({} {} {})", binary_symbol(*op), show(left), show(right))
            }
            ExprKind::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                format!(
                    "(? {} {} {})",
                    show(condition),
                    show(if_true),
                    show(if_false)
                )
            }
            ExprKind::Call { name, args, .. } => {
                let args: Vec<String> = args.iter().map(show).collect();
                format!("({name} {})", args.join(" ")).replace(" )", ")")
            }
        }
    }

    fn unary_symbol(op: UnaryOp) -> &'static str {
        match op {
            UnaryOp::Negate => "neg",
            UnaryOp::Not => "!",
        }
    }

    fn binary_symbol(op: BinaryOp) -> &'static str {
        match op {
            BinaryOp::Power => "^",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Remainder => "%",
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Less => "<",
            BinaryOp::LessOrEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterOrEqual => ">=",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }

    fn assert_parses(cases: &[(&str, &str)]) {
        for (source, expected) in cases {
            assert_eq!(show(&parse(source).unwrap()), *expected, "{source}");
        }
    }

    fn parse_error(source: &str) -> (ParseErrorKind, Span) {
        let error = parse(source).unwrap_err();
        (error.kind().clone(), error.span())
    }

    #[test]
    fn power_is_tightest_and_right_associative() {
        assert_parses(&[
            ("2^3^2", "(^ 2 (^ 3 2))"),
            ("-2^2", "(neg (^ 2 2))"),
            ("2^-1", "(^ 2 (neg 1))"),
            ("a * b^2", "(* a (^ b 2))"),
        ]);
    }

    #[test]
    fn unary_operators_nest_and_bind_tighter_than_multiplication() {
        assert_parses(&[
            ("--x", "(neg (neg x))"),
            ("!!a", "(! (! a))"),
            ("-x * 3", "(* (neg x) 3)"),
        ]);
    }

    #[test]
    fn multiplicative_and_additive_levels_are_left_associative() {
        assert_parses(&[
            ("a / b % c * d", "(* (% (/ a b) c) d)"),
            ("a - b - c", "(- (- a b) c)"),
            ("a + b * c", "(+ a (* b c))"),
            ("a * b + c", "(+ (* a b) c)"),
        ]);
    }

    #[test]
    fn comparison_binds_tighter_than_equality() {
        assert_parses(&[
            ("a + b < c", "(< (+ a b) c)"),
            ("a < b == c >= d", "(== (< a b) (>= c d))"),
            ("a <= b > c", "(> (<= a b) c)"),
            ("a != b == c", "(== (!= a b) c)"),
        ]);
    }

    #[test]
    fn and_binds_tighter_than_or() {
        assert_parses(&[
            ("a == b && c", "(&& (== a b) c)"),
            ("a || b && c", "(|| a (&& b c))"),
            ("a || b || c", "(|| (|| a b) c)"),
            ("!a && b", "(&& (! a) b)"),
        ]);
    }

    #[test]
    fn conditional_is_loosest_and_right_associative() {
        assert_parses(&[
            ("a || b ? c : d", "(? (|| a b) c d)"),
            ("a ? b : c ? d : e", "(? a b (? c d e))"),
            ("a ? b ? c : d : e", "(? a (? b c d) e)"),
            ("fire ? #FF0000 : #00FF00", "(? fire #FF0000FF #00FF00FF)"),
        ]);
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_parses(&[
            ("(a + b) * c", "(* (+ a b) c)"),
            ("(-2)^2", "(^ (neg 2) 2)"),
            ("((a))", "a"),
        ]);
    }

    #[test]
    fn function_calls() {
        assert_parses(&[
            ("clamp(x, 0, 1)", "(clamp x 0 1)"),
            ("f()", "(f)"),
            ("max(sin(t), -1) * 2", "(* (max (sin t) (neg 1)) 2)"),
        ]);
    }

    #[test]
    fn literals() {
        assert_parses(&[
            ("1.5", "1.5"),
            ("true", "true"),
            ("#FF880080", "#FF880080"),
            (
                r#""Speed: " + format(speed, 1)"#,
                r#"(+ "Speed: " (format speed 1))"#,
            ),
        ]);
    }

    #[test]
    fn spans_cover_the_source_text() {
        let expr = parse("(a + bb) * f(c)").unwrap();
        assert_eq!(expr.span, Span::new(0, 15));
        let ExprKind::Binary { left, right, .. } = &expr.kind else {
            panic!("expected a binary expression, got {expr:?}");
        };
        assert_eq!(left.span, Span::new(0, 8));
        let ExprKind::Call { name_span, .. } = &right.kind else {
            panic!("expected a call, got {right:?}");
        };
        assert_eq!(
            (right.span, *name_span),
            (Span::new(11, 15), Span::new(11, 12))
        );
    }

    #[test]
    fn empty_expressions() {
        assert_eq!(
            parse_error(""),
            (ParseErrorKind::EmptyExpression, Span::new(0, 0))
        );
        assert_eq!(
            parse_error(" \r\n"),
            (ParseErrorKind::EmptyExpression, Span::new(0, 3))
        );
    }

    #[test]
    fn incomplete_expressions_point_at_the_end() {
        let cases = [
            ("1 +", "a value"),
            ("(a", "`)`"),
            ("a ? b", "`:`"),
            ("f(a,", "a value"),
            ("f(a", "`,` or `)`"),
        ];
        for (source, expected) in cases {
            let end = source.len();
            assert_eq!(
                parse_error(source),
                (
                    ParseErrorKind::UnexpectedEnd { expected },
                    Span::new(end, end)
                ),
                "{source}"
            );
        }
    }

    #[test]
    fn unexpected_tokens_point_at_the_token() {
        let cases = [
            (
                "a b",
                "a name",
                "an operator or the end of the expression",
                2,
                3,
            ),
            (")", "`)`", "a value", 0, 1),
            ("f(a b)", "a name", "`,` or `)`", 4, 5),
            ("a ? b , c", "`,`", "`:`", 6, 7),
            ("1 + * 2", "`*`", "a value", 4, 5),
            ("f(a,)", "`)`", "a value", 4, 5),
        ];
        for (source, found, expected, start, end) in cases {
            assert_eq!(
                parse_error(source),
                (
                    ParseErrorKind::UnexpectedToken { found, expected },
                    Span::new(start, end)
                ),
                "{source}"
            );
        }
    }

    #[test]
    fn lexer_errors_pass_through() {
        assert_eq!(parse_error("a & b").1, Span::new(2, 3));
    }

    #[test]
    fn reasonable_nesting_parses() {
        let source = format!("{}x{}", "(".repeat(30), ")".repeat(30));
        assert_parses(&[(&source, "x")]);
    }

    #[test]
    fn deep_nesting_fails_instead_of_overflowing_the_stack() {
        for source in [
            format!("{}x", "(".repeat(10_000)),
            format!("{}x", "-".repeat(10_000)),
            "2^".repeat(10_000) + "2",
        ] {
            assert_eq!(parse_error(&source).0, ParseErrorKind::TooDeep);
        }
    }

    #[test]
    fn errors_display_as_editor_messages() {
        let error = parse("a = b").unwrap_err();
        assert_eq!(
            error.to_string(),
            "`=` is not an operator; did you mean `==`?"
        );
        let error = parse("1 +").unwrap_err();
        assert_eq!(
            error.to_string(),
            "expected a value, found the end of the expression"
        );
    }
}
