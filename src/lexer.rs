use crate::diagnostic::{Diagnostic, Result, SourceSpan};

const PHP_BINARY_BYTE_SENTINEL_BASE: u32 = 0xE000;

pub(crate) fn decode_php_source_bytes(bytes: &[u8]) -> String {
    let mut source = String::with_capacity(bytes.len());
    for &byte in bytes {
        push_php_string_byte(&mut source, byte);
    }
    source
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenTag,
    CloseTag,
    InlineHtml(String),
    Echo,
    Print,
    If,
    Elseif,
    Else,
    Do,
    While,
    For,
    Foreach,
    As,
    Switch,
    Match,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
    Try,
    Catch,
    Throw,
    Yield,
    Goto,
    Const,
    Function,
    Global,
    New,
    Clone,
    Identifier(String),
    String(String),
    BacktickString(String),
    InterpolatedString(Vec<StringPart>),
    Int(i64),
    Float(f64),
    True,
    False,
    Null,
    Variable(String),
    Dollar,
    Equal,
    DoubleArrow,
    DoubleColon,
    QuestionQuestion,
    QuestionQuestionEqual,
    EqualEqual,
    EqualEqualEqual,
    NotEqual,
    NotEqualEqual,
    Spaceship,
    Less,
    LessEqual,
    ShiftLeft,
    ShiftLeftEqual,
    Greater,
    GreaterEqual,
    ShiftRight,
    ShiftRightEqual,
    KeywordAnd,
    KeywordOr,
    KeywordXor,
    AndAnd,
    OrOr,
    AmpersandEqual,
    PipeEqual,
    CaretEqual,
    PlusEqual,
    MinusEqual,
    PlusPlus,
    MinusMinus,
    ObjectOperator,
    NullsafeObjectOperator,
    AsteriskEqual,
    AsteriskAsteriskEqual,
    SlashEqual,
    PercentEqual,
    DotEqual,
    Plus,
    Minus,
    Asterisk,
    AsteriskAsterisk,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    PipeGreater,
    Caret,
    Tilde,
    Bang,
    At,
    AttributeStart,
    Backslash,
    Ellipsis,
    Dot,
    Comma,
    Question,
    Colon,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    IntType,
    IntegerType,
    FloatType,
    DoubleType,
    StringType,
    BinaryType,
    BoolType,
    BooleanType,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
    LegacyDollarBraceVariable(String),
    PropertyFetch {
        variable: String,
        property: String,
    },
    PropertyChain {
        variable: String,
        properties: Vec<String>,
    },
    MethodCall {
        variable: String,
        method: String,
    },
    ArrayAccess {
        array: String,
        indices: Vec<StringInterpolationIndex>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringInterpolationIndex {
    String(String),
    Int(i64),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

pub fn lex(source: &str) -> Result<Vec<Token>> {
    Lexer::new(source).lex()
}

struct Lexer<'a> {
    source: &'a str,
    cursor: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    seen_open_tag: bool,
    closed_php: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            seen_open_tag: false,
            closed_php: false,
        }
    }

    fn lex(mut self) -> Result<Vec<Token>> {
        while let Some(ch) = self.peek_char() {
            if self.closed_php {
                self.lex_inline_html()?;
                continue;
            }

            if !self.seen_open_tag {
                if self.cursor == 0 && self.rest().starts_with("#!") {
                    self.skip_line();
                    continue;
                }
                if self.rest().starts_with("<?php") {
                    self.push_open_tag();
                    continue;
                }
                if ch.is_whitespace() && self.leading_whitespace_reaches_open_tag_or_eof() {
                    self.bump_char();
                    continue;
                }
                self.lex_inline_html()?;
                continue;
            }

            match ch {
                '?' if self.rest().starts_with("?>") => self.push_close_tag(),
                '?' if self.rest().starts_with("??=") => {
                    self.push_fixed(TokenKind::QuestionQuestionEqual, 3)
                }
                '?' if self.rest().starts_with("??") => {
                    self.push_fixed(TokenKind::QuestionQuestion, 2)
                }
                '/' if self.rest().starts_with("//") => self.skip_line_comment(),
                '/' if self.rest().starts_with("/*") => self.skip_block_comment()?,
                '#' if self.rest().starts_with("#[") => {
                    self.push_fixed(TokenKind::AttributeStart, 2)
                }
                '#' => self.skip_line_comment(),
                c if c.is_whitespace() => self.bump_char(),
                ';' => self.push_fixed(TokenKind::Semicolon, 1),
                ',' => self.push_fixed(TokenKind::Comma, 1),
                '?' if self.rest().starts_with("?->") => {
                    self.push_fixed(TokenKind::NullsafeObjectOperator, 3)
                }
                '?' => self.push_fixed(TokenKind::Question, 1),
                ':' if self.rest().starts_with("::") => self.push_fixed(TokenKind::DoubleColon, 2),
                ':' => self.push_fixed(TokenKind::Colon, 1),
                '{' => self.push_fixed(TokenKind::LeftBrace, 1),
                '}' => self.push_fixed(TokenKind::RightBrace, 1),
                '[' => self.push_fixed(TokenKind::LeftBracket, 1),
                ']' => self.push_fixed(TokenKind::RightBracket, 1),
                '=' if self.rest().starts_with("===") => {
                    self.push_fixed(TokenKind::EqualEqualEqual, 3)
                }
                '=' if self.rest().starts_with("==") => self.push_fixed(TokenKind::EqualEqual, 2),
                '=' if self.rest().starts_with("=>") => self.push_fixed(TokenKind::DoubleArrow, 2),
                '=' => self.push_fixed(TokenKind::Equal, 1),
                '!' if self.rest().starts_with("!==") => {
                    self.push_fixed(TokenKind::NotEqualEqual, 3)
                }
                '!' if self.rest().starts_with("!=") => self.push_fixed(TokenKind::NotEqual, 2),
                '<' if self.rest().starts_with("<<<") => self.lex_heredoc_string()?,
                '<' if self.rest().starts_with("<=>") => self.push_fixed(TokenKind::Spaceship, 3),
                '<' if self.rest().starts_with("<<=") => {
                    self.push_fixed(TokenKind::ShiftLeftEqual, 3)
                }
                '<' if self.rest().starts_with("<>") => self.push_fixed(TokenKind::NotEqual, 2),
                '<' if self.rest().starts_with("<=") => self.push_fixed(TokenKind::LessEqual, 2),
                '<' if self.rest().starts_with("<<") => self.push_fixed(TokenKind::ShiftLeft, 2),
                '<' => self.push_fixed(TokenKind::Less, 1),
                '>' if self.rest().starts_with(">>=") => {
                    self.push_fixed(TokenKind::ShiftRightEqual, 3)
                }
                '>' if self.rest().starts_with(">=") => self.push_fixed(TokenKind::GreaterEqual, 2),
                '>' if self.rest().starts_with(">>") => self.push_fixed(TokenKind::ShiftRight, 2),
                '>' => self.push_fixed(TokenKind::Greater, 1),
                '&' if self.rest().starts_with("&&") => self.push_fixed(TokenKind::AndAnd, 2),
                '&' if self.rest().starts_with("&=") => {
                    self.push_fixed(TokenKind::AmpersandEqual, 2)
                }
                '&' => self.push_fixed(TokenKind::Ampersand, 1),
                '|' if self.rest().starts_with("||") => self.push_fixed(TokenKind::OrOr, 2),
                '|' if self.rest().starts_with("|=") => self.push_fixed(TokenKind::PipeEqual, 2),
                '|' if self.rest().starts_with("|>") => self.push_fixed(TokenKind::PipeGreater, 2),
                '|' => self.push_fixed(TokenKind::Pipe, 1),
                '^' if self.rest().starts_with("^=") => self.push_fixed(TokenKind::CaretEqual, 2),
                '^' => self.push_fixed(TokenKind::Caret, 1),
                '~' => self.push_fixed(TokenKind::Tilde, 1),
                '+' if self.rest().starts_with("+=") => self.push_fixed(TokenKind::PlusEqual, 2),
                '+' if self.rest().starts_with("++") => self.push_fixed(TokenKind::PlusPlus, 2),
                '+' => self.push_fixed(TokenKind::Plus, 1),
                '-' if self.rest().starts_with("-=") => self.push_fixed(TokenKind::MinusEqual, 2),
                '-' if self.rest().starts_with("--") => self.push_fixed(TokenKind::MinusMinus, 2),
                '-' if self.rest().starts_with("->") => {
                    self.push_fixed(TokenKind::ObjectOperator, 2)
                }
                '-' => self.push_fixed(TokenKind::Minus, 1),
                '*' if self.rest().starts_with("**=") => {
                    self.push_fixed(TokenKind::AsteriskAsteriskEqual, 3)
                }
                '*' if self.rest().starts_with("**") => {
                    self.push_fixed(TokenKind::AsteriskAsterisk, 2)
                }
                '*' if self.rest().starts_with("*=") => {
                    self.push_fixed(TokenKind::AsteriskEqual, 2)
                }
                '*' => self.push_fixed(TokenKind::Asterisk, 1),
                '/' if self.rest().starts_with("/=") => self.push_fixed(TokenKind::SlashEqual, 2),
                '/' => self.push_fixed(TokenKind::Slash, 1),
                '%' if self.rest().starts_with("%=") => self.push_fixed(TokenKind::PercentEqual, 2),
                '%' => self.push_fixed(TokenKind::Percent, 1),
                '!' => self.push_fixed(TokenKind::Bang, 1),
                '@' => self.push_fixed(TokenKind::At, 1),
                '\\' => self.push_fixed(TokenKind::Backslash, 1),
                '.' if self.rest().starts_with("...") => self.push_fixed(TokenKind::Ellipsis, 3),
                '.' if self.rest().starts_with(".=") => self.push_fixed(TokenKind::DotEqual, 2),
                '.' if self.starts_leading_dot_float() => self.lex_leading_dot_float()?,
                '.' => self.push_fixed(TokenKind::Dot, 1),
                '(' => self.push_fixed(TokenKind::LeftParen, 1),
                ')' => self.push_fixed(TokenKind::RightParen, 1),
                '$' => self.lex_variable()?,
                'b' | 'B'
                    if self.rest().starts_with("b'")
                        || self.rest().starts_with("B'")
                        || self.rest().starts_with("b\"")
                        || self.rest().starts_with("B\"") =>
                {
                    self.bump_char();
                    let quote = self.peek_char().expect("binary string prefix has quote");
                    self.lex_string(quote)?
                }
                '\'' | '"' => self.lex_string(ch)?,
                '`' => self.lex_backtick_string()?,
                c if c.is_ascii_digit() => self.lex_number()?,
                c if is_ident_start(c) => self.lex_word()?,
                _ => {
                    return Err(Diagnostic::new(
                        format!("unsupported PHP token {:?}", ch),
                        Some(self.current_char_span()),
                    ))
                }
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.current_span(0),
        });
        Ok(self.tokens)
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch == '\n' || self.rest().starts_with("?>") {
                break;
            }
            self.bump_char();
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let start = self.current_span(0);
        self.bump_char();
        self.bump_char();
        while self.peek_char().is_some() {
            if self.rest().starts_with("*/") {
                self.bump_char();
                self.bump_char();
                return Ok(());
            }
            self.bump_char();
        }
        Err(Diagnostic::parse_error(
            format!("Unterminated comment starting line {}", start.line),
            Some(start),
        ))
    }

    fn skip_line(&mut self) {
        while let Some(ch) = self.peek_char() {
            self.bump_char();
            if ch == '\n' {
                break;
            }
        }
    }

    fn lex_inline_html(&mut self) -> Result<()> {
        let start = self.current_span(0);
        let mut content = String::new();
        while let Some(ch) = self.peek_char() {
            if self.rest().starts_with("<?php") {
                if !content.is_empty() {
                    self.tokens.push(Token {
                        kind: TokenKind::InlineHtml(content),
                        span: SourceSpan::new(
                            start.byte_start,
                            self.cursor,
                            start.line,
                            start.column,
                        ),
                    });
                }
                self.push_open_tag();
                return Ok(());
            }
            content.push(ch);
            self.bump_char();
        }
        if !content.is_empty() {
            self.tokens.push(Token {
                kind: TokenKind::InlineHtml(content),
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
        }
        Ok(())
    }

    fn leading_whitespace_reaches_open_tag_or_eof(&self) -> bool {
        let mut cursor = self.cursor;
        while cursor < self.source.len() {
            let Some(ch) = self.source[cursor..].chars().next() else {
                break;
            };
            if !ch.is_whitespace() {
                break;
            }
            cursor += ch.len_utf8();
        }
        cursor == self.source.len() || self.source[cursor..].starts_with("<?php")
    }

    fn lex_string(&mut self, quote: char) -> Result<()> {
        if quote == '"' {
            return self.lex_double_quoted_string();
        }

        let start = self.current_span(0);
        self.bump_char();
        let mut value = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == quote {
                self.bump_char();
                self.tokens.push(Token {
                    kind: TokenKind::String(value),
                    span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
                });
                return Ok(());
            }
            if ch == '\\' {
                self.bump_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    Diagnostic::new("unterminated string escape", Some(self.current_span(0)))
                })?;
                match escaped {
                    'n' if quote == '"' => value.push('\n'),
                    'r' if quote == '"' => value.push('\r'),
                    't' if quote == '"' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '\'' if quote == '\'' => value.push('\''),
                    '"' if quote == '"' => value.push('"'),
                    other => {
                        value.push('\\');
                        value.push(other);
                    }
                }
                self.bump_char();
            } else {
                value.push(ch);
                self.bump_char();
            }
        }
        Err(Diagnostic::new("unterminated string literal", Some(start)))
    }

    fn lex_backtick_string(&mut self) -> Result<()> {
        let start = self.current_span(0);
        self.bump_char();
        let mut value = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '`' {
                self.bump_char();
                self.tokens.push(Token {
                    kind: TokenKind::BacktickString(value),
                    span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
                });
                return Ok(());
            }
            if ch == '$' && self.starts_string_interpolation() {
                return Err(Diagnostic::new(
                    "backtick interpolation is unsupported",
                    Some(self.current_char_span()),
                ));
            }
            if ch == '\\' {
                self.bump_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    Diagnostic::new("unterminated backtick escape", Some(self.current_span(0)))
                })?;
                match escaped {
                    '`' => value.push('`'),
                    '\\' => value.push('\\'),
                    '$' => value.push('$'),
                    other => {
                        value.push('\\');
                        value.push(other);
                    }
                }
                self.bump_char();
                continue;
            }
            value.push(ch);
            self.bump_char();
        }
        Err(Diagnostic::new("unterminated backtick string", Some(start)))
    }

    fn lex_heredoc_string(&mut self) -> Result<()> {
        let start = self.current_span(0);
        self.bump_char();
        self.bump_char();
        self.bump_char();

        self.skip_horizontal_whitespace();
        let (label, nowdoc) = self.lex_heredoc_label(start)?;
        self.skip_horizontal_whitespace();
        match self.peek_char() {
            Some('\r') => {
                self.bump_char();
                if matches!(self.peek_char(), Some('\n')) {
                    self.bump_char();
                }
            }
            Some('\n') => self.bump_char(),
            _ => {
                return Err(Diagnostic::new(
                    "expected heredoc label newline",
                    Some(self.current_char_span()),
                ))
            }
        }

        let mut value = String::new();
        let mut at_line_start = true;
        while self.peek_char().is_some() {
            if at_line_start && self.starts_heredoc_closing_label(&label) {
                trim_heredoc_terminal_newline(&mut value);
                for _ in 0..label.len() {
                    self.bump_char();
                }
                self.tokens.push(Token {
                    kind: TokenKind::String(value),
                    span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
                });
                return Ok(());
            }

            let ch = self.peek_char().expect("checked by loop condition");
            if !nowdoc && ch == '$' && self.starts_heredoc_interpolation() {
                return Err(Diagnostic::new(
                    "heredoc interpolation is unsupported",
                    Some(self.current_char_span()),
                ));
            }
            if !nowdoc && ch == '\\' {
                self.bump_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    Diagnostic::new("unterminated string escape", Some(self.current_span(0)))
                })?;
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    'e' => value.push('\u{1b}'),
                    'v' => value.push('\u{0b}'),
                    'f' => value.push('\u{0c}'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '$' => value.push('$'),
                    'x' => {
                        self.bump_char();
                        let mut digits = String::new();
                        for _ in 0..2 {
                            if let Some(hex) = self.peek_char() {
                                if hex.is_ascii_hexdigit() {
                                    digits.push(hex);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        if digits.is_empty() {
                            value.push('\\');
                            value.push('x');
                        } else {
                            let byte = u8::from_str_radix(&digits, 16).unwrap();
                            push_php_string_byte(&mut value, byte);
                        }
                        at_line_start = false;
                        continue;
                    }
                    '0'..='7' => {
                        let mut digits = String::new();
                        for _ in 0..3 {
                            if let Some(octal) = self.peek_char() {
                                if matches!(octal, '0'..='7') {
                                    digits.push(octal);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        let byte = (u16::from_str_radix(&digits, 8).unwrap() & 0xff) as u8;
                        push_php_string_byte(&mut value, byte);
                        at_line_start = false;
                        continue;
                    }
                    other => {
                        value.push('\\');
                        value.push(other);
                    }
                }
                self.bump_char();
                at_line_start = false;
                continue;
            }

            if !nowdoc && ch == '\\' {
                self.bump_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    Diagnostic::new("unterminated string escape", Some(self.current_span(0)))
                })?;
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    'e' => value.push('\u{1b}'),
                    'v' => value.push('\u{0b}'),
                    'f' => value.push('\u{0c}'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '$' => value.push('$'),
                    'x' => {
                        self.bump_char();
                        let mut digits = String::new();
                        for _ in 0..2 {
                            if let Some(hex) = self.peek_char() {
                                if hex.is_ascii_hexdigit() {
                                    digits.push(hex);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        if digits.is_empty() {
                            value.push('\\');
                            value.push('x');
                        } else {
                            let byte = u8::from_str_radix(&digits, 16).unwrap();
                            push_php_string_byte(&mut value, byte);
                        }
                        at_line_start = false;
                        continue;
                    }
                    '0'..='7' => {
                        let mut digits = String::new();
                        for _ in 0..3 {
                            if let Some(octal) = self.peek_char() {
                                if matches!(octal, '0'..='7') {
                                    digits.push(octal);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        let byte = u16::from_str_radix(&digits, 8).unwrap();
                        push_php_string_byte(&mut value, (byte & 0xff) as u8);
                        at_line_start = false;
                        continue;
                    }
                    other => {
                        value.push('\\');
                        value.push(other);
                    }
                }
                self.bump_char();
                at_line_start = false;
                continue;
            }

            value.push(ch);
            self.bump_char();
            at_line_start = ch == '\n';
        }

        Err(Diagnostic::new("unterminated heredoc string", Some(start)))
    }

    fn lex_heredoc_label(&mut self, start: SourceSpan) -> Result<(String, bool)> {
        match self.peek_char() {
            Some('\'') | Some('"') => {
                let quote = self.peek_char().expect("peeked quote");
                let nowdoc = quote == '\'';
                self.bump_char();
                let mut label = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch == quote {
                        self.bump_char();
                        validate_heredoc_label(&label, start)?;
                        return Ok((label, nowdoc));
                    }
                    label.push(ch);
                    self.bump_char();
                }
                Err(Diagnostic::new("unterminated heredoc label", Some(start)))
            }
            Some(ch) if is_ident_start(ch) => {
                let mut label = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        label.push(ch);
                        self.bump_char();
                    } else {
                        break;
                    }
                }
                Ok((label, false))
            }
            Some(_) => Err(Diagnostic::new(
                "expected heredoc label",
                Some(self.current_char_span()),
            )),
            None => Err(Diagnostic::new("expected heredoc label", Some(start))),
        }
    }

    fn lex_double_quoted_string(&mut self) -> Result<()> {
        let start = self.current_span(0);
        self.bump_char();
        let mut literal = String::new();
        let mut parts = Vec::new();
        let mut has_variable = false;

        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.bump_char();
                let kind = if has_variable {
                    if !literal.is_empty() {
                        parts.push(StringPart::Literal(literal));
                    }
                    TokenKind::InterpolatedString(parts)
                } else {
                    TokenKind::String(literal)
                };
                self.tokens.push(Token {
                    kind,
                    span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
                });
                return Ok(());
            }

            if ch == '\\' {
                self.bump_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    Diagnostic::new("unterminated string escape", Some(self.current_span(0)))
                })?;
                match escaped {
                    'n' => literal.push('\n'),
                    'r' => literal.push('\r'),
                    't' => literal.push('\t'),
                    'e' => literal.push('\u{1b}'),
                    'v' => literal.push('\u{0b}'),
                    'f' => literal.push('\u{0c}'),
                    '\\' => literal.push('\\'),
                    '"' => literal.push('"'),
                    '$' => literal.push('$'),
                    'x' => {
                        self.bump_char();
                        let mut digits = String::new();
                        for _ in 0..2 {
                            if let Some(hex) = self.peek_char() {
                                if hex.is_ascii_hexdigit() {
                                    digits.push(hex);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        if digits.is_empty() {
                            literal.push('\\');
                            literal.push('x');
                        } else {
                            let value = u8::from_str_radix(&digits, 16).unwrap();
                            push_php_string_byte(&mut literal, value);
                        }
                        continue;
                    }
                    '0'..='7' => {
                        let mut digits = String::new();
                        for _ in 0..3 {
                            if let Some(octal) = self.peek_char() {
                                if matches!(octal, '0'..='7') {
                                    digits.push(octal);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        let value = u16::from_str_radix(&digits, 8).unwrap();
                        push_php_string_byte(&mut literal, (value & 0xff) as u8);
                        continue;
                    }
                    other => {
                        literal.push('\\');
                        literal.push(other);
                    }
                }
                self.bump_char();
                continue;
            }

            if ch == '{' && self.rest().starts_with("{$") {
                if !literal.is_empty() {
                    parts.push(StringPart::Literal(literal));
                    literal = String::new();
                }
                parts.push(self.lex_braced_interpolation_part()?);
                has_variable = true;
                continue;
            }

            if ch == '$' {
                let start = self.current_span(1);
                self.bump_char();
                if let Some(first) = self.peek_char() {
                    if first == '{' {
                        if !literal.is_empty() {
                            parts.push(StringPart::Literal(literal));
                            literal = String::new();
                        }
                        parts.push(self.lex_legacy_dollar_brace_interpolation_part(start)?);
                        has_variable = true;
                        continue;
                    }
                    if is_ident_start(first) {
                        if !literal.is_empty() {
                            parts.push(StringPart::Literal(literal));
                            literal = String::new();
                        }
                        let mut name = String::new();
                        while let Some(ch) = self.peek_char() {
                            if ch.is_ascii_alphanumeric() || ch == '_' {
                                name.push(ch);
                                self.bump_char();
                            } else {
                                break;
                            }
                        }
                        if self.rest().starts_with("->") {
                            self.bump_char();
                            self.bump_char();
                            let property = self.read_interpolation_variable_name(start)?;
                            parts.push(StringPart::PropertyFetch {
                                variable: name,
                                property,
                            });
                            has_variable = true;
                            continue;
                        }
                        let indices = self.lex_unbraced_interpolation_indices()?;
                        if indices.is_empty() {
                            parts.push(StringPart::Variable(name));
                        } else {
                            parts.push(StringPart::ArrayAccess {
                                array: name,
                                indices,
                            });
                        }
                        has_variable = true;
                        continue;
                    }
                }
                literal.push('$');
                continue;
            }

            literal.push(ch);
            self.bump_char();
        }

        Err(Diagnostic::new("unterminated string literal", Some(start)))
    }

    fn lex_legacy_dollar_brace_interpolation_part(
        &mut self,
        start: SourceSpan,
    ) -> Result<StringPart> {
        debug_assert_eq!(self.peek_char(), Some('{'));
        self.bump_char();
        if matches!(self.peek_char(), Some('$')) {
            return Err(Diagnostic::new(
                "complex string interpolation is unsupported",
                Some(self.current_char_span()),
            ));
        }
        let name = self.read_interpolation_variable_name(start)?;
        match self.peek_char() {
            Some('}') => {
                self.bump_char();
                Ok(StringPart::LegacyDollarBraceVariable(name))
            }
            Some(_) => Err(Diagnostic::new(
                "complex string interpolation is unsupported",
                Some(self.current_char_span()),
            )),
            None => Err(Diagnostic::new(
                "unterminated string interpolation",
                Some(start),
            )),
        }
    }

    fn lex_braced_interpolation_part(&mut self) -> Result<StringPart> {
        let start = self.current_span(1);
        self.bump_char();
        debug_assert_eq!(self.peek_char(), Some('$'));
        self.bump_char();
        let array = self.read_interpolation_variable_name(start)?;
        let mut indices = Vec::new();

        loop {
            self.skip_interpolation_whitespace();
            match self.peek_char() {
                Some('}') => {
                    self.bump_char();
                    break;
                }
                Some('-') if self.rest().starts_with("->") && indices.is_empty() => {
                    self.bump_char();
                    self.bump_char();
                    let first_member = self.read_interpolation_variable_name(start)?;
                    self.skip_interpolation_whitespace();
                    if matches!(self.peek_char(), Some('(')) {
                        self.bump_char();
                        self.skip_interpolation_whitespace();
                        if !matches!(self.peek_char(), Some(')')) {
                            return Err(Diagnostic::new(
                                "complex string interpolation is unsupported",
                                Some(self.current_char_span()),
                            ));
                        }
                        self.bump_char();
                        self.skip_interpolation_whitespace();
                        if !matches!(self.peek_char(), Some('}')) {
                            return Err(Diagnostic::new(
                                "complex string interpolation is unsupported",
                                Some(self.current_char_span()),
                            ));
                        }
                        self.bump_char();
                        return Ok(StringPart::MethodCall {
                            variable: array,
                            method: first_member,
                        });
                    }
                    let mut properties = vec![first_member];
                    loop {
                        self.skip_interpolation_whitespace();
                        if !self.rest().starts_with("->") {
                            break;
                        }
                        self.bump_char();
                        self.bump_char();
                        properties.push(self.read_interpolation_variable_name(start)?);
                    }
                    self.skip_interpolation_whitespace();
                    if !matches!(self.peek_char(), Some('}')) {
                        return Err(Diagnostic::new(
                            "complex string interpolation is unsupported",
                            Some(self.current_char_span()),
                        ));
                    }
                    self.bump_char();
                    if properties.len() == 1 {
                        return Ok(StringPart::PropertyFetch {
                            variable: array,
                            property: properties.remove(0),
                        });
                    }
                    return Ok(StringPart::PropertyChain {
                        variable: array,
                        properties,
                    });
                }
                Some('[') => {
                    self.bump_char();
                    self.skip_interpolation_whitespace();
                    if matches!(self.peek_char(), Some(']')) {
                        return Err(Diagnostic::new(
                            "array append interpolation is unsupported",
                            Some(self.current_char_span()),
                        ));
                    }
                    let index = self.lex_interpolation_index()?;
                    self.skip_interpolation_whitespace();
                    if !matches!(self.peek_char(), Some(']')) {
                        return Err(Diagnostic::new(
                            "expected interpolation array index close bracket",
                            Some(self.current_char_span()),
                        ));
                    }
                    self.bump_char();
                    indices.push(index);
                }
                Some('{') => {
                    return Err(Diagnostic::parse_error(
                        "syntax error, unexpected token \"{\", expecting \"->\" or \"?->\" or \"[\"",
                        Some(self.current_char_span()),
                    ));
                }
                Some(_) => {
                    return Err(Diagnostic::new(
                        "complex string interpolation is unsupported",
                        Some(self.current_char_span()),
                    ));
                }
                None => {
                    return Err(Diagnostic::new(
                        "unterminated string interpolation",
                        Some(start),
                    ));
                }
            }
        }

        if indices.is_empty() {
            Ok(StringPart::Variable(array))
        } else {
            Ok(StringPart::ArrayAccess { array, indices })
        }
    }

    fn lex_unbraced_interpolation_indices(&mut self) -> Result<Vec<StringInterpolationIndex>> {
        let mut indices = Vec::new();
        while matches!(self.peek_char(), Some('[')) {
            self.bump_char();
            if matches!(self.peek_char(), Some(']')) {
                return Err(Diagnostic::new(
                    "array append interpolation is unsupported",
                    Some(self.current_char_span()),
                ));
            }
            let index = self.lex_unbraced_interpolation_index()?;
            if !matches!(self.peek_char(), Some(']')) {
                return Err(Diagnostic::new(
                    "expected interpolation array index close bracket",
                    Some(self.current_char_span()),
                ));
            }
            self.bump_char();
            indices.push(index);
        }
        Ok(indices)
    }

    fn lex_unbraced_interpolation_index(&mut self) -> Result<StringInterpolationIndex> {
        match self.peek_char() {
            Some('$') => {
                let span = self.current_span(1);
                self.bump_char();
                Ok(StringInterpolationIndex::Variable(
                    self.read_interpolation_variable_name(span)?,
                ))
            }
            Some('-') | Some('0'..='9') => self.lex_unbraced_interpolation_numeric_index(),
            Some(ch) if is_ident_start(ch) => {
                let mut key = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        key.push(ch);
                        self.bump_char();
                    } else {
                        break;
                    }
                }
                Ok(StringInterpolationIndex::String(key))
            }
            Some(_) => Err(Diagnostic::new(
                "complex string interpolation is unsupported",
                Some(self.current_char_span()),
            )),
            None => Err(Diagnostic::new(
                "unterminated string interpolation",
                Some(self.current_span(0)),
            )),
        }
    }

    fn lex_interpolation_index(&mut self) -> Result<StringInterpolationIndex> {
        self.skip_interpolation_whitespace();
        match self.peek_char() {
            Some('\'') | Some('"') => Ok(StringInterpolationIndex::String(
                self.lex_interpolation_index_string()?,
            )),
            Some('$') => {
                let span = self.current_span(1);
                self.bump_char();
                Ok(StringInterpolationIndex::Variable(
                    self.read_interpolation_variable_name(span)?,
                ))
            }
            Some('-') | Some('0'..='9') => self.lex_interpolation_index_int(),
            Some(_) => Err(Diagnostic::new(
                "complex string interpolation is unsupported",
                Some(self.current_char_span()),
            )),
            None => Err(Diagnostic::new(
                "unterminated string interpolation",
                Some(self.current_span(0)),
            )),
        }
    }

    fn lex_interpolation_index_string(&mut self) -> Result<String> {
        let start = self.current_span(1);
        let quote = self.peek_char().expect("string index starts with quote");
        self.bump_char();
        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == quote {
                self.bump_char();
                return Ok(value);
            }

            if ch == '\\' {
                self.bump_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    Diagnostic::new("unterminated string escape", Some(self.current_span(0)))
                })?;
                match escaped {
                    'n' if quote == '"' => value.push('\n'),
                    'r' if quote == '"' => value.push('\r'),
                    't' if quote == '"' => value.push('\t'),
                    'e' if quote == '"' => value.push('\u{1b}'),
                    'v' if quote == '"' => value.push('\u{0b}'),
                    'f' if quote == '"' => value.push('\u{0c}'),
                    '\\' => value.push('\\'),
                    '\'' if quote == '\'' => value.push('\''),
                    '"' if quote == '"' => value.push('"'),
                    '$' if quote == '"' => value.push('$'),
                    'x' if quote == '"' => {
                        self.bump_char();
                        let mut digits = String::new();
                        for _ in 0..2 {
                            if let Some(hex) = self.peek_char() {
                                if hex.is_ascii_hexdigit() {
                                    digits.push(hex);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        if digits.is_empty() {
                            value.push('\\');
                            value.push('x');
                        } else {
                            let byte = u8::from_str_radix(&digits, 16).unwrap();
                            push_php_string_byte(&mut value, byte);
                        }
                        continue;
                    }
                    '0'..='7' if quote == '"' => {
                        let mut digits = String::new();
                        for _ in 0..3 {
                            if let Some(octal) = self.peek_char() {
                                if matches!(octal, '0'..='7') {
                                    digits.push(octal);
                                    self.bump_char();
                                    continue;
                                }
                            }
                            break;
                        }
                        let byte = u16::from_str_radix(&digits, 8).unwrap();
                        push_php_string_byte(&mut value, (byte & 0xff) as u8);
                        continue;
                    }
                    other => {
                        value.push('\\');
                        value.push(other);
                    }
                }
                self.bump_char();
            } else {
                value.push(ch);
                self.bump_char();
            }
        }

        Err(Diagnostic::new("unterminated string literal", Some(start)))
    }

    fn lex_interpolation_index_int(&mut self) -> Result<StringInterpolationIndex> {
        let start = self.current_span(1);
        let mut text = String::new();
        if matches!(self.peek_char(), Some('-')) {
            text.push('-');
            self.bump_char();
        }
        let saw_digits = self.collect_digits(&mut text, |ch| ch.is_ascii_digit());
        if !saw_digits || text == "-" {
            return Err(Diagnostic::new("invalid integer literal", Some(start)));
        }
        let value = text
            .parse::<i64>()
            .map_err(|_| Diagnostic::new("invalid integer literal", Some(start)))?;
        Ok(StringInterpolationIndex::Int(value))
    }

    fn lex_unbraced_interpolation_numeric_index(&mut self) -> Result<StringInterpolationIndex> {
        let start = self.current_span(1);
        let mut text = String::new();
        if matches!(self.peek_char(), Some('-')) {
            text.push('-');
            self.bump_char();
        }
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }
        if text == "-" {
            return Err(Diagnostic::new("invalid integer literal", Some(start)));
        }
        if let Some(value) = canonical_interpolation_array_int_index(&text) {
            Ok(StringInterpolationIndex::Int(value))
        } else {
            Ok(StringInterpolationIndex::String(text))
        }
    }

    fn read_interpolation_variable_name(&mut self, span: SourceSpan) -> Result<String> {
        let Some(first) = self.peek_char() else {
            return Err(Diagnostic::new(
                "expected variable name after `$`",
                Some(span),
            ));
        };
        if !is_ident_start(first) {
            return Err(Diagnostic::new(
                "expected variable name after `$`",
                Some(self.current_char_span()),
            ));
        }
        let mut name = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                name.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }
        Ok(name)
    }

    fn skip_interpolation_whitespace(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.bump_char();
        }
    }

    fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(' ' | '\t')) {
            self.bump_char();
        }
    }

    fn starts_heredoc_closing_label(&self, label: &str) -> bool {
        if !self.rest().starts_with(label) {
            return false;
        }
        matches!(
            self.rest()[label.len()..].chars().next(),
            None | Some(';') | Some('\r') | Some('\n')
        )
    }

    fn starts_heredoc_interpolation(&self) -> bool {
        matches!(
            self.rest().chars().nth(1),
            Some('$') | Some('{') | Some('a'..='z') | Some('A'..='Z') | Some('_')
        )
    }

    fn starts_string_interpolation(&self) -> bool {
        self.starts_heredoc_interpolation()
    }

    fn lex_number(&mut self) -> Result<()> {
        let start = self.current_span(0);
        let mut text = String::new();

        if self.starts_radix_integer_prefix("0x", |ch| ch.is_ascii_hexdigit())
            || self.starts_radix_integer_prefix("0X", |ch| ch.is_ascii_hexdigit())
        {
            self.bump_char();
            self.bump_char();
            self.collect_digits(&mut text, |ch| ch.is_ascii_hexdigit());
            self.tokens.push(Token {
                kind: radix_integer_token_kind(&text, 16, start)?,
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
            return Ok(());
        }

        if self.starts_radix_integer_prefix("0b", |ch| matches!(ch, '0' | '1'))
            || self.starts_radix_integer_prefix("0B", |ch| matches!(ch, '0' | '1'))
        {
            self.bump_char();
            self.bump_char();
            self.collect_digits(&mut text, |ch| matches!(ch, '0' | '1'));
            self.tokens.push(Token {
                kind: radix_integer_token_kind(&text, 2, start)?,
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
            return Ok(());
        }

        if self.starts_radix_integer_prefix("0o", |ch| matches!(ch, '0'..='7'))
            || self.starts_radix_integer_prefix("0O", |ch| matches!(ch, '0'..='7'))
        {
            self.bump_char();
            self.bump_char();
            self.collect_digits(&mut text, |ch| matches!(ch, '0'..='7'));
            self.tokens.push(Token {
                kind: radix_integer_token_kind(&text, 8, start)?,
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
            return Ok(());
        }

        self.collect_digits(&mut text, |ch| ch.is_ascii_digit());
        let mut is_float = false;
        if self.peek_char() == Some('.') {
            is_float = true;
            text.push('.');
            self.bump_char();
            self.collect_digits(&mut text, |ch| ch.is_ascii_digit());
        }

        if self.starts_valid_exponent() {
            is_float = true;
            let exponent = self.peek_char().expect("valid exponent has marker");
            text.push(exponent);
            self.bump_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                text.push(self.peek_char().expect("peeked sign"));
                self.bump_char();
            }
            self.collect_digits(&mut text, |ch| ch.is_ascii_digit());
        }

        if is_float {
            let value = text
                .parse::<f64>()
                .map_err(|_| Diagnostic::new("invalid float literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Float(value),
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
        } else {
            let kind = if text.len() > 1 && text.starts_with('0') {
                if text.bytes().any(|digit| matches!(digit, b'8' | b'9')) {
                    return Err(Diagnostic::parse_error(
                        "Invalid numeric literal",
                        Some(start),
                    ));
                }
                radix_integer_token_kind(&text, 8, start)
            } else {
                decimal_integer_token_kind(&text, start)
            }?;
            self.tokens.push(Token {
                kind,
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
        }
        Ok(())
    }

    fn starts_leading_dot_float(&self) -> bool {
        let mut chars = self.rest().chars();
        matches!(chars.next(), Some('.')) && chars.next().is_some_and(|ch| ch.is_ascii_digit())
    }

    fn lex_leading_dot_float(&mut self) -> Result<()> {
        let start = self.current_span(0);
        let mut text = String::from("0");

        text.push('.');
        self.bump_char();
        self.collect_digits(&mut text, |ch| ch.is_ascii_digit());

        if self.starts_valid_exponent() {
            let exponent = self.peek_char().expect("valid exponent has marker");
            text.push(exponent);
            self.bump_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                text.push(self.peek_char().expect("peeked sign"));
                self.bump_char();
            }
            self.collect_digits(&mut text, |ch| ch.is_ascii_digit());
        }

        let value = text
            .parse::<f64>()
            .map_err(|_| Diagnostic::new("invalid float literal", Some(start)))?;
        self.tokens.push(Token {
            kind: TokenKind::Float(value),
            span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
        });
        Ok(())
    }

    fn starts_radix_integer_prefix<F>(&self, prefix: &str, is_digit: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        self.rest().starts_with(prefix)
            && self.rest().chars().nth(prefix.len()).is_some_and(is_digit)
    }

    fn starts_valid_exponent(&self) -> bool {
        let mut chars = self.rest().chars();
        let Some(marker) = chars.next() else {
            return false;
        };
        if !matches!(marker, 'e' | 'E') {
            return false;
        }
        match chars.next() {
            Some('+') | Some('-') => chars.next().is_some_and(|ch| ch.is_ascii_digit()),
            Some(ch) => ch.is_ascii_digit(),
            None => false,
        }
    }

    fn collect_digits<F>(&mut self, text: &mut String, is_digit: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        let mut saw_digit = false;
        let mut last_was_digit = false;
        while let Some(ch) = self.peek_char() {
            if is_digit(ch) {
                text.push(ch);
                self.bump_char();
                saw_digit = true;
                last_was_digit = true;
            } else if ch == '_' && last_was_digit && self.next_char_matches(&is_digit) {
                self.bump_char();
                last_was_digit = false;
            } else {
                break;
            }
        }
        saw_digit
    }

    fn next_char_matches<F>(&self, predicate: &F) -> bool
    where
        F: Fn(char) -> bool,
    {
        self.rest().chars().nth(1).is_some_and(predicate)
    }

    fn lex_variable(&mut self) -> Result<()> {
        let start = self.current_span(0);
        self.bump_char();
        let Some(first) = self.peek_char() else {
            return Err(Diagnostic::new(
                "expected variable name after `$`",
                Some(start),
            ));
        };
        if first == '$' || first == '{' {
            self.tokens.push(Token {
                kind: TokenKind::Dollar,
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
            return Ok(());
        }
        if !is_ident_start(first) {
            return Err(Diagnostic::new(
                "expected variable name after `$`",
                Some(self.current_span(1)),
            ));
        }
        let mut name = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                name.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Variable(name),
            span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
        });
        Ok(())
    }

    fn lex_word(&mut self) -> Result<()> {
        let start = self.current_span(0);
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }
        let lowercase = text.to_ascii_lowercase();
        let kind = match lowercase.as_str() {
            "echo" => TokenKind::Echo,
            "print" => TokenKind::Print,
            "if" => TokenKind::If,
            "elseif" => TokenKind::Elseif,
            "else" => TokenKind::Else,
            "do" => TokenKind::Do,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "foreach" => TokenKind::Foreach,
            "as" => TokenKind::As,
            "switch" => TokenKind::Switch,
            "match" => TokenKind::Match,
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "include" => TokenKind::Include,
            "include_once" => TokenKind::IncludeOnce,
            "require" => TokenKind::Require,
            "require_once" => TokenKind::RequireOnce,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "throw" => TokenKind::Throw,
            "yield" => TokenKind::Yield,
            "goto" => TokenKind::Goto,
            "const" => TokenKind::Const,
            "function" => TokenKind::Function,
            "global" => TokenKind::Global,
            "new" => TokenKind::New,
            "clone" => TokenKind::Clone,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "and" => TokenKind::KeywordAnd,
            "or" => TokenKind::KeywordOr,
            "xor" => TokenKind::KeywordXor,
            "int" => TokenKind::IntType,
            "integer" => TokenKind::IntegerType,
            "float" => TokenKind::FloatType,
            "double" => TokenKind::DoubleType,
            "string" => TokenKind::StringType,
            "binary" => TokenKind::BinaryType,
            "bool" => TokenKind::BoolType,
            "boolean" => TokenKind::BooleanType,
            _ => TokenKind::Identifier(text),
        };
        self.tokens.push(Token {
            kind,
            span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
        });
        Ok(())
    }

    fn push_fixed(&mut self, kind: TokenKind, len: usize) {
        let span = self.current_span(len);
        for _ in 0..len {
            self.bump_char();
        }
        self.tokens.push(Token { kind, span });
    }

    fn push_open_tag(&mut self) {
        self.push_fixed(TokenKind::OpenTag, 5);
        self.seen_open_tag = true;
        self.closed_php = false;
    }

    fn push_close_tag(&mut self) {
        let span = self.current_span(2);
        self.bump_char();
        self.bump_char();
        if self.rest().starts_with("\r\n") {
            self.bump_char();
            self.bump_char();
        } else if self.rest().starts_with('\n') {
            self.bump_char();
        }
        self.tokens.push(Token {
            kind: TokenKind::CloseTag,
            span,
        });
        self.closed_php = true;
    }

    fn current_span(&self, width: usize) -> SourceSpan {
        SourceSpan::new(self.cursor, self.cursor + width, self.line, self.column)
    }

    fn current_char_span(&self) -> SourceSpan {
        let width = self.peek_char().map(char::len_utf8).unwrap_or(0);
        self.current_span(width)
    }

    fn rest(&self) -> &'a str {
        &self.source[self.cursor..]
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.cursor += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn validate_heredoc_label(label: &str, span: SourceSpan) -> Result<()> {
    let mut chars = label.chars();
    let Some(first) = chars.next() else {
        return Err(Diagnostic::new("expected heredoc label", Some(span)));
    };
    if !is_ident_start(first) || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(Diagnostic::new("invalid heredoc label", Some(span)));
    }
    Ok(())
}

fn trim_heredoc_terminal_newline(value: &mut String) {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
}

fn radix_integer_token_kind(text: &str, radix: u32, span: SourceSpan) -> Result<TokenKind> {
    match i64::from_str_radix(text, radix) {
        Ok(value) => Ok(TokenKind::Int(value)),
        Err(_) => {
            let mut value = radix_digits_to_f64(text, radix, span)?;
            if radix == 2 {
                value = php_binary_overflow_float(value);
            }
            Ok(TokenKind::Float(value))
        }
    }
}

fn radix_digits_to_f64(text: &str, radix: u32, span: SourceSpan) -> Result<f64> {
    let mut value = 0.0;
    let radix_value = radix as f64;
    for ch in text.chars() {
        let digit = ch
            .to_digit(radix)
            .ok_or_else(|| Diagnostic::new("invalid integer literal", Some(span)))?;
        value = value * radix_value + digit as f64;
    }
    Ok(value)
}

fn php_binary_overflow_float(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        f64::from_bits(value.to_bits() - 1)
    } else {
        value
    }
}

fn decimal_integer_token_kind(text: &str, span: SourceSpan) -> Result<TokenKind> {
    match text.parse::<i64>() {
        Ok(value) => Ok(TokenKind::Int(value)),
        Err(_) => {
            let value = text
                .parse::<f64>()
                .map_err(|_| Diagnostic::new("invalid integer literal", Some(span)))?;
            Ok(TokenKind::Float(value))
        }
    }
}

fn canonical_interpolation_array_int_index(text: &str) -> Option<i64> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if digits.len() > 1 && digits.starts_with('0') {
        return None;
    }
    if text.starts_with("-0") {
        return None;
    }
    text.parse::<i64>().ok()
}

fn push_php_string_byte(value: &mut String, byte: u8) {
    if byte <= 0x7f {
        value.push(byte as char);
    } else {
        value.push(
            char::from_u32(PHP_BINARY_BYTE_SENTINEL_BASE + byte as u32)
                .expect("PHP binary-byte sentinel must be a Unicode scalar"),
        );
    }
}
