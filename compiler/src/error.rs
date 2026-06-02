use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Lex,
    Parse,
    Runtime,
    Codegen,
    Io,
    Cli,
    Test,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Phase::Lex => "lex",
            Phase::Parse => "parse",
            Phase::Runtime => "runtime",
            Phase::Codegen => "codegen",
            Phase::Io => "io",
            Phase::Cli => "cli",
            Phase::Test => "test",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub phase: Phase,
    pub file: Option<PathBuf>,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn new(phase: Phase, line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            phase,
            file: None,
            line,
            column,
            message: message.into(),
        }
    }

    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn cli_display(&self) -> String {
        if matches!(self.phase, Phase::Parse) && self.message.starts_with("syntax error,") {
            let file = self
                .file
                .as_ref()
                .map(|file| file.display().to_string())
                .unwrap_or_else(|| "Command line code".to_string());
            return format!(
                "Parse error: {} in {file} on line {}",
                self.message, self.line
            );
        }

        self.to_string()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file {
            Some(file) => write!(
                f,
                "{} error at {}:{}:{}: {}",
                self.phase,
                file.display(),
                self.line,
                self.column,
                self.message
            ),
            None => write!(
                f,
                "{} error at {}:{}: {}",
                self.phase, self.line, self.column, self.message
            ),
        }
    }
}

impl std::error::Error for Diagnostic {}

pub type CompileResult<T> = Result<T, Diagnostic>;
