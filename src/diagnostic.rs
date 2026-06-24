use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub end_line: usize,
    pub column: usize,
}

impl SourceSpan {
    pub fn new(byte_start: usize, byte_end: usize, line: usize, column: usize) -> Self {
        Self {
            byte_start,
            byte_end,
            line,
            end_line: line,
            column,
        }
    }

    pub fn new_with_end_line(
        byte_start: usize,
        byte_end: usize,
        line: usize,
        end_line: usize,
        column: usize,
    ) -> Self {
        Self {
            byte_start,
            byte_end,
            line,
            end_line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<SourceSpan>,
    pub kind: DiagnosticKind,
    pub uncaught: Option<UncaughtFatal>,
    pub notices: Vec<DiagnosticNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncaughtFatal {
    pub throwable: String,
    pub call_frame: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticNotice {
    pub message: String,
    pub span: SourceSpan,
    pub kind: DiagnosticNoticeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticNoticeKind {
    Warning,
    UncaughtError,
    Deprecation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    Fatal,
    ParseError,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            message: message.into(),
            span,
            kind: DiagnosticKind::Fatal,
            uncaught: None,
            notices: Vec::new(),
        }
    }

    pub fn parse_error(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            message: message.into(),
            span,
            kind: DiagnosticKind::ParseError,
            uncaught: None,
            notices: Vec::new(),
        }
    }

    pub fn uncaught_fatal(
        throwable: impl Into<String>,
        message: impl Into<String>,
        span: Option<SourceSpan>,
        call_frame: Option<String>,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind: DiagnosticKind::Fatal,
            uncaught: Some(UncaughtFatal {
                throwable: throwable.into(),
                call_frame,
            }),
            notices: Vec::new(),
        }
    }

    pub fn with_notices(mut self, notices: Vec<DiagnosticNotice>) -> Self {
        self.notices = notices;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(f, "{} at {}:{}", self.message, span.line, span.column),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for Diagnostic {}

pub type Result<T> = std::result::Result<T, Diagnostic>;
