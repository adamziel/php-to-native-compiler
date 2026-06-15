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
    precision: Option<i16>,
    date_timezone: Option<String>,
    assert_exception: Option<String>,
    display_errors: Option<String>,
    error_reporting: Option<i64>,
    zend_assertions: Option<String>,
    memory_limit: Option<String>,
    max_memory_limit: Option<String>,
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
        if let Ok(parsed) = raw_value.parse::<i16>() {
            if (-1..=53).contains(&parsed) {
                ini.precision = Some(parsed);
            }
        }
    } else if name.eq_ignore_ascii_case("assert.exception") {
        ini.assert_exception = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("date.timezone") {
        ini.date_timezone = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("display_errors") {
        ini.display_errors = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("error_reporting") {
        if let Some(parsed) = parse_error_reporting_level(raw_value) {
            ini.error_reporting = Some(parsed);
        }
    } else if name.eq_ignore_ascii_case("zend.assertions") {
        ini.zend_assertions = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("memory_limit") {
        ini.memory_limit = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("max_memory_limit") {
        ini.max_memory_limit = Some(raw_value.to_string());
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

fn parse_error_reporting_level(raw_value: &str) -> Option<i64> {
    ErrorReportingParser::new(raw_value).parse()
}

struct ErrorReportingParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> ErrorReportingParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse(mut self) -> Option<i64> {
        let value = self.parse_or()?;
        self.skip_whitespace();
        if self.position == self.input.len() {
            Some(value)
        } else {
            None
        }
    }

    fn parse_or(&mut self) -> Option<i64> {
        let mut value = self.parse_xor()?;
        loop {
            self.skip_whitespace();
            if !self.consume_byte(b'|') {
                break;
            }
            value |= self.parse_xor()?;
        }
        Some(value)
    }

    fn parse_xor(&mut self) -> Option<i64> {
        let mut value = self.parse_and()?;
        loop {
            self.skip_whitespace();
            if !self.consume_byte(b'^') {
                break;
            }
            value ^= self.parse_and()?;
        }
        Some(value)
    }

    fn parse_and(&mut self) -> Option<i64> {
        let mut value = self.parse_unary()?;
        loop {
            self.skip_whitespace();
            if !self.consume_byte(b'&') {
                break;
            }
            value &= self.parse_unary()?;
        }
        Some(value)
    }

    fn parse_unary(&mut self) -> Option<i64> {
        self.skip_whitespace();
        if self.consume_byte(b'~') {
            Some(!self.parse_unary()?)
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Option<i64> {
        self.skip_whitespace();
        if self.consume_byte(b'(') {
            let value = self.parse_or()?;
            self.skip_whitespace();
            return self.consume_byte(b')').then_some(value);
        }

        let start = self.position;
        let bytes = self.input.as_bytes();
        if start < bytes.len() && (bytes[start].is_ascii_digit() || bytes[start] == b'-') {
            self.position += 1;
            while self.position < bytes.len() && bytes[self.position].is_ascii_digit() {
                self.position += 1;
            }
            return self.input[start..self.position].parse::<i64>().ok();
        }

        if start < bytes.len() && (bytes[start].is_ascii_alphabetic() || bytes[start] == b'_') {
            self.position += 1;
            while self.position < bytes.len()
                && (bytes[self.position].is_ascii_alphanumeric() || bytes[self.position] == b'_')
            {
                self.position += 1;
            }
            return error_reporting_constant(&self.input[start..self.position]);
        }

        None
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len()
            && self.input.as_bytes()[self.position].is_ascii_whitespace()
        {
            self.position += 1;
        }
    }

    fn consume_byte(&mut self, byte: u8) -> bool {
        if self.position < self.input.len() && self.input.as_bytes()[self.position] == byte {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

fn error_reporting_constant(name: &str) -> Option<i64> {
    match name {
        "E_ERROR" => Some(1),
        "E_WARNING" => Some(2),
        "E_PARSE" => Some(4),
        "E_NOTICE" => Some(8),
        "E_CORE_ERROR" => Some(16),
        "E_CORE_WARNING" => Some(32),
        "E_COMPILE_ERROR" => Some(64),
        "E_COMPILE_WARNING" => Some(128),
        "E_USER_ERROR" => Some(256),
        "E_USER_WARNING" => Some(512),
        "E_USER_NOTICE" => Some(1024),
        "E_STRICT" => Some(2048),
        "E_RECOVERABLE_ERROR" => Some(4096),
        "E_DEPRECATED" => Some(8192),
        "E_USER_DEPRECATED" => Some(16384),
        "E_ALL" => Some(32767),
        _ => None,
    }
}

fn parse_ini_quantity_bytes(raw_value: &str) -> Option<i64> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let mut index = 0usize;
    let sign = if bytes[index] == b'-' {
        index += 1;
        -1i128
    } else if bytes[index] == b'+' {
        index += 1;
        1i128
    } else {
        1i128
    };
    if index >= bytes.len() {
        return None;
    }

    let mut radix = 10u32;
    if index + 1 < bytes.len()
        && bytes[index] == b'0'
        && (bytes[index + 1] == b'x' || bytes[index + 1] == b'X')
    {
        radix = 16;
        index += 2;
    }

    let digit_start = index;
    let mut value = 0i128;
    while index < bytes.len() {
        let digit = match bytes[index] {
            b'0'..=b'9' => (bytes[index] - b'0') as u32,
            b'a'..=b'f' if radix == 16 => 10 + (bytes[index] - b'a') as u32,
            b'A'..=b'F' if radix == 16 => 10 + (bytes[index] - b'A') as u32,
            _ => break,
        };
        if digit >= radix {
            break;
        }
        value = value
            .saturating_mul(radix as i128)
            .saturating_add(digit as i128);
        index += 1;
    }
    if index == digit_start {
        return None;
    }

    let suffix = trimmed
        .bytes()
        .rev()
        .find(|byte| !byte.is_ascii_whitespace())
        .map(|byte| byte.to_ascii_lowercase());
    let multiplier = match suffix {
        Some(b'g') => 1024i128 * 1024 * 1024,
        Some(b'm') => 1024i128 * 1024,
        Some(b'k') => 1024i128,
        _ => 1i128,
    };
    let quantity = sign.saturating_mul(value).saturating_mul(multiplier);
    if quantity > i64::MAX as i128 {
        Some(i64::MAX)
    } else if quantity < i64::MIN as i128 {
        Some(i64::MIN)
    } else {
        Some(quantity as i64)
    }
}

fn apply_memory_limit_bounds(ini: &mut RuntimeIni) -> Option<String> {
    let max_value = ini
        .max_memory_limit
        .as_deref()
        .and_then(parse_ini_quantity_bytes)
        .unwrap_or(-1);
    if max_value < 0 {
        return None;
    }
    let Some(memory_limit) = ini.memory_limit.as_deref() else {
        return None;
    };
    let memory_value = parse_ini_quantity_bytes(memory_limit).unwrap_or(0);
    if memory_value > max_value {
        let max = ini
            .max_memory_limit
            .clone()
            .unwrap_or_else(|| max_value.to_string());
        ini.memory_limit = Some(max);
        Some(format!(
            "Warning: Failed to set memory_limit to {memory_value} bytes. Setting to max_memory_limit instead (currently: {max_value} bytes) in Unknown on line 0\n"
        ))
    } else if memory_value < 0 {
        ini.memory_limit = ini.max_memory_limit.clone();
        None
    } else {
        None
    }
}

fn compile_and_run(script: &Path, args: &[String], ini: &RuntimeIni) -> Result<i32, PhpcError> {
    let mut ini = RuntimeIni {
        precision: ini.precision,
        date_timezone: ini.date_timezone.clone(),
        assert_exception: ini.assert_exception.clone(),
        display_errors: ini.display_errors.clone(),
        error_reporting: ini.error_reporting,
        zend_assertions: ini.zend_assertions.clone(),
        memory_limit: ini.memory_limit.clone(),
        max_memory_limit: ini.max_memory_limit.clone(),
    };
    let memory_limit_warning = apply_memory_limit_bounds(&mut ini);
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
    if let Some(date_timezone) = &ini.date_timezone {
        command.env("PTN_DATE_TIMEZONE", date_timezone);
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
    if let Some(zend_assertions) = &ini.zend_assertions {
        command.env("PTN_ZEND_ASSERTIONS", zend_assertions);
    }
    if let Some(memory_limit) = &ini.memory_limit {
        command.env("PTN_MEMORY_LIMIT", memory_limit);
    }
    if let Some(max_memory_limit) = &ini.max_memory_limit {
        command.env("PTN_MAX_MEMORY_LIMIT", max_memory_limit);
    }
    if let Some(warning) = memory_limit_warning {
        print!("{warning}");
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
