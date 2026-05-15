use crate::ast::{InterpolatedStringPart, Span};
use crate::error::{CompileResult, Diagnostic, Phase};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Eof,
    Variable(String),
    Identifier(String),
    Int(i64),
    Float(f64),
    StringLiteral(String),
    InterpolatedString(Vec<InterpolatedStringPart>),
    Echo,
    Print,
    Function,
    Fn,
    Class,
    Interface,
    Trait,
    Enum,
    Abstract,
    Final,
    Readonly,
    New,
    Public,
    Protected,
    Private,
    Static,
    Extends,
    Implements,
    Clone,
    Instanceof,
    Return,
    Global,
    Namespace,
    Use,
    Declare,
    Eval,
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
    If,
    Else,
    ElseIf,
    While,
    Do,
    Foreach,
    For,
    Switch,
    Match,
    Break,
    Continue,
    Throw,
    Try,
    Catch,
    Finally,
    Null,
    True,
    False,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Plus,
    Minus,
    Star,
    StarStar,
    Slash,
    Percent,
    Dot,
    ObjectOperator,
    DoubleColon,
    Backslash,
    Ellipsis,
    Ampersand,
    AmpAmp,
    Question,
    QuestionQuestion,
    Pipe,
    PipePipe,
    Caret,
    Tilde,
    Colon,
    Bang,
    Equal,
    FatArrow,
    EqualEqual,
    StrictEqual,
    BangEqual,
    StrictBangEqual,
    Less,
    LessEqual,
    LeftShift,
    Greater,
    GreaterEqual,
    RightShift,
}

pub fn tokenize(source: &str) -> CompileResult<Vec<Token>> {
    Lexer::new(source).tokenize()
}

struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    fn tokenize(mut self) -> CompileResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments()?;
            if self.is_at_end() {
                break;
            }

            if self.matches_php_open_tag() {
                continue;
            }

            if self.matches_php_close_tag() {
                continue;
            }

            let span = self.span();
            let ch = self.advance();
            let kind = match ch {
                '$' => self.lex_variable(span)?,
                '\'' | '"' => self.lex_string(ch, span)?,
                '0'..='9' => self.lex_number(ch, span)?,
                'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(ch),
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ';' => TokenKind::Semicolon,
                ',' => TokenKind::Comma,
                '+' => TokenKind::Plus,
                '-' => {
                    if self.match_char('>') {
                        TokenKind::ObjectOperator
                    } else {
                        TokenKind::Minus
                    }
                }
                '*' => {
                    if self.match_char('*') {
                        TokenKind::StarStar
                    } else {
                        TokenKind::Star
                    }
                }
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '.' => {
                    if self.match_char('.') {
                        if self.match_char('.') {
                            TokenKind::Ellipsis
                        } else {
                            return Err(self.error_at(span, "unexpected '..'"));
                        }
                    } else {
                        TokenKind::Dot
                    }
                }
                '&' => {
                    if self.match_char('&') {
                        TokenKind::AmpAmp
                    } else {
                        TokenKind::Ampersand
                    }
                }
                '?' => {
                    if self.match_char('?') {
                        TokenKind::QuestionQuestion
                    } else {
                        TokenKind::Question
                    }
                }
                '|' => {
                    if self.match_char('|') {
                        TokenKind::PipePipe
                    } else {
                        TokenKind::Pipe
                    }
                }
                '^' => TokenKind::Caret,
                '~' => TokenKind::Tilde,
                ':' => {
                    if self.match_char(':') {
                        TokenKind::DoubleColon
                    } else {
                        TokenKind::Colon
                    }
                }
                '\\' => TokenKind::Backslash,
                '=' => {
                    if self.match_char('=') {
                        if self.match_char('=') {
                            TokenKind::StrictEqual
                        } else {
                            TokenKind::EqualEqual
                        }
                    } else if self.match_char('>') {
                        TokenKind::FatArrow
                    } else {
                        TokenKind::Equal
                    }
                }
                '!' => {
                    if self.match_char('=') {
                        if self.match_char('=') {
                            TokenKind::StrictBangEqual
                        } else {
                            TokenKind::BangEqual
                        }
                    } else {
                        TokenKind::Bang
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        TokenKind::LessEqual
                    } else if self.match_char('<') {
                        if self.match_char('<') {
                            return Err(self.error_at(
                                span,
                                "unsupported heredoc/nowdoc string syntax: multiline string literals are not implemented",
                            ));
                        }
                        TokenKind::LeftShift
                    } else {
                        TokenKind::Less
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        TokenKind::GreaterEqual
                    } else if self.match_char('>') {
                        TokenKind::RightShift
                    } else {
                        TokenKind::Greater
                    }
                }
                _ => {
                    return Err(self.error_at(span, format!("unexpected character '{ch}'")));
                }
            };

            tokens.push(Token { kind, span });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.span(),
        });
        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) -> CompileResult<()> {
        loop {
            while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
                self.advance();
            }

            if self.peek() == Some('/') && self.peek_next() == Some('/') {
                while !matches!(self.peek(), None | Some('\n')) {
                    self.advance();
                }
                continue;
            }

            if self.peek() == Some('#') {
                if self.peek_next() == Some('[') {
                    self.skip_attribute_block()?;
                    continue;
                }
                while !matches!(self.peek(), None | Some('\n')) {
                    self.advance();
                }
                continue;
            }

            if self.peek() == Some('/') && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                let start = self.span();
                while !(self.peek() == Some('*') && self.peek_next() == Some('/')) {
                    if self.is_at_end() {
                        return Err(self.error_at(start, "unterminated block comment"));
                    }
                    self.advance();
                }
                self.advance();
                self.advance();
                continue;
            }

            break;
        }

        Ok(())
    }

    fn skip_attribute_block(&mut self) -> CompileResult<()> {
        let start = self.span();
        self.advance();
        self.advance();
        let mut depth = 1usize;

        while !self.is_at_end() {
            match self.advance() {
                '\'' | '"' => self.skip_quoted_attribute_string(start)?,
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        Err(self.error_at(
            start,
            "unterminated attribute syntax: expected ']' to close PHP attribute",
        ))
    }

    fn skip_quoted_attribute_string(&mut self, start: Span) -> CompileResult<()> {
        let quote = self.chars[self.index - 1];
        while !self.is_at_end() {
            let ch = self.advance();
            if ch == '\\' && !self.is_at_end() {
                self.advance();
                continue;
            }
            if ch == quote {
                return Ok(());
            }
        }

        Err(self.error_at(
            start,
            "unterminated attribute syntax: expected quoted string to close before PHP attribute end",
        ))
    }

    fn matches_php_open_tag(&mut self) -> bool {
        if !self.starts_with("<?") {
            return false;
        }

        if self.starts_with("<?php") {
            for _ in 0..5 {
                self.advance();
            }
            return true;
        }

        false
    }

    fn matches_php_close_tag(&mut self) -> bool {
        if !self.starts_with("?>") {
            return false;
        }

        self.advance();
        self.advance();
        true
    }

    fn lex_variable(&mut self, span: Span) -> CompileResult<TokenKind> {
        let mut name = String::new();
        match self.peek() {
            Some('$' | '{') => {
                return Err(self.error_at(
                    span,
                    "unsupported variable variable: variable variables are not implemented",
                ));
            }
            Some(ch) if is_identifier_start(ch) => name.push(self.advance()),
            _ => return Err(self.error_at(span, "expected variable name after '$'")),
        }

        while let Some(ch) = self.peek() {
            if is_identifier_part(ch) {
                name.push(self.advance());
            } else {
                break;
            }
        }

        Ok(TokenKind::Variable(name))
    }

    fn lex_string(&mut self, quote: char, span: Span) -> CompileResult<TokenKind> {
        let mut value = String::new();
        let mut parts = Vec::new();
        let interpolate = quote == '"';

        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                if !parts.is_empty() {
                    if !value.is_empty() {
                        parts.push(InterpolatedStringPart::Literal(value));
                    }
                    return Ok(TokenKind::InterpolatedString(parts));
                }
                return Ok(TokenKind::StringLiteral(value));
            }

            if ch == '\\' {
                self.advance();
                let escaped = match self.peek() {
                    Some('n') => {
                        self.advance();
                        '\n'
                    }
                    Some('r') => {
                        self.advance();
                        '\r'
                    }
                    Some('t') => {
                        self.advance();
                        '\t'
                    }
                    Some('\\') => {
                        self.advance();
                        '\\'
                    }
                    Some('\'') => {
                        self.advance();
                        '\''
                    }
                    Some('"') => {
                        self.advance();
                        '"'
                    }
                    Some('$') => {
                        self.advance();
                        '$'
                    }
                    Some(other) => {
                        self.advance();
                        other
                    }
                    None => return Err(self.error_at(span, "unterminated string literal")),
                };
                value.push(escaped);
                continue;
            }

            if interpolate && ch == '$' {
                if matches!(self.peek_next(), Some(next) if is_identifier_start(next)) {
                    self.advance();
                    if !value.is_empty() {
                        parts.push(InterpolatedStringPart::Literal(value));
                        value = String::new();
                    }

                    let mut name = String::new();
                    name.push(self.advance());
                    while let Some(next) = self.peek() {
                        if is_identifier_part(next) {
                            name.push(self.advance());
                        } else {
                            break;
                        }
                    }
                    parts.push(InterpolatedStringPart::Variable(name));
                    continue;
                }

                if matches!(self.peek_next(), Some('$' | '{')) {
                    return Err(self.error_at(
                        span,
                        "unsupported string interpolation: only simple $name interpolation in double-quoted strings is implemented; braced/complex interpolation is not implemented",
                    ));
                }
            }

            if interpolate && ch == '{' && self.peek_next() == Some('$') {
                return Err(self.error_at(
                    span,
                    "unsupported string interpolation: only simple $name interpolation in double-quoted strings is implemented; braced/complex interpolation is not implemented",
                ));
            }

            value.push(self.advance());
        }

        Err(self.error_at(span, "unterminated string literal"))
    }

    fn lex_number(&mut self, first: char, span: Span) -> CompileResult<TokenKind> {
        let mut text = String::new();
        text.push(first);

        if first == '0' && matches!(self.peek(), Some('x' | 'X')) {
            text.push(self.advance());
            let mut digits = String::new();
            while matches!(self.peek(), Some(ch) if ch.is_ascii_hexdigit()) {
                let ch = self.advance();
                text.push(ch);
                digits.push(ch);
            }
            if digits.is_empty() {
                return Err(self.error_at(span, format!("invalid integer literal '{text}'")));
            }
            let value = i64::from_str_radix(&digits, 16)
                .map_err(|_| self.error_at(span, format!("invalid integer literal '{text}'")))?;
            return Ok(TokenKind::Int(value));
        }

        while matches!(self.peek(), Some('0'..='9')) {
            text.push(self.advance());
        }

        if self.peek() == Some('.') && matches!(self.peek_next(), Some('0'..='9')) {
            text.push(self.advance());
            while matches!(self.peek(), Some('0'..='9')) {
                text.push(self.advance());
            }
            let value = text
                .parse::<f64>()
                .map_err(|_| self.error_at(span, format!("invalid float literal '{text}'")))?;
            return Ok(TokenKind::Float(value));
        }

        let value = text
            .parse::<i64>()
            .map_err(|_| self.error_at(span, format!("invalid integer literal '{text}'")))?;
        Ok(TokenKind::Int(value))
    }

    fn lex_identifier(&mut self, first: char) -> TokenKind {
        let mut text = String::new();
        text.push(first);

        while let Some(ch) = self.peek() {
            if is_identifier_part(ch) {
                text.push(self.advance());
            } else {
                break;
            }
        }

        match text.as_str() {
            "echo" => TokenKind::Echo,
            "print" => TokenKind::Print,
            "function" => TokenKind::Function,
            "fn" => TokenKind::Fn,
            "class" => TokenKind::Class,
            "interface" => TokenKind::Interface,
            "trait" => TokenKind::Trait,
            "enum" => TokenKind::Enum,
            "abstract" => TokenKind::Abstract,
            "final" => TokenKind::Final,
            "readonly" => TokenKind::Readonly,
            "new" => TokenKind::New,
            "public" => TokenKind::Public,
            "protected" => TokenKind::Protected,
            "private" => TokenKind::Private,
            "static" => TokenKind::Static,
            "extends" => TokenKind::Extends,
            "implements" => TokenKind::Implements,
            "clone" => TokenKind::Clone,
            "instanceof" => TokenKind::Instanceof,
            "return" => TokenKind::Return,
            "global" => TokenKind::Global,
            "namespace" => TokenKind::Namespace,
            "use" => TokenKind::Use,
            "declare" => TokenKind::Declare,
            "eval" => TokenKind::Eval,
            "include" => TokenKind::Include,
            "include_once" => TokenKind::IncludeOnce,
            "require" => TokenKind::Require,
            "require_once" => TokenKind::RequireOnce,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elseif" => TokenKind::ElseIf,
            "while" => TokenKind::While,
            "do" => TokenKind::Do,
            "foreach" => TokenKind::Foreach,
            "for" => TokenKind::For,
            "switch" => TokenKind::Switch,
            "match" => TokenKind::Match,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "throw" => TokenKind::Throw,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "null" | "NULL" => TokenKind::Null,
            "true" | "TRUE" => TokenKind::True,
            "false" | "FALSE" => TokenKind::False,
            _ => TokenKind::Identifier(text),
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.index];
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.source[self.byte_index()..].starts_with(needle)
    }

    fn byte_index(&self) -> usize {
        self.chars[..self.index]
            .iter()
            .map(|ch| ch.len_utf8())
            .sum()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.chars.len()
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.column)
    }

    fn error_at(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Lex, span.line, span.column, message)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_part(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}
