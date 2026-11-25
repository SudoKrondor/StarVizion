//! Splits an expression's source text into tokens, following the lexical rules in the design doc's
//! Language Rules.

use crate::color::Color;
use crate::error::{ParseError, ParseErrorKind, Span};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    Number(f64),
    Bool(bool),
    Color(Color),
    Text(String),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Bang,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,
    Question,
    Colon,
    Comma,
    LeftParen,
    RightParen,
}

impl TokenKind {
    /// How the token is named in an error message.
    pub(crate) fn describe(&self) -> &'static str {
        match self {
            Self::Number(_) => "a number",
            Self::Bool(_) => "a boolean",
            Self::Color(_) => "a color",
            Self::Text(_) => "text",
            Self::Name(_) => "a name",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Star => "`*`",
            Self::Slash => "`/`",
            Self::Percent => "`%`",
            Self::Caret => "`^`",
            Self::Bang => "`!`",
            Self::EqualEqual => "`==`",
            Self::BangEqual => "`!=`",
            Self::Less => "`<`",
            Self::LessEqual => "`<=`",
            Self::Greater => "`>`",
            Self::GreaterEqual => "`>=`",
            Self::AndAnd => "`&&`",
            Self::OrOr => "`||`",
            Self::Question => "`?`",
            Self::Colon => "`:`",
            Self::Comma => "`,`",
            Self::LeftParen => "`(`",
            Self::RightParen => "`)`",
        }
    }
}

