use crate::diagnostic::{Diagnostic, Result, SourceSpan};

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
    Case,
    Default,
    Break,
    Continue,
    Return,
    Try,
    Catch,
    Goto,
    Const,
    Function,
    Identifier(String),
    String(String),
    InterpolatedString(Vec<StringPart>),
    Int(i64),
    Float(f64),
    True,
    False,
    Null,
    Variable(String),
    Equal,
    DoubleArrow,
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
    Caret,
    Tilde,
    Bang,
    Backslash,
    Dot,
    Comma,
    Question,
    QuestionQuestion,
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
                '?' if self.rest().starts_with("??") => {
                    self.push_fixed(TokenKind::QuestionQuestion, 2)
                }
                '?' => self.push_fixed(TokenKind::Question, 1),
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
                '<' if self.rest().starts_with("<=>") => self.push_fixed(TokenKind::Spaceship, 3),
                '<' if self.rest().starts_with("<<=") => {
                    self.push_fixed(TokenKind::ShiftLeftEqual, 3)
                }
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
                '\\' => self.push_fixed(TokenKind::Backslash, 1),
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
                return Err(Diagnostic::new(
                    "inline HTML between PHP blocks is unsupported",
                    Some(self.current_span(5)),
                ));
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
                    '\\' => literal.push('\\'),
                    '"' => literal.push('"'),
                    '$' => literal.push('$'),
                    other => {
                        literal.push('\\');
                        literal.push(other);
                    }
                }
                self.bump_char();
                continue;
            }

            if ch == '$' {
                self.bump_char();
                if let Some(first) = self.peek_char() {
                    if is_ident_start(first) {
                        if literal.ends_with('{') {
                            return Err(Diagnostic::new(
                                "complex string interpolation is unsupported",
                                Some(self.current_span(1)),
                            ));
                        }
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
                        parts.push(StringPart::Variable(name));
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

    fn lex_number(&mut self) -> Result<()> {
        let start = self.current_span(0);
        let mut text = String::new();

        if self.starts_radix_integer_prefix("0x", |ch| ch.is_ascii_hexdigit())
            || self.starts_radix_integer_prefix("0X", |ch| ch.is_ascii_hexdigit())
        {
            self.bump_char();
            self.bump_char();
            self.collect_digits(&mut text, |ch| ch.is_ascii_hexdigit());
            let value = i64::from_str_radix(&text, 16)
                .map_err(|_| Diagnostic::new("invalid integer literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Int(value),
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
            let value = i64::from_str_radix(&text, 2)
                .map_err(|_| Diagnostic::new("invalid integer literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Int(value),
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
            let value = i64::from_str_radix(&text, 8)
                .map_err(|_| Diagnostic::new("invalid integer literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Int(value),
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
            let value = if text.len() > 1 && text.starts_with('0') {
                if text.bytes().any(|digit| matches!(digit, b'8' | b'9')) {
                    return Err(Diagnostic::parse_error(
                        "Invalid numeric literal",
                        Some(start),
                    ));
                }
                i64::from_str_radix(&text, 8)
            } else {
                text.parse::<i64>()
            }
            .map_err(|_| Diagnostic::new("invalid integer literal", Some(start)))?;
            self.tokens.push(Token {
                kind: TokenKind::Int(value),
                span: SourceSpan::new(start.byte_start, self.cursor, start.line, start.column),
            });
        }
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
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "goto" => TokenKind::Goto,
            "const" => TokenKind::Const,
            "function" => TokenKind::Function,
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
