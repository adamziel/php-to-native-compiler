use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions, Diagnostic, DiagnosticKind};

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(255);
        }
    }
}

fn run() -> Result<i32, PhpcError> {
    let invocation = Invocation::parse(std::env::args().skip(1))?;
    let ini = invocation.ini;
    match invocation.mode {
        Mode::Version => {
            println!("PHP 8.4.0 (ptn phpc)");
            Ok(0)
        }
        Mode::Modules => Ok(0),
        Mode::Script { script, args } => compile_and_run(&script, &args, &ini),
        Mode::Inline { source } => {
            let temp = TempPath::new("ptn-phpc-inline", "php");
            let source = if source.trim_start().starts_with("<?") {
                source
            } else {
                format!("<?php {source}")
            };
            fs::write(temp.path(), source)
                .map_err(|error| format!("failed to write inline source: {error}"))?;
            compile_and_run(temp.path(), &[], &ini)
        }
    }
}

#[derive(Debug)]
enum PhpcError {
    Message(String),
    SourceFatal {
        diagnostic: Diagnostic,
        script: PathBuf,
    },
}

impl std::fmt::Display for PhpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhpcError::Message(message) => write!(f, "phpc: {message}"),
            PhpcError::SourceFatal { diagnostic, script } => match diagnostic.span {
                Some(span) => write!(
                    f,
                    "{}: {} in {} on line {}",
                    match diagnostic.kind {
                        DiagnosticKind::Fatal => "Fatal error",
                        DiagnosticKind::ParseError => "Parse error",
                    },
                    diagnostic.message,
                    script.display(),
                    span.line
                ),
                None => write!(f, "phpc: {diagnostic}"),
            },
        }
    }
}

impl From<String> for PhpcError {
    fn from(message: String) -> Self {
        PhpcError::Message(message)
    }
}

#[derive(Debug)]
struct Invocation {
    mode: Mode,
    ini: RuntimeIni,
}

#[derive(Debug)]
enum Mode {
    Version,
    Modules,
    Script { script: PathBuf, args: Vec<String> },
    Inline { source: String },
}

#[derive(Debug, Default)]
struct RuntimeIni {
    precision: Option<u8>,
    assert_exception: Option<String>,
    display_errors: Option<String>,
    error_reporting: Option<i64>,
    memory_limit: Option<String>,
    max_memory_limit: Option<String>,
    zend_assertions: Option<String>,
}

impl Invocation {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter().peekable();
        let mut script = None;
        let mut script_args = Vec::new();
        let mut ini = RuntimeIni::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-q" | "-n" => {}
                "-v" | "--version" => {
                    return Ok(Self {
                        mode: Mode::Version,
                        ini,
                    });
                }
                "-m" => {
                    return Ok(Self {
                        mode: Mode::Modules,
                        ini,
                    });
                }
                "-d" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("missing value for {arg}"))?;
                    apply_ini_setting(&value, &mut ini);
                }
                "-c" => {
                    args.next()
                        .ok_or_else(|| format!("missing value for {arg}"))?;
                }
                "-f" => {
                    let path = args
                        .next()
                        .ok_or_else(|| "missing value for -f".to_string())?;
                    script = Some(PathBuf::from(path));
                    break;
                }
                "-r" => {
                    let source = args
                        .next()
                        .ok_or_else(|| "missing inline source for -r".to_string())?;
                    return Ok(Self {
                        mode: Mode::Inline { source },
                        ini,
                    });
                }
                "run" if script.is_none() => {
                    let path = args
                        .next()
                        .ok_or_else(|| "missing script path after run".to_string())?;
                    script = Some(PathBuf::from(path));
                    script_args.extend(args);
                    break;
                }
                "--" => {
                    if let Some(path) = args.next() {
                        script = Some(PathBuf::from(path));
                        script_args.extend(args);
                    }
                    break;
                }
                _ if let Some(value) = arg.strip_prefix("-d") => {
                    apply_ini_setting(value, &mut ini);
                }
                _ if arg.starts_with("-c") => {}
                _ if arg.starts_with('-') => {}
                _ => {
                    script = Some(PathBuf::from(arg));
                    script_args.extend(args);
                    break;
                }
            }
        }

        let script = script.ok_or_else(usage)?;
        Ok(Self {
            mode: Mode::Script {
                script,
                args: script_args,
            },
            ini,
        })
    }
}

