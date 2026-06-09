use crate::diagnostic::{Diagnostic, Result, SourceSpan};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenTag,
    Echo,
    String(String),
    Int(i64),
    Float(f64),
    True,
    False,
    Null,
    Variable(String),
    Equal,
    Plus,
    Dot,
    Comma,
    Semicolon,
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
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> Result<Vec<Token>> {
        while let Some(ch) = self.peek_char() {
            match ch {
                '<' if self.rest().starts_with("<?php") => self.push_fixed(TokenKind::OpenTag, 5),
                c if c.is_whitespace() => self.bump_char(),
                ';' => self.push_fixed(TokenKind::Semicolon, 1),
                ',' => self.push_fixed(TokenKind::Comma, 1),
                '=' => self.push_fixed(TokenKind::Equal, 1),
                '+' => self.push_fixed(TokenKind::Plus, 1),
                '.' => self.push_fixed(TokenKind::Dot, 1),
                '$' => self.lex_variable()?,
                '\'' | '"' => self.lex_string(ch)?,
                c if c.is_ascii_digit() => self.lex_number()?,
                c if is_ident_start(c) => self.lex_word()?,
                _ => {
                    return Err(Diagnostic::new(
                        format!("unsupported PHP token {:?}", ch),
                        Some(self.current_span(1)),
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
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
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

    fn current_span(&self, width: usize) -> SourceSpan {
        SourceSpan::new(self.cursor, self.cursor + width, self.line, self.column)
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
