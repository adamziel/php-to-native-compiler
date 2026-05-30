use crate::ast::{InterpolatedAccessSegment, InterpolatedArrayKey, InterpolatedStringPart, Span};
use crate::error::{CompileResult, Diagnostic, Phase};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeToken {
    pub name: String,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Eof,
    Attribute(Vec<AttributeToken>),
    Dollar,
    Variable(String),
    Identifier(String),
    Int(i64),
    Float(f64),
    StringLiteral(String),
    InterpolatedString(Vec<InterpolatedStringPart>),
    DocComment(String),
    InlineHtml(String),
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
    NullsafeObjectOperator,
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
    At,
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

const PHP_ESCAPED_BYTE_SENTINEL_BASE: u32 = 0xE000;

fn php_escaped_byte_sentinel(byte: u8) -> char {
    char::from_u32(PHP_ESCAPED_BYTE_SENTINEL_BASE + u32::from(byte))
        .expect("PHP escaped byte sentinel must be a valid private-use scalar")
}

struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    index: usize,
    byte_index: usize,
    line: usize,
    column: usize,
    at_initial_php_boundary: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            index: 0,
            byte_index: 0,
            line: 1,
            column: 1,
            at_initial_php_boundary: true,
        }
    }

    fn tokenize(mut self) -> CompileResult<Vec<Token>> {
        let mut tokens = Vec::new();
        self.skip_initial_shebang_line();

        while !self.is_at_end() {
            if self.at_initial_php_boundary && !self.starts_with("<?") {
                let span = self.span();
                if let Some(kind) = self.lex_initial_inline_html_before_open_tag() {
                    tokens.push(Token { kind, span });
                    continue;
                }
            }

            self.skip_whitespace_and_comments()?;
            if self.is_at_end() {
                break;
            }

            if self.matches_php_open_tag() {
                self.at_initial_php_boundary = false;
                continue;
            }

            if self.starts_with("<?=") {
                return Err(self.error_at(self.span(), unsupported_short_echo_tag_message()));
            }

            if self.starts_with("?>") {
                self.at_initial_php_boundary = false;
                let span = self.span();
                if should_insert_close_tag_statement_terminator(&tokens) {
                    tokens.push(Token {
                        kind: TokenKind::Semicolon,
                        span,
                    });
                }
                if let Some(kind) = self.lex_inline_html_after_close_tag() {
                    tokens.push(Token { kind, span });
                }
                continue;
            }

            if self.starts_with("/**") && !self.starts_with("/**/") {
                let span = self.span();
                let kind = self.lex_doc_comment(span)?;
                tokens.push(Token { kind, span });
                continue;
            }

            if self.starts_with("#[") {
                let span = self.span();
                let kind = self.lex_attribute_block()?;
                tokens.push(Token { kind, span });
                continue;
            }

            let span = self.span();
            let ch = self.advance();
            let kind = match ch {
                '$' => self.lex_variable(span)?,
                '\'' | '"' => self.lex_string(ch, span)?,
                '0'..='9' => self.lex_number(ch, span)?,
                'b' | 'B' if matches!(self.peek(), Some('\'' | '"')) => {
                    let quote = self.advance();
                    self.lex_string(quote, span)?
                }
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
                    if matches!(self.peek(), Some('0'..='9')) {
                        self.lex_leading_dot_number(span)?
                    } else if self.match_char('.') {
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
                    } else if self.starts_with("->") {
                        self.advance();
                        self.advance();
                        TokenKind::NullsafeObjectOperator
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
                '@' => TokenKind::At,
                '`' => {
                    return Err(self.error_at(span, unsupported_backtick_operator_message()));
                }
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
                            self.lex_heredoc(span)?
                        } else {
                            TokenKind::LeftShift
                        }
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

            self.at_initial_php_boundary = false;
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
                    break;
                }
                while !matches!(self.peek(), None | Some('\n')) {
                    self.advance();
                }
                continue;
            }

            if self.peek() == Some('/') && self.peek_next() == Some('*') {
                if self.starts_with("/**") && !self.starts_with("/**/") {
                    break;
                }
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

    fn lex_doc_comment(&mut self, span: Span) -> CompileResult<TokenKind> {
        let start = self.byte_index;
        self.advance();
        self.advance();
        while !(self.peek() == Some('*') && self.peek_next() == Some('/')) {
            if self.is_at_end() {
                return Err(self.error_at(span, "unterminated block comment"));
            }
            self.advance();
        }
        self.advance();
        self.advance();
        Ok(TokenKind::DocComment(
            self.source[start..self.byte_index].to_string(),
        ))
    }

    fn skip_attribute_block(&mut self) -> CompileResult<()> {
        self.lex_attribute_block().map(|_| ())
    }

    fn lex_attribute_block(&mut self) -> CompileResult<TokenKind> {
        let start = self.span();
        self.advance();
        self.advance();
        let mut depth = 1usize;
        let mut content = String::new();

        while !self.is_at_end() {
            match self.advance() {
                '\'' | '"' => self.push_quoted_attribute_string(start, &mut content)?,
                '[' => {
                    depth += 1;
                    content.push('[');
                }
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        let attributes = parse_attribute_tokens(&content).ok_or_else(|| {
                            self.error_at(start, unsupported_attribute_syntax_message())
                        })?;
                        return Ok(TokenKind::Attribute(attributes));
                    }
                    content.push(']');
                }
                ch => content.push(ch),
            }
        }

        Err(self.error_at(
            start,
            "unterminated attribute syntax: expected ']' to close PHP attribute",
        ))
    }

    fn push_quoted_attribute_string(
        &mut self,
        start: Span,
        content: &mut String,
    ) -> CompileResult<()> {
        let quote = self.chars[self.index - 1];
        content.push(quote);
        while !self.is_at_end() {
            let ch = self.advance();
            content.push(ch);
            if ch == '\\' && !self.is_at_end() {
                content.push(self.advance());
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

    fn lex_inline_html_after_close_tag(&mut self) -> Option<TokenKind> {
        self.advance();
        self.advance();

        if self.peek() == Some('\r') {
            self.advance();
            if self.peek() == Some('\n') {
                self.advance();
            }
        } else if self.peek() == Some('\n') {
            self.advance();
        }

        let mut html = String::new();
        while !self.is_at_end() && !self.starts_with("<?") {
            html.push(self.advance());
        }

        if html.is_empty() {
            None
        } else {
            Some(TokenKind::InlineHtml(html))
        }
    }

    fn lex_initial_inline_html_before_open_tag(&mut self) -> Option<TokenKind> {
        if !self.source[self.byte_index()..].contains("<?") {
            return None;
        }

        let mut html = String::new();
        while !self.is_at_end() && !self.starts_with("<?") {
            html.push(self.advance());
        }

        if html.is_empty() {
            None
        } else {
            Some(TokenKind::InlineHtml(html))
        }
    }

    fn skip_initial_shebang_line(&mut self) {
        if self.index != 0 || !self.starts_with("#!") {
            return;
        }

        while !self.is_at_end() {
            if self.advance() == '\n' {
                break;
            }
        }
    }

    fn lex_variable(&mut self, span: Span) -> CompileResult<TokenKind> {
        let mut name = String::new();
        match self.peek() {
            Some('$' | '{') => return Ok(TokenKind::Dollar),
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
                if !interpolate {
                    match self.peek() {
                        Some('\\') => {
                            self.advance();
                            value.push('\\');
                        }
                        Some('\'') => {
                            self.advance();
                            value.push('\'');
                        }
                        Some(_) => {
                            value.push('\\');
                            value.push(self.advance());
                        }
                        None => return Err(self.error_at(span, "unterminated string literal")),
                    }
                    continue;
                }

                let Some(escaped) = self.peek() else {
                    return Err(self.error_at(span, "unterminated string literal"));
                };
                self.push_escaped_double_quoted_char(&mut value, escaped, true);
                continue;
            }

            if interpolate && ch == '$' {
                if matches!(self.peek_next(), Some(next) if is_identifier_start(next)) {
                    self.advance();
                    if !value.is_empty() {
                        parts.push(InterpolatedStringPart::Literal(value));
                        value = String::new();
                    }

                    let name = self.lex_identifier_name();
                    let part = self.lex_interpolated_suffix(name, span)?;
                    parts.push(part);
                    continue;
                }

                match self.peek_next() {
                    Some('$') if self.dollar_run_is_literal() => {
                        value.push(self.advance());
                        continue;
                    }
                    Some('{') => {
                        self.advance();
                        self.advance();
                        if !value.is_empty() {
                            parts.push(InterpolatedStringPart::Literal(value));
                            value = String::new();
                        }

                        let Some(first) = self.peek() else {
                            return Err(self.error_at(span, "unterminated string literal"));
                        };
                        if !is_identifier_start(first) {
                            return Err(
                                self.error_at(span, unsupported_string_interpolation_message())
                            );
                        }

                        let name = self.lex_identifier_name();
                        if self.peek() != Some('}') {
                            return Err(
                                self.error_at(span, unsupported_string_interpolation_message())
                            );
                        }
                        self.advance();
                        parts.push(InterpolatedStringPart::DeprecatedDollarBraceVariable(name));
                        continue;
                    }
                    Some('$') => {
                        return Err(self.error_at(span, unsupported_string_interpolation_message()));
                    }
                    _ => {}
                }
            }

            if interpolate && ch == '{' && self.peek_next() == Some('$') {
                self.advance();
                self.advance();
                if !value.is_empty() {
                    parts.push(InterpolatedStringPart::Literal(value));
                    value = String::new();
                }

                let Some(first) = self.peek() else {
                    return Err(self.error_at(span, "unterminated string literal"));
                };
                if !is_identifier_start(first) {
                    return Err(self.error_at(span, unsupported_string_interpolation_message()));
                }

                let name = self.lex_identifier_name();
                let part = self.lex_interpolated_suffix(name, span)?;
                if self.peek() != Some('}') {
                    return Err(self.error_at(span, unsupported_string_interpolation_message()));
                }
                self.advance();
                parts.push(part);
                continue;
            }

            value.push(self.advance());
        }

        Err(self.error_at(span, "unterminated string literal"))
    }

    fn lex_heredoc(&mut self, span: Span) -> CompileResult<TokenKind> {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.advance();
        }

        let interpolate = match self.peek() {
            Some('\'') => {
                self.advance();
                false
            }
            Some('"') => {
                self.advance();
                true
            }
            _ => true,
        };

        let Some(first) = self.peek() else {
            return Err(self.error_at(span, "unterminated heredoc/nowdoc string literal"));
        };
        if !is_identifier_start(first) {
            return Err(self.error_at(span, unsupported_heredoc_message()));
        }
        let label = self.lex_identifier_name();

        if !interpolate || self.peek() == Some('"') {
            let quote = if interpolate { '"' } else { '\'' };
            if self.peek() == Some(quote) {
                self.advance();
            } else if !interpolate {
                return Err(self.error_at(span, unsupported_heredoc_message()));
            }
        }

        while matches!(self.peek(), Some(' ' | '\t')) {
            self.advance();
        }

        if self.peek() == Some('\r') {
            self.advance();
        }
        if self.peek() != Some('\n') {
            return Err(self.error_at(span, unsupported_heredoc_message()));
        }
        self.advance();

        let mut value = String::new();
        let mut parts = Vec::new();

        while !self.is_at_end() {
            if self.at_heredoc_terminator(&label) {
                self.consume_heredoc_terminator(&label);
                trim_heredoc_final_newline(&mut value, &mut parts);
                if !parts.is_empty() {
                    if !value.is_empty() {
                        parts.push(InterpolatedStringPart::Literal(value));
                    }
                    return Ok(TokenKind::InterpolatedString(parts));
                }
                return Ok(TokenKind::StringLiteral(value));
            }

            let ch = self.peek().expect("checked not at end");
            if interpolate && ch == '\\' {
                self.advance();
                let Some(escaped) = self.peek() else {
                    return Err(self.error_at(span, "unterminated heredoc string literal"));
                };
                self.push_escaped_double_quoted_char(&mut value, escaped, false);
                continue;
            }

            if interpolate && ch == '$' {
                if matches!(self.peek_next(), Some(next) if is_identifier_start(next)) {
                    self.advance();
                    if !value.is_empty() {
                        parts.push(InterpolatedStringPart::Literal(value));
                        value = String::new();
                    }

                    let name = self.lex_identifier_name();
                    let part = self.lex_interpolated_suffix(name, span)?;
                    parts.push(part);
                    continue;
                }

                match self.peek_next() {
                    Some('$') if self.dollar_run_is_literal() => {
                        value.push(self.advance());
                        continue;
                    }
                    Some('{') => {
                        self.advance();
                        self.advance();
                        if !value.is_empty() {
                            parts.push(InterpolatedStringPart::Literal(value));
                            value = String::new();
                        }

                        let Some(first) = self.peek() else {
                            return Err(self.error_at(span, "unterminated heredoc string literal"));
                        };
                        if !is_identifier_start(first) {
                            return Err(
                                self.error_at(span, unsupported_string_interpolation_message())
                            );
                        }

                        let name = self.lex_identifier_name();
                        if self.peek() != Some('}') {
                            return Err(
                                self.error_at(span, unsupported_string_interpolation_message())
                            );
                        }
                        self.advance();
                        parts.push(InterpolatedStringPart::DeprecatedDollarBraceVariable(name));
                        continue;
                    }
                    Some('$') => {
                        return Err(self.error_at(span, unsupported_string_interpolation_message()));
                    }
                    _ => {}
                }
            }

            if interpolate && ch == '{' && self.peek_next() == Some('$') {
                self.advance();
                self.advance();
                if !value.is_empty() {
                    parts.push(InterpolatedStringPart::Literal(value));
                    value = String::new();
                }

                let Some(first) = self.peek() else {
                    return Err(self.error_at(span, "unterminated heredoc string literal"));
                };
                if !is_identifier_start(first) {
                    return Err(self.error_at(span, unsupported_string_interpolation_message()));
                }

                let name = self.lex_identifier_name();
                let part = self.lex_interpolated_suffix(name, span)?;
                if self.peek() != Some('}') {
                    return Err(self.error_at(span, unsupported_string_interpolation_message()));
                }
                self.advance();
                parts.push(part);
                continue;
            }

            value.push(self.advance());
        }

        Err(self.error_at(span, "unterminated heredoc/nowdoc string literal"))
    }

    fn consume_heredoc_terminator(&mut self, label: &str) {
        for _ in label.chars() {
            self.advance();
        }
    }

    fn at_heredoc_terminator(&self, label: &str) -> bool {
        if self.column != 1 || !self.starts_with(label) {
            return false;
        }
        let start = self.byte_index() + label.len();
        matches!(
            self.source[start..].chars().next(),
            None | Some(';' | ',' | ')' | ']' | '\r' | '\n')
        )
    }

    fn lex_identifier_name(&mut self) -> String {
        let mut name = String::new();
        name.push(self.advance());
        while let Some(next) = self.peek() {
            if is_identifier_part(next) {
                name.push(self.advance());
            } else {
                break;
            }
        }
        name
    }

    fn lex_interpolated_suffix(
        &mut self,
        name: String,
        span: Span,
    ) -> CompileResult<InterpolatedStringPart> {
        let mut segments = Vec::new();

        loop {
            if self.peek() == Some('[') {
                let key = self.lex_interpolated_array_key(span)?;
                segments.push(InterpolatedAccessSegment::ArrayOffset(key));
                continue;
            }

            if self.starts_with("->") {
                self.advance();
                self.advance();
                let Some(first) = self.peek() else {
                    return Err(self.error_at(span, "unterminated string literal"));
                };
                if !is_identifier_start(first) {
                    return Err(self.error_at(span, unsupported_string_interpolation_message()));
                }
                let property = self.lex_identifier_name();
                segments.push(InterpolatedAccessSegment::ObjectProperty(property));
                continue;
            }

            break;
        }

        if segments.is_empty() {
            return Ok(InterpolatedStringPart::Variable(name));
        }

        if segments.len() == 1 {
            return match segments.remove(0) {
                InterpolatedAccessSegment::ArrayOffset(key) => {
                    Ok(InterpolatedStringPart::ArrayOffset {
                        variable: name,
                        key,
                    })
                }
                InterpolatedAccessSegment::ObjectProperty(property) => {
                    Ok(InterpolatedStringPart::ObjectProperty {
                        variable: name,
                        property,
                    })
                }
            };
        }

        Ok(InterpolatedStringPart::AccessChain {
            variable: name,
            segments,
        })
    }

    fn lex_interpolated_array_key(&mut self, span: Span) -> CompileResult<InterpolatedArrayKey> {
        self.advance();
        let key = match self.peek() {
            Some('\'') | Some('"') => {
                let quote = self.advance();
                let mut value = String::new();
                while let Some(ch) = self.peek() {
                    if ch == quote {
                        self.advance();
                        break;
                    }
                    if ch == '\\' {
                        self.advance();
                        let Some(escaped) = self.peek() else {
                            return Err(self.error_at(span, "unterminated string literal"));
                        };
                        value.push(self.advance_escaped_string_char(escaped));
                    } else {
                        value.push(self.advance());
                    }
                }
                InterpolatedArrayKey::String(value)
            }
            Some('$') => {
                self.advance();
                let Some(first) = self.peek() else {
                    return Err(self.error_at(span, "unterminated string literal"));
                };
                if !is_identifier_start(first) {
                    return Err(self.error_at(span, unsupported_string_interpolation_message()));
                }
                InterpolatedArrayKey::Variable(self.lex_identifier_name())
            }
            Some(ch) if ch.is_ascii_digit() => {
                let mut value = String::new();
                value.push(self.advance());
                while let Some(next) = self.peek() {
                    if next.is_ascii_digit() {
                        value.push(self.advance());
                    } else {
                        break;
                    }
                }
                let value = value
                    .parse::<i64>()
                    .map_err(|_| self.error_at(span, unsupported_string_interpolation_message()))?;
                InterpolatedArrayKey::Int(value)
            }
            Some(ch) if is_identifier_start(ch) => {
                InterpolatedArrayKey::String(self.lex_identifier_name())
            }
            _ => return Err(self.error_at(span, unsupported_string_interpolation_message())),
        };

        if self.peek() != Some(']') {
            return Err(self.error_at(span, unsupported_string_interpolation_message()));
        }
        self.advance();
        Ok(key)
    }

    fn advance_escaped_string_char(&mut self, escaped: char) -> char {
        match escaped {
            'n' => {
                self.advance();
                '\n'
            }
            'r' => {
                self.advance();
                '\r'
            }
            't' => {
                self.advance();
                '\t'
            }
            'v' => {
                self.advance();
                '\u{000B}'
            }
            'f' => {
                self.advance();
                '\u{000C}'
            }
            'e' => {
                self.advance();
                '\u{001B}'
            }
            'x' => {
                self.advance();
                self.consume_hex_escape_byte()
                    .map(php_escaped_byte_sentinel)
                    .unwrap_or('x')
            }
            '0'..='7' => php_escaped_byte_sentinel(self.consume_octal_escape_byte()),
            other => {
                self.advance();
                other
            }
        }
    }

    fn push_escaped_double_quoted_char(
        &mut self,
        value: &mut String,
        escaped: char,
        recognize_quote: bool,
    ) {
        match escaped {
            'n' => {
                self.advance();
                value.push('\n');
            }
            'r' => {
                self.advance();
                value.push('\r');
            }
            't' => {
                self.advance();
                value.push('\t');
            }
            'v' => {
                self.advance();
                value.push('\u{000B}');
            }
            'f' => {
                self.advance();
                value.push('\u{000C}');
            }
            'e' => {
                self.advance();
                value.push('\u{001B}');
            }
            'x' if self
                .chars
                .get(self.index + 1)
                .is_some_and(|ch| ch.is_ascii_hexdigit()) =>
            {
                self.advance();
                if let Some(byte) = self.consume_hex_escape_byte() {
                    value.push(php_escaped_byte_sentinel(byte));
                }
            }
            '0'..='7' => value.push(php_escaped_byte_sentinel(self.consume_octal_escape_byte())),
            '\\' => {
                self.advance();
                value.push('\\');
            }
            '$' => {
                self.advance();
                value.push('$');
            }
            '"' if recognize_quote => {
                self.advance();
                value.push('"');
            }
            other => {
                self.advance();
                value.push('\\');
                value.push(other);
            }
        }
    }

    fn consume_hex_escape_byte(&mut self) -> Option<u8> {
        let mut value = 0u8;
        let mut digits = 0;
        while digits < 2 {
            let Some(ch) = self.peek() else {
                break;
            };
            let Some(digit) = ch.to_digit(16) else {
                break;
            };
            self.advance();
            value = (value << 4) | digit as u8;
            digits += 1;
        }
        (digits > 0).then_some(value)
    }

    fn consume_octal_escape_byte(&mut self) -> u8 {
        let mut value = 0u16;
        let mut digits = 0;
        while digits < 3 {
            let Some(ch @ '0'..='7') = self.peek() else {
                break;
            };
            self.advance();
            value = (value << 3) | (ch as u16 - '0' as u16);
            digits += 1;
        }
        value as u8
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

        if first == '0' && matches!(self.peek(), Some('0'..='9')) {
            let mut digits = String::from("0");
            while matches!(self.peek(), Some('0'..='9')) {
                let ch = self.advance();
                text.push(ch);
                digits.push(ch);
            }
            if digits.bytes().any(|byte| !matches!(byte, b'0'..=b'7')) {
                return Err(self.error_at(span, format!("invalid integer literal '{text}'")));
            }
            let value = i64::from_str_radix(&digits, 8)
                .map_err(|_| self.error_at(span, format!("invalid integer literal '{text}'")))?;
            return Ok(TokenKind::Int(value));
        }

        while matches!(self.peek(), Some('0'..='9')) {
            text.push(self.advance());
        }

        let mut is_float = false;
        if self.peek() == Some('.') && matches!(self.peek_next(), Some('0'..='9')) {
            is_float = true;
            text.push(self.advance());
            while matches!(self.peek(), Some('0'..='9')) {
                text.push(self.advance());
            }
        }

        if self.peek().is_some_and(|ch| matches!(ch, 'e' | 'E'))
            && (self
                .chars
                .get(self.index + 1)
                .is_some_and(|ch| ch.is_ascii_digit())
                || (self
                    .chars
                    .get(self.index + 1)
                    .is_some_and(|ch| matches!(*ch, '+' | '-'))
                    && self
                        .chars
                        .get(self.index + 2)
                        .is_some_and(|ch| ch.is_ascii_digit())))
        {
            is_float = true;
            text.push(self.advance());
            if self.peek().is_some_and(|ch| matches!(ch, '+' | '-')) {
                text.push(self.advance());
            }
            while matches!(self.peek(), Some('0'..='9')) {
                text.push(self.advance());
            }
        }

        if is_float {
            let value = text
                .parse::<f64>()
                .map_err(|_| self.error_at(span, format!("invalid float literal '{text}'")))?;
            return Ok(TokenKind::Float(value));
        }

        match text.parse::<i64>() {
            Ok(value) => Ok(TokenKind::Int(value)),
            Err(_) => {
                let value = text.parse::<f64>().map_err(|_| {
                    self.error_at(span, format!("invalid integer literal '{text}'"))
                })?;
                Ok(TokenKind::Float(value))
            }
        }
    }

    fn lex_leading_dot_number(&mut self, span: Span) -> CompileResult<TokenKind> {
        let mut text = String::from(".");
        while matches!(self.peek(), Some('0'..='9')) {
            text.push(self.advance());
        }

        if self.peek().is_some_and(|ch| matches!(ch, 'e' | 'E'))
            && (self
                .chars
                .get(self.index + 1)
                .is_some_and(|ch| ch.is_ascii_digit())
                || (self
                    .chars
                    .get(self.index + 1)
                    .is_some_and(|ch| matches!(*ch, '+' | '-'))
                    && self
                        .chars
                        .get(self.index + 2)
                        .is_some_and(|ch| ch.is_ascii_digit())))
        {
            text.push(self.advance());
            if self.peek().is_some_and(|ch| matches!(ch, '+' | '-')) {
                text.push(self.advance());
            }
            while matches!(self.peek(), Some('0'..='9')) {
                text.push(self.advance());
            }
        }

        let value = text
            .parse::<f64>()
            .map_err(|_| self.error_at(span, format!("invalid float literal '{text}'")))?;
        Ok(TokenKind::Float(value))
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

        match text.to_ascii_lowercase().as_str() {
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
            keyword if keyword.eq_ignore_ascii_case("null") => TokenKind::Null,
            keyword if keyword.eq_ignore_ascii_case("true") => TokenKind::True,
            keyword if keyword.eq_ignore_ascii_case("false") => TokenKind::False,
            _ => TokenKind::Identifier(text),
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.index];
        self.index += 1;
        self.byte_index += ch.len_utf8();
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
        self.byte_index
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn dollar_run_is_literal(&self) -> bool {
        let mut lookahead = self.index;
        while matches!(self.chars.get(lookahead), Some('$')) {
            lookahead += 1;
        }

        !matches!(
            self.chars.get(lookahead).copied(),
            Some(next) if is_identifier_start(next) || next == '{'
        )
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

fn should_insert_close_tag_statement_terminator(tokens: &[Token]) -> bool {
    let Some(previous) = tokens.last() else {
        return false;
    };

    match &previous.kind {
        TokenKind::Variable(_)
        | TokenKind::Identifier(_)
        | TokenKind::Int(_)
        | TokenKind::Float(_)
        | TokenKind::StringLiteral(_)
        | TokenKind::InterpolatedString(_)
        | TokenKind::Null
        | TokenKind::True
        | TokenKind::False
        | TokenKind::RParen
        | TokenKind::RBracket
        | TokenKind::Return
        | TokenKind::Break
        | TokenKind::Continue => true,
        TokenKind::Plus => matches!(
            tokens
                .get(tokens.len().saturating_sub(2))
                .map(|token| &token.kind),
            Some(TokenKind::Plus)
        ),
        TokenKind::Minus => matches!(
            tokens
                .get(tokens.len().saturating_sub(2))
                .map(|token| &token.kind),
            Some(TokenKind::Minus)
        ),
        _ => false,
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_part(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

fn unsupported_string_interpolation_message() -> &'static str {
    "unsupported string interpolation: only simple $name, {$name}, ${name}, array offsets, and object properties in double-quoted strings are implemented; variable variables, dynamic properties, static properties, arbitrary expressions, and complex interpolation are not implemented"
}

fn unsupported_heredoc_message() -> &'static str {
    "unsupported heredoc/nowdoc string syntax: only unindented identifier labels are implemented; indentation stripping, label expressions, and malformed labels are not implemented"
}

fn unsupported_short_echo_tag_message() -> &'static str {
    "unsupported short echo tag: <?= is not implemented; use <?php echo ... ?> in the current subset"
}

fn unsupported_backtick_operator_message() -> &'static str {
    "unsupported backtick execution operator: shell command execution, interpolation, process I/O, error handling, platform behavior, references/copy-on-write, and native lowering are not implemented"
}

fn unsupported_attribute_syntax_message() -> &'static str {
    "unsupported PHP attribute syntax: expected comma-separated attribute names with optional balanced constructor arguments"
}

fn parse_attribute_tokens(content: &str) -> Option<Vec<AttributeToken>> {
    let mut attributes = Vec::new();
    for raw_part in split_top_level_attribute_items(content)? {
        let part = raw_part.trim();
        if part.is_empty() {
            return None;
        }
        let name_end = attribute_name_end(part)?;
        let name = part[..name_end].trim();
        if !is_simple_attribute_name(name) {
            return None;
        }
        let rest = part[name_end..].trim();
        let arguments = if rest.is_empty() {
            None
        } else if is_balanced_attribute_arguments(rest) {
            Some(rest.to_string())
        } else {
            return None;
        };
        attributes.push(AttributeToken {
            name: name.to_string(),
            arguments,
        });
    }
    (!attributes.is_empty()).then_some(attributes)
}

fn split_top_level_attribute_items(content: &str) -> Option<Vec<&str>> {
    let mut items = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut chars = content.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        match ch {
            '\'' | '"' => skip_quoted_attribute_chars(&mut chars, ch)?,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                items.push(&content[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    items.push(&content[start..]);
    Some(items)
}

fn skip_quoted_attribute_chars<I>(chars: &mut std::iter::Peekable<I>, quote: char) -> Option<()>
where
    I: Iterator<Item = (usize, char)>,
{
    while let Some((_, ch)) = chars.next() {
        if ch == '\\' {
            chars.next();
            continue;
        }
        if ch == quote {
            return Some(());
        }
    }
    None
}

fn attribute_name_end(part: &str) -> Option<usize> {
    let mut end = None;
    for (index, ch) in part.char_indices() {
        if ch == '\\' || ch == '_' || ch.is_ascii_alphanumeric() {
            end = Some(index + ch.len_utf8());
            continue;
        }
        break;
    }
    end
}

fn is_balanced_attribute_arguments(arguments: &str) -> bool {
    if !arguments.starts_with('(') || !arguments.ends_with(')') {
        return false;
    }
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut chars = arguments.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        match ch {
            '\'' | '"' => {
                if skip_quoted_attribute_chars(&mut chars, ch).is_none() {
                    return false;
                }
            }
            '(' => paren_depth += 1,
            ')' => {
                let Some(next_depth) = paren_depth.checked_sub(1) else {
                    return false;
                };
                paren_depth = next_depth;
                if paren_depth == 0 && chars.peek().is_some() {
                    return false;
                }
            }
            '[' => bracket_depth += 1,
            ']' => {
                let Some(next_depth) = bracket_depth.checked_sub(1) else {
                    return false;
                };
                bracket_depth = next_depth;
            }
            _ => {}
        }
    }
    paren_depth == 0 && bracket_depth == 0
}

fn is_simple_attribute_name(name: &str) -> bool {
    let mut saw_identifier_part = false;
    let mut expect_identifier_start = true;
    let mut previous_was_separator = true;
    let mut chars = name.chars().peekable();
    if matches!(chars.peek(), Some('\\')) {
        chars.next();
        previous_was_separator = true;
        expect_identifier_start = true;
    }
    for ch in chars {
        if ch == '\\' {
            if previous_was_separator || expect_identifier_start {
                return false;
            }
            previous_was_separator = true;
            expect_identifier_start = true;
            continue;
        }
        if expect_identifier_start {
            if ch == '_' || ch.is_ascii_alphabetic() {
                saw_identifier_part = true;
                previous_was_separator = false;
                expect_identifier_start = false;
                continue;
            }
            return false;
        }
        if ch == '_' || ch.is_ascii_alphanumeric() {
            saw_identifier_part = true;
            previous_was_separator = false;
            continue;
        }
        return false;
    }
    saw_identifier_part && !previous_was_separator
}

fn trim_heredoc_final_newline(value: &mut String, parts: &mut [InterpolatedStringPart]) {
    if trim_one_line_ending(value) {
        return;
    }

    if let Some(InterpolatedStringPart::Literal(last)) = parts.last_mut() {
        trim_one_line_ending(last);
    }
}

fn trim_one_line_ending(value: &mut String) -> bool {
    if !value.ends_with('\n') {
        return false;
    }
    value.pop();
    if value.ends_with('\r') {
        value.pop();
    }
    true
}