fn apply_ini_setting(value: &str, ini: &mut RuntimeIni) {
    let Some((name, raw_value)) = value.split_once('=') else {
        return;
    };
    let name = name.trim();
    let raw_value = raw_value.trim();
    if name.eq_ignore_ascii_case("precision") {
        if let Ok(parsed) = raw_value.parse::<u8>() {
            if parsed <= 53 {
                ini.precision = Some(parsed);
            }
        }
    } else if name.eq_ignore_ascii_case("assert.exception") {
        ini.assert_exception = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("display_errors") {
        ini.display_errors = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("error_reporting") {
        if let Some(parsed) = parse_error_reporting_value(raw_value.trim()) {
            ini.error_reporting = Some(parsed);
        }
    } else if name.eq_ignore_ascii_case("memory_limit") {
        ini.memory_limit = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("max_memory_limit") {
        ini.max_memory_limit = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("zend.assertions") {
        ini.zend_assertions = Some(normalize_ini_scalar(raw_value));
    }
}

fn normalize_ini_scalar(raw_value: &str) -> String {
    let trimmed = raw_value.trim();
    if trimmed.eq_ignore_ascii_case("false")
        || trimmed.eq_ignore_ascii_case("off")
        || trimmed.eq_ignore_ascii_case("no")
    {
        String::new()
    } else if trimmed.eq_ignore_ascii_case("true")
        || trimmed.eq_ignore_ascii_case("on")
        || trimmed.eq_ignore_ascii_case("yes")
    {
        "1".to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrorReportingToken {
    Value(i64),
    And,
    Or,
    Xor,
    Not,
    LParen,
    RParen,
}

struct ErrorReportingParser {
    tokens: Vec<ErrorReportingToken>,
    pos: usize,
}

fn parse_error_reporting_value(value: &str) -> Option<i64> {
    if let Ok(parsed) = value.parse::<i64>() {
        return Some(parsed);
    }
    let tokens = tokenize_error_reporting(value)?;
    if tokens.is_empty() {
        return None;
    }
    let mut parser = ErrorReportingParser { tokens, pos: 0 };
    let result = parser.parse_or()?;
    (parser.pos == parser.tokens.len()).then_some(result)
}

fn tokenize_error_reporting(value: &str) -> Option<Vec<ErrorReportingToken>> {
    let mut tokens = Vec::new();
    let mut chars = value.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => {}
            '&' => tokens.push(ErrorReportingToken::And),
            '|' => tokens.push(ErrorReportingToken::Or),
            '^' => tokens.push(ErrorReportingToken::Xor),
            '~' => tokens.push(ErrorReportingToken::Not),
            '(' => tokens.push(ErrorReportingToken::LParen),
            ')' => tokens.push(ErrorReportingToken::RParen),
            '-' | '0'..='9' => {
                let mut end = start + ch.len_utf8();
                while let Some(&(idx, next)) = chars.peek() {
                    if next.is_ascii_digit() {
                        chars.next();
                        end = idx + next.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(ErrorReportingToken::Value(
                    value[start..end].parse::<i64>().ok()?,
                ));
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                let mut end = start + ch.len_utf8();
                while let Some(&(idx, next)) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        chars.next();
                        end = idx + next.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(ErrorReportingToken::Value(error_reporting_constant(
                    &value[start..end],
                )?));
            }
            _ => return None,
        }
    }
    Some(tokens)
}

fn error_reporting_constant(name: &str) -> Option<i64> {
    if name.eq_ignore_ascii_case("E_ERROR") {
        Some(1)
    } else if name.eq_ignore_ascii_case("E_WARNING") {
        Some(2)
    } else if name.eq_ignore_ascii_case("E_PARSE") {
        Some(4)
    } else if name.eq_ignore_ascii_case("E_NOTICE") {
        Some(8)
    } else if name.eq_ignore_ascii_case("E_CORE_ERROR") {
        Some(16)
    } else if name.eq_ignore_ascii_case("E_CORE_WARNING") {
        Some(32)
    } else if name.eq_ignore_ascii_case("E_COMPILE_ERROR") {
        Some(64)
    } else if name.eq_ignore_ascii_case("E_COMPILE_WARNING") {
        Some(128)
    } else if name.eq_ignore_ascii_case("E_USER_ERROR") {
        Some(256)
    } else if name.eq_ignore_ascii_case("E_USER_WARNING") {
        Some(512)
    } else if name.eq_ignore_ascii_case("E_USER_NOTICE") {
        Some(1024)
    } else if name.eq_ignore_ascii_case("E_STRICT") {
        Some(2048)
    } else if name.eq_ignore_ascii_case("E_RECOVERABLE_ERROR") {
        Some(4096)
    } else if name.eq_ignore_ascii_case("E_DEPRECATED") {
        Some(8192)
    } else if name.eq_ignore_ascii_case("E_USER_DEPRECATED") {
        Some(16384)
    } else if name.eq_ignore_ascii_case("E_ALL") {
        Some(32767)
    } else {
        None
    }
}

impl ErrorReportingParser {
    fn parse_or(&mut self) -> Option<i64> {
        let mut value = self.parse_xor()?;
        while self.consume(ErrorReportingToken::Or) {
            value |= self.parse_xor()?;
        }
        Some(value)
    }

    fn parse_xor(&mut self) -> Option<i64> {
        let mut value = self.parse_and()?;
        while self.consume(ErrorReportingToken::Xor) {
            value ^= self.parse_and()?;
        }
        Some(value)
    }

    fn parse_and(&mut self) -> Option<i64> {
        let mut value = self.parse_unary()?;
        while self.consume(ErrorReportingToken::And) {
            value &= self.parse_unary()?;
        }
        Some(value)
    }

    fn parse_unary(&mut self) -> Option<i64> {
        if self.consume(ErrorReportingToken::Not) {
            Some(!self.parse_unary()?)
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Option<i64> {
        match self.tokens.get(self.pos).copied()? {
            ErrorReportingToken::Value(value) => {
                self.pos += 1;
                Some(value)
            }
            ErrorReportingToken::LParen => {
                self.pos += 1;
                let value = self.parse_or()?;
                self.consume(ErrorReportingToken::RParen).then_some(value)
            }
            _ => None,
        }
    }

    fn consume(&mut self, token: ErrorReportingToken) -> bool {
        if self.tokens.get(self.pos) == Some(&token) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

fn compile_and_run(script: &Path, args: &[String], ini: &RuntimeIni) -> Result<i32, PhpcError> {
    let native = TempPath::new("ptn-phpc-native", "bin");
    compile_file(script, native.path(), CompileOptions { emit_c: false }).map_err(|error| {
        if error.span.is_some() {
            PhpcError::SourceFatal {
                diagnostic: error,
                script: script.to_path_buf(),
            }
        } else {
            PhpcError::Message(error.to_string())
        }
    })?;

    let mut command = Command::new(native.path());
    command.args(args);
    if let Some(precision) = ini.precision {
        command.env("PTN_PHP_PRECISION", precision.to_string());
    }
    if let Some(assert_exception) = &ini.assert_exception {
        command.env("PTN_ASSERT_EXCEPTION", assert_exception);
    }
    if let Some(display_errors) = &ini.display_errors {
        command.env("PTN_PHP_DISPLAY_ERRORS", display_errors);
    }
    if let Some(error_reporting) = ini.error_reporting {
        command.env("PTN_PHP_ERROR_REPORTING", error_reporting.to_string());
    }
    if let Some(memory_limit) = &ini.memory_limit {
        command.env("PTN_PHP_MEMORY_LIMIT", memory_limit);
    }
    if let Some(max_memory_limit) = &ini.max_memory_limit {
        command.env("PTN_PHP_MAX_MEMORY_LIMIT", max_memory_limit);
    }
    if let Some(zend_assertions) = &ini.zend_assertions {
        command.env("PTN_ZEND_ASSERTIONS", zend_assertions);
    }
    let status = command
        .status()
        .map_err(|error| PhpcError::Message(format!("failed to run native binary: {error}")))?;
    Ok(status.code().unwrap_or(255))
}

struct TempPath {
    path: PathBuf,
}

impl TempPath {
    fn new(prefix: &str, extension: &str) -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        path.push(format!(
            "{prefix}-{}-{nanos}.{extension}",
            std::process::id()
        ));
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
        let c_path = self.path.with_extension("c");
        match fs::remove_file(c_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

fn usage() -> String {
    "usage: phpc [-q] [-n] [-d key=value] [-c php.ini] [-r code] [-f] [run] <script.php>"
        .to_string()
}