pub(crate) fn tokenize(source: &str) -> Result<Vec<Token>, ParseError> {
    let mut lexer = Lexer { source, pos: 0 };
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token()? {
        tokens.push(token);
    }
    Ok(tokens)
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

impl Lexer<'_> {
    fn next_token(&mut self) -> Result<Option<Token>, ParseError> {
        self.skip_whitespace();
        let start = self.pos;
        let Some(c) = self.peek() else {
            return Ok(None);
        };
        let starts_number = c.is_ascii_digit()
            || (c == '.' && self.peek_second().is_some_and(|d| d.is_ascii_digit()));
        let kind = match c {
            _ if starts_number => self.number()?,
            _ if is_name_start(c) => self.name(),
            '#' => self.color()?,
            '"' => self.text()?,
            _ => self.operator(c)?,
        };
        Ok(Some(Token {
            kind,
            span: Span::new(start, self.pos),
        }))
    }

    fn skip_whitespace(&mut self) {
        // WHY: `\r` is whitespace so expressions pasted from Windows, with CRLF line endings, parse the same.
        while let Some(c) = self
            .peek()
            .filter(|c| matches!(c, ' ' | '\t' | '\n' | '\r'))
        {
            self.advance(c);
        }
    }

    fn number(&mut self) -> Result<TokenKind, ParseError> {
        let start = self.pos;
        self.eat_digits();
        if self.peek() == Some('.') {
            self.pos += 1;
            if self.eat_digits() == 0 {
                return Err(self.malformed_number(start));
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }
            if self.eat_digits() == 0 {
                return Err(self.malformed_number(start));
            }
        }
        if self.peek().is_some_and(|c| is_name_char(c) || c == '.') {
            return Err(self.malformed_number(start));
        }
        let value: f64 = self.source[start..self.pos]
            .parse()
            .expect("the lexer only accepts digits that form a valid f64 literal");
        if value.is_infinite() {
            return Err(self.error(ParseErrorKind::NumberTooLarge, start));
        }
        Ok(TokenKind::Number(value))
    }

    /// Takes the rest of a malformed number, so the error marks all of it.
    fn malformed_number(&mut self, start: usize) -> ParseError {
        while let Some(c) = self.peek().filter(|&c| is_name_char(c) || c == '.') {
            self.advance(c);
        }
        self.error(ParseErrorKind::MalformedNumber, start)
    }

    fn name(&mut self) -> TokenKind {
        let start = self.pos;
        self.eat_name_chars();
        match &self.source[start..self.pos] {
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            name => TokenKind::Name(name.to_owned()),
        }
    }

    fn color(&mut self) -> Result<TokenKind, ParseError> {
        let start = self.pos;
        self.pos += 1;
        let digits_start = self.pos;
        self.eat_name_chars();
        match Color::from_hex(&self.source[digits_start..self.pos]) {
            Some(color) => Ok(TokenKind::Color(color)),
            None => Err(self.error(ParseErrorKind::InvalidColor, start)),
        }
    }

    fn text(&mut self) -> Result<TokenKind, ParseError> {
        let start = self.pos;
        self.pos += 1;
        let mut value = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(self.error(ParseErrorKind::UnterminatedText, start));
            };
            let char_start = self.pos;
            self.advance(c);
            match c {
                '"' => return Ok(TokenKind::Text(value)),
                '\n' | '\r' => return Err(self.error(ParseErrorKind::LineBreakInText, char_start)),
                '\\' => {
                    // WHY: a backslash at the very end falls through to the unterminated-text error above.
                    let Some(escaped) = self.peek() else { continue };
                    self.advance(escaped);
                    let Some(unescaped) = unescape(escaped) else {
                        return Err(self.error(ParseErrorKind::UnknownEscape(escaped), char_start));
                    };
                    value.push(unescaped);
                }
                _ => value.push(c),
            }
        }
    }

    fn operator(&mut self, c: char) -> Result<TokenKind, ParseError> {
        let start = self.pos;
        self.advance(c);
        let pair = |second: char, kind: TokenKind| (Some(second), kind);
        let (second, kind) = match (c, self.peek()) {
            ('/', Some('/' | '*')) => {
                self.pos += 1;
                return Err(self.error(ParseErrorKind::CommentNotSupported, start));
            }
            ('=', Some('=')) => pair('=', TokenKind::EqualEqual),
            ('!', Some('=')) => pair('=', TokenKind::BangEqual),
            ('<', Some('=')) => pair('=', TokenKind::LessEqual),
            ('>', Some('=')) => pair('=', TokenKind::GreaterEqual),
            ('&', Some('&')) => pair('&', TokenKind::AndAnd),
            ('|', Some('|')) => pair('|', TokenKind::OrOr),
            ('=', _) => return Err(self.lone_operator(c, "==", start)),
            ('&', _) => return Err(self.lone_operator(c, "&&", start)),
            ('|', _) => return Err(self.lone_operator(c, "||", start)),
            ('+', _) => (None, TokenKind::Plus),
            ('-', _) => (None, TokenKind::Minus),
            ('*', _) => (None, TokenKind::Star),
            ('/', _) => (None, TokenKind::Slash),
            ('%', _) => (None, TokenKind::Percent),
            ('^', _) => (None, TokenKind::Caret),
            ('!', _) => (None, TokenKind::Bang),
            ('<', _) => (None, TokenKind::Less),
            ('>', _) => (None, TokenKind::Greater),
            ('?', _) => (None, TokenKind::Question),
            (':', _) => (None, TokenKind::Colon),
            (',', _) => (None, TokenKind::Comma),
            ('(', _) => (None, TokenKind::LeftParen),
            (')', _) => (None, TokenKind::RightParen),
            _ => return Err(self.error(ParseErrorKind::UnexpectedCharacter(c), start)),
        };
        if let Some(second) = second {
            self.advance(second);
        }
        Ok(kind)
    }

    fn lone_operator(&self, found: char, suggestion: &'static str, start: usize) -> ParseError {
        self.error(ParseErrorKind::LoneOperator { found, suggestion }, start)
    }

    /// An error spanning from `start` to the current position.
    fn error(&self, kind: ParseErrorKind, start: usize) -> ParseError {
        ParseError::new(kind, Span::new(start, self.pos))
    }

    fn eat_digits(&mut self) -> usize {
        let start = self.pos;
        while let Some(c) = self.peek().filter(char::is_ascii_digit) {
            self.advance(c);
        }
        self.pos - start
    }

    fn eat_name_chars(&mut self) {
        while let Some(c) = self.peek().filter(|&c| is_name_char(c)) {
            self.advance(c);
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_second(&self) -> Option<char> {
        self.source[self.pos..].chars().nth(1)
    }

    fn advance(&mut self, c: char) {
        self.pos += c.len_utf8();
    }
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn unescape(escaped: char) -> Option<char> {
    match escaped {
        '"' => Some('"'),
        '\\' => Some('\\'),
        'n' => Some('\n'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source)
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    fn lex_error(source: &str) -> (ParseErrorKind, Span) {
        let error = tokenize(source).unwrap_err();
        (error.kind().clone(), error.span())
    }

    fn name(name: &str) -> TokenKind {
        TokenKind::Name(name.to_owned())
    }

    #[test]
    fn number_forms() {
        let cases = [
            ("1", 1.0),
            ("1.5", 1.5),
            (".5", 0.5),
            ("1e3", 1000.0),
            ("1.5e-3", 0.0015),
            ("2E+2", 200.0),
        ];
        for (source, value) in cases {
            assert_eq!(kinds(source), [TokenKind::Number(value)], "{source}");
        }
    }

    #[test]
    fn malformed_numbers_mark_the_whole_number() {
        let cases = [
            ("1.", 0, 2),
            ("1e", 0, 2),
            ("1e+", 0, 3),
            ("1_000", 0, 5),
            ("x + 2pi", 4, 7),
            ("1.5.2", 0, 5),
        ];
        for (source, start, end) in cases {
            assert_eq!(
                lex_error(source),
                (ParseErrorKind::MalformedNumber, Span::new(start, end)),
                "{source}"
            );
        }
    }

    #[test]
    fn a_number_that_overflows_is_too_large() {
        assert_eq!(
            lex_error("1e999"),
            (ParseErrorKind::NumberTooLarge, Span::new(0, 5))
        );
    }

    #[test]
    fn names_are_case_sensitive_and_only_lowercase_true_false_are_booleans() {
        assert_eq!(
            kinds("Throttle throttle _a1 true false True"),
            [
                name("Throttle"),
                name("throttle"),
                name("_a1"),
                TokenKind::Bool(true),
                TokenKind::Bool(false),
                name("True")
            ]
        );
    }

    #[test]
    fn colors() {
        let orange = Color {
            r: 0xFF,
            g: 0x88,
            b: 0x00,
            a: 0xFF,
        };
        let translucent = Color { a: 0x80, ..orange };
        assert_eq!(
            kinds("#FF8800 #ff880080"),
            [TokenKind::Color(orange), TokenKind::Color(translucent)]
        );
    }

    #[test]
    fn invalid_colors_mark_the_whole_color() {
        for (source, end) in [("#FFF", 4), ("#12345G", 7), ("#FF8800FF00", 11), ("#", 1)] {
            assert_eq!(
                lex_error(source),
                (ParseErrorKind::InvalidColor, Span::new(0, end)),
                "{source}"
            );
        }
    }

    #[test]
    fn text_with_escapes() {
        assert_eq!(
            kinds(r#""Speed: " "say \"hi\"\n\\""#),
            [
                TokenKind::Text("Speed: ".to_owned()),
                TokenKind::Text("say \"hi\"\n\\".to_owned())
            ]
        );
    }

    #[test]
    fn text_errors() {
        let cases = [
            (
                r#""open"#,
                ParseErrorKind::UnterminatedText,
                Span::new(0, 5),
            ),
            (
                r#""ends in \"#,
                ParseErrorKind::UnterminatedText,
                Span::new(0, 10),
            ),
            (
                r#""a\tb""#,
                ParseErrorKind::UnknownEscape('t'),
                Span::new(2, 4),
            ),
            (
                "\"one\ntwo\"",
                ParseErrorKind::LineBreakInText,
                Span::new(4, 5),
            ),
            (
                "\"one\r\ntwo\"",
                ParseErrorKind::LineBreakInText,
                Span::new(4, 5),
            ),
        ];
        for (source, kind, span) in cases {
            assert_eq!(lex_error(source), (kind, span), "{source:?}");
        }
    }

    #[test]
    fn crlf_and_tabs_between_tokens_are_whitespace() {
        assert_eq!(kinds("a\r\n+\tb"), [name("a"), TokenKind::Plus, name("b")]);
    }

    #[test]
    fn every_operator() {
        use TokenKind::*;
        assert_eq!(
            kinds("+ - * / % ^ ! == != < <= > >= && || ? : , ( )"),
            [
                Plus,
                Minus,
                Star,
                Slash,
                Percent,
                Caret,
                Bang,
                EqualEqual,
                BangEqual,
                Less,
                LessEqual,
                Greater,
                GreaterEqual,
                AndAnd,
                OrOr,
                Question,
                Colon,
                Comma,
                LeftParen,
                RightParen
            ]
        );
    }

    #[test]
    fn operators_need_no_spaces() {
        assert_eq!(
            kinds("a<=-b"),
            [name("a"), TokenKind::LessEqual, TokenKind::Minus, name("b")]
        );
    }

    #[test]
    fn lone_operator_characters_suggest_the_operator() {
        let cases = [
            ("a & b", '&', "&&"),
            ("a | b", '|', "||"),
            ("a = b", '=', "=="),
        ];
        for (source, found, suggestion) in cases {
            assert_eq!(
                lex_error(source),
                (
                    ParseErrorKind::LoneOperator { found, suggestion },
                    Span::new(2, 3)
                ),
                "{source}"
            );
        }
    }

    #[test]
    fn comments_are_not_supported() {
        for source in ["a // note", "a /* note */"] {
            assert_eq!(
                lex_error(source),
                (ParseErrorKind::CommentNotSupported, Span::new(2, 4)),
                "{source}"
            );
        }
    }

    #[test]
    fn unexpected_characters_including_non_ascii() {
        assert_eq!(
            lex_error("a @ b"),
            (ParseErrorKind::UnexpectedCharacter('@'), Span::new(2, 3))
        );
        assert_eq!(
            lex_error("é"),
            (ParseErrorKind::UnexpectedCharacter('é'), Span::new(0, 2))
        );
    }

    #[test]
    fn spans_are_byte_ranges() {
        let spans: Vec<Span> = tokenize("ab + 1.5")
            .unwrap()
            .into_iter()
            .map(|t| t.span)
            .collect();
        assert_eq!(spans, [Span::new(0, 2), Span::new(3, 4), Span::new(5, 8)]);
    }
}
