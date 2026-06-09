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
    match invocation.mode {
        Mode::Version => {
            println!("PHP 8.4.0 (ptn phpc)");
            Ok(0)
        }
        Mode::Modules => Ok(0),
        Mode::Script { script, args } => compile_and_run(&script, &args),
        Mode::Inline { source } => {
            let temp = TempPath::new("ptn-phpc-inline", "php");
            let source = if source.trim_start().starts_with("<?") {
                source
            } else {
                format!("<?php {source}")
            };
            fs::write(temp.path(), source)
                .map_err(|error| format!("failed to write inline source: {error}"))?;
            compile_and_run(temp.path(), &[])
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
}

#[derive(Debug)]
enum Mode {
    Version,
    Modules,
    Script { script: PathBuf, args: Vec<String> },
    Inline { source: String },
}

impl Invocation {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter().peekable();
        let mut script = None;
        let mut script_args = Vec::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-q" | "-n" => {}
                "-v" | "--version" => {
                    return Ok(Self {
                        mode: Mode::Version,
                    });
                }
                "-m" => {
                    return Ok(Self {
                        mode: Mode::Modules,
                    });
                }
                "-d" | "-c" => {
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
                _ if arg.starts_with("-d") || arg.starts_with("-c") => {}
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
        })
    }
}

fn compile_and_run(script: &Path, args: &[String]) -> Result<i32, PhpcError> {
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

    let status = Command::new(native.path())
        .args(args)
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
