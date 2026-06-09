use crate::diagnostic::{Diagnostic, Result, SourceSpan};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenTag,
    CloseTag,
    Echo,
    Print,
    String(String),
    Int(i64),
    Float(f64),
    True,
    False,
    Null,
    Variable(String),
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    Greater,
    AndAnd,
    OrOr,
    PlusEqual,
    DotEqual,
    Plus,
    Minus,
    Bang,
    Dot,
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    IntType,
    FloatType,
    StringType,
    BoolType,
    Eof,
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
                if ch.is_whitespace() {
                    self.bump_char();
                    continue;
                }

                return Err(Diagnostic::new(
                    "inline HTML after close tag is unsupported",
                    Some(self.current_char_span()),
                ));
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

                return Err(Diagnostic::new(
                    "expected <?php open tag",
                    Some(self.current_char_span()),
                ));
            }

            match ch {
                '?' if self.rest().starts_with("?>") => self.push_close_tag(),
                '/' if self.rest().starts_with("//") => self.skip_line_comment(),
                '/' if self.rest().starts_with("/*") => self.skip_block_comment()?,
                '#' => self.skip_line_comment(),
                c if c.is_whitespace() => self.bump_char(),
                ';' => self.push_fixed(TokenKind::Semicolon, 1),
                ',' => self.push_fixed(TokenKind::Comma, 1),
                '=' if self.rest().starts_with("==") => self.push_fixed(TokenKind::EqualEqual, 2),
                '=' => self.push_fixed(TokenKind::Equal, 1),
                '!' if self.rest().starts_with("!=") => self.push_fixed(TokenKind::NotEqual, 2),
                '<' => self.push_fixed(TokenKind::Less, 1),
                '>' => self.push_fixed(TokenKind::Greater, 1),
                '&' if self.rest().starts_with("&&") => self.push_fixed(TokenKind::AndAnd, 2),
                '|' if self.rest().starts_with("||") => self.push_fixed(TokenKind::OrOr, 2),
                '+' if self.rest().starts_with("+=") => self.push_fixed(TokenKind::PlusEqual, 2),
                '+' if self.rest().starts_with("++") => {
                    return Err(Diagnostic::new(
                        "unsupported increment operator `++`",
                        Some(self.current_span(2)),
                    ));
                }
                '+' => self.push_fixed(TokenKind::Plus, 1),
                '-' if self.rest().starts_with("--") => {
                    return Err(Diagnostic::new(
                        "unsupported decrement operator `--`",
                        Some(self.current_span(2)),
                    ));
                }
                '-' => self.push_fixed(TokenKind::Minus, 1),
                '!' => self.push_fixed(TokenKind::Bang, 1),
                '.' if self.rest().starts_with(".=") => self.push_fixed(TokenKind::DotEqual, 2),
                '.' => self.push_fixed(TokenKind::Dot, 1),
                '(' => self.push_fixed(TokenKind::LeftParen, 1),
                ')' => self.push_fixed(TokenKind::RightParen, 1),
                '$' => self.lex_variable()?,
                '\'' | '"' => self.lex_string(ch)?,
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
        Err(Diagnostic::new("unterminated comment", Some(start)))
    }

    fn skip_line(&mut self) {
        while let Some(ch) = self.peek_char() {
            self.bump_char();
            if ch == '\n' {
                break;
            }
        }
    }

    fn lex_string(&mut self, quote: char) -> Result<()> {
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
                let mapped = match escaped {
                    'n' if quote == '"' => '\n',
                    'r' if quote == '"' => '\r',
                    't' if quote == '"' => '\t',
                    '\\' => '\\',
                    '\'' if quote == '\'' => '\'',
                    '"' if quote == '"' => '"',
                    other => other,
                };
                value.push(mapped);
                self.bump_char();
            } else {
                value.push(ch);
                self.bump_char();
            }
        }
        Err(Diagnostic::new("unterminated string literal", Some(start)))
    }

    fn lex_number(&mut self) -> Result<()> {
        let start = self.current_span(0);
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }
        if self.peek_char() == Some('.') {
            text.push('.');
            self.bump_char();
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    text.push(ch);
                    self.bump_char();
                } else {
                    break;
                }
            }
            let value = text
                .parse::<f64>()
                .map_err(|_| Diagnostic::new("invalid float literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Float(value),
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
        } else {
            let value = text
                .parse::<i64>()
                .map_err(|_| Diagnostic::new("invalid integer literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Int(value),
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
        }
        Ok(())
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
        let kind = match text.as_str() {
            "echo" => TokenKind::Echo,
            "print" => TokenKind::Print,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "int" => TokenKind::IntType,
            "float" => TokenKind::FloatType,
            "string" => TokenKind::StringType,
            "bool" => TokenKind::BoolType,
            _ => {
                return Err(Diagnostic::new(
                    format!("unsupported identifier `{text}`"),
                    Some(start),
                ))
            }
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
    }

    fn push_close_tag(&mut self) {
        self.push_fixed(TokenKind::CloseTag, 2);
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
