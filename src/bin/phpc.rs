use std::fmt;
use std::fs;
use std::io::{self, IsTerminal, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::diagnostic::{DiagnosticNotice, DiagnosticNoticeKind};
use ptn::{
    compile_file_with_preloads_and_source_options, CompileOptions, CompileSourceOptions,
    Diagnostic, DiagnosticKind,
};

fn main() {
    match run_with_compiler_stack() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(255);
        }
    }
}

fn run_with_compiler_stack() -> Result<i32, PhpcError> {
    std::thread::Builder::new()
        .name("ptn-phpc".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .map_err(|error| format!("failed to start compiler thread: {error}"))?
        .join()
        .map_err(|_| "compiler thread panicked".to_string())?
}

fn run() -> Result<i32, PhpcError> {
    let invocation = Invocation::parse(std::env::args().skip(1))?;
    let sapi = invocation.sapi;
    let ini = invocation.ini;
    match invocation.mode {
        Mode::Version => {
            println!(
                "PHP 8.4.0 (cli) (built: ptn) (NTS)\nCopyright \u{00a9} The PHP Group and Contributors\nZend Engine v4.4.0, Copyright \u{00a9} Zend by Perforce"
            );
            Ok(0)
        }
        Mode::Modules => {
            print_modules();
            Ok(0)
        }
        Mode::Script { script, args } => compile_and_run(&script, &args, &ini, sapi),
        Mode::Inline { source, args } => {
            let temp = TempPath::new("ptn-phpc-inline", "php");
            let source = if source.trim_start().starts_with("<?") {
                source
            } else {
                format!("<?php {source}")
            };
            fs::write(temp.path(), source)
                .map_err(|error| format!("failed to write inline source: {error}"))?;
            compile_and_run(temp.path(), &args, &ini, sapi)
        }
        Mode::Stdin => {
            if io::stdin().is_terminal() {
                return Err(usage().into());
            }
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|error| format!("failed to read stdin source: {error}"))?;
            let temp = TempPath::new("ptn-phpc-stdin", "php");
            fs::write(temp.path(), source)
                .map_err(|error| format!("failed to write stdin source: {error}"))?;
            compile_and_run(temp.path(), &[], &ini, sapi)
        }
    }
}

fn print_modules() {
    println!("[PHP Modules]");
    for module in [
        "Core",
        "ctype",
        "curl",
        "date",
        "dom",
        "filter",
        "hash",
        "iconv",
        "intl",
        "json",
        "libxml",
        "mbstring",
        "openssl",
        "pcre",
        "Phar",
        "Reflection",
        "session",
        "simplexml",
        "sockets",
        "soap",
        "SPL",
        "standard",
        "tokenizer",
        "xml",
        "xmlreader",
        "xmlwriter",
        "zip",
        "zend_test",
        "zlib",
    ] {
        println!("{module}");
    }
    println!();
    println!("[Zend Modules]");
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
            PhpcError::SourceFatal { diagnostic, script } => {
                if write_source_fatal_notices(f, diagnostic, script)? {
                    return Ok(());
                }
                match diagnostic.span {
                    Some(span) => {
                        let source = if span.line == 0 {
                            "Unknown".to_string()
                        } else {
                            script.display().to_string()
                        };
                        if let Some(uncaught) = &diagnostic.uncaught {
                            if let Some(call_frame) = &uncaught.call_frame {
                                return write!(
                                    f,
                                    "Fatal error: Uncaught {}: {} in {}:{}\nStack trace:\n#0 {}({}): {}\n#1 {{main}}\n  thrown in {} on line {}",
                                    uncaught.throwable,
                                    diagnostic.message,
                                    source,
                                    span.line,
                                    source,
                                    span.line,
                                    call_frame,
                                    source,
                                    span.line
                                );
                            }
                            return write!(
                                f,
                                "Fatal error: Uncaught {}: {} in {}:{}\nStack trace:\n#0 {{main}}\n  thrown in {} on line {}",
                                uncaught.throwable,
                                diagnostic.message,
                                source,
                                span.line,
                                source,
                                span.line
                            );
                        }
                        if source_fatal_is_uncaught_error(diagnostic) {
                            return write!(
                                f,
                                "Fatal error: Uncaught Error: {} in {}:{}\nStack trace:\n#0 {{main}}\n  thrown in {} on line {}",
                                diagnostic.message, source, span.line, source, span.line
                            );
                        }
                        write!(
                            f,
                            "{}: {} in {} on line {}",
                            match diagnostic.kind {
                                DiagnosticKind::Fatal => "Fatal error",
                                DiagnosticKind::ParseError => "Parse error",
                            },
                            diagnostic.message,
                            source,
                            span.line
                        )
                    }
                    None => write!(f, "phpc: {diagnostic}"),
                }
            }
        }
    }
}

fn write_source_fatal_notices(
    f: &mut fmt::Formatter<'_>,
    diagnostic: &Diagnostic,
    script: &Path,
) -> Result<bool, fmt::Error> {
    for notice in &diagnostic.notices {
        if write_source_fatal_notice(f, notice, script)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn write_source_fatal_notice(
    f: &mut fmt::Formatter<'_>,
    notice: &DiagnosticNotice,
    script: &Path,
) -> Result<bool, fmt::Error> {
    let source = if notice.span.line == 0 {
        "Unknown".to_string()
    } else {
        script.display().to_string()
    };
    match notice.kind {
        DiagnosticNoticeKind::Warning => {
            writeln!(
                f,
                "Warning: {} in {} on line {}",
                notice.message, source, notice.span.line
            )?;
            writeln!(f)?;
            Ok(false)
        }
        DiagnosticNoticeKind::Deprecation => {
            writeln!(
                f,
                "Deprecated: {} in {} on line {}",
                notice.message, source, notice.span.line
            )?;
            writeln!(f)?;
            Ok(false)
        }
        DiagnosticNoticeKind::UncaughtError => {
            write!(
                f,
                "Fatal error: Uncaught Error: {} in {}:{}\nStack trace:\n#0 {{main}}\n  thrown in {} on line {}",
                notice.message, source, notice.span.line, source, notice.span.line
            )?;
            Ok(true)
        }
    }
}

fn source_fatal_is_uncaught_error(diagnostic: &Diagnostic) -> bool {
    if diagnostic.kind != DiagnosticKind::Fatal {
        return false;
    }
    let message = diagnostic.message.as_str();
    (message.starts_with("Class \"") || message.starts_with("Interface \""))
        && message.ends_with("\" not found")
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
    sapi: Sapi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sapi {
    Cli,
    Cgi,
}

#[derive(Debug)]
enum Mode {
    Version,
    Modules,
    Script { script: PathBuf, args: Vec<String> },
    Inline { source: String, args: Vec<String> },
    Stdin,
}

#[derive(Debug, Default)]
struct RuntimeIni {
    loaded_file_path: Option<PathBuf>,
    precision: Option<i16>,
    serialize_precision: Option<String>,
    default_charset: Option<String>,
    arg_separator_input: Option<String>,
    arg_separator_output: Option<String>,
    highlight_comment: Option<String>,
    highlight_default: Option<String>,
    highlight_html: Option<String>,
    highlight_keyword: Option<String>,
    highlight_string: Option<String>,
    date_timezone: Option<String>,
    assert_active: Option<String>,
    assert_bail: Option<String>,
    assert_callback: Option<String>,
    assert_exception: Option<String>,
    assert_warning: Option<String>,
    auto_detect_line_endings: Option<String>,
    disable_functions: Option<String>,
    display_errors: Option<String>,
    html_errors: Option<String>,
    error_reporting: Option<i64>,
    ignore_repeated_errors: Option<String>,
    ignore_repeated_source: Option<String>,
    output_handler: Option<String>,
    filter_default: Option<String>,
    pcre_backtrack_limit: Option<String>,
    pcre_recursion_limit: Option<String>,
    pcre_jit: Option<String>,
    open_basedir: Option<String>,
    session: Vec<(String, String)>,
    opcache: Vec<(String, String)>,
    opcache_save_comments: Option<String>,
    phar_readonly: Option<String>,
    phar_require_hash: Option<String>,
    phar_cache_list: Option<String>,
    bcmath_scale: Option<String>,
    sendmail_path: Option<String>,
    mail_add_x_header: Option<String>,
    internal_encoding: Option<String>,
    input_encoding: Option<String>,
    output_encoding: Option<String>,
    iconv_internal_encoding: Option<String>,
    iconv_input_encoding: Option<String>,
    iconv_output_encoding: Option<String>,
    mbstring_internal_encoding: Option<String>,
    mbstring_http_input: Option<String>,
    mbstring_http_output: Option<String>,
    mbstring_language: Option<String>,
    mbstring_detect_order: Option<String>,
    mbstring_substitute_character: Option<String>,
    mbstring_encoding_translation: Option<String>,
    intl_error_level: Option<String>,
    intl_use_exceptions: Option<String>,
    intl_default_locale: Option<String>,
    zend_multibyte: Option<String>,
    zend_script_encoding: Option<String>,
    zend_assertions: Option<String>,
    zend_enable_gc: Option<String>,
    memory_limit: Option<String>,
    max_memory_limit: Option<String>,
    fiber_stack_size: Option<String>,
    variables_order: Option<String>,
    register_argc_argv: Option<String>,
    enable_post_data_reading: Option<String>,
    file_uploads: Option<String>,
    max_input_vars: Option<String>,
    max_input_nesting_level: Option<String>,
    post_max_size: Option<String>,
    always_populate_raw_post_data: Option<String>,
    upload_tmp_dir: Option<String>,
    expose_php: Option<String>,
    user_agent: Option<String>,
    exception_ignore_args: Option<String>,
    exception_string_param_max_len: Option<String>,
    allow_url_fopen: Option<String>,
    allow_url_include: Option<String>,
    allow_url_include_deprecated: bool,
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
        let mut sapi = Sapi::Cli;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-q" => {}
                "-n" => {
                    ini.loaded_file_path = None;
                }
                "-C" => {
                    sapi = Sapi::Cgi;
                }
                "-v" | "--version" => {
                    return Ok(Self {
                        mode: Mode::Version,
                        ini,
                        sapi,
                    });
                }
                "-m" => {
                    return Ok(Self {
                        mode: Mode::Modules,
                        ini,
                        sapi,
                    });
                }
                "-d" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("missing value for {arg}"))?;
                    apply_ini_setting(&value, &mut ini);
                }
                "-c" => {
                    let path = args
                        .next()
                        .ok_or_else(|| format!("missing value for {arg}"))?;
                    ini.loaded_file_path = Some(PathBuf::from(path));
                }
                "-f" => {
                    let path = args
                        .next()
                        .ok_or_else(|| "missing value for -f".to_string())?;
                    script = Some(PathBuf::from(path));
                    if matches!(args.peek().map(String::as_str), Some("--")) {
                        args.next();
                    }
                    script_args.extend(args);
                    break;
                }
                "-r" => {
                    let source = args
                        .next()
                        .ok_or_else(|| "missing inline source for -r".to_string())?;
                    if matches!(args.peek().map(String::as_str), Some("--")) {
                        args.next();
                    }
                    return Ok(Self {
                        mode: Mode::Inline {
                            source,
                            args: args.collect(),
                        },
                        ini,
                        sapi,
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
                _ if let Some(path) = arg.strip_prefix("-c") => {
                    if !path.is_empty() {
                        ini.loaded_file_path = Some(PathBuf::from(path));
                    }
                }
                _ if arg.starts_with('-') => {}
                _ => {
                    script = Some(PathBuf::from(arg));
                    script_args.extend(args);
                    break;
                }
            }
        }

        let mode = match script {
            Some(script) => Mode::Script {
                script,
                args: script_args,
            },
            None => Mode::Stdin,
        };
        Ok(Self { mode, ini, sapi })
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
            if (-1..=1000).contains(&parsed) {
                ini.precision = Some(parsed);
            }
        }
    } else if name.eq_ignore_ascii_case("serialize_precision") {
        if let Ok(parsed) = raw_value.parse::<i16>() {
            if (-1..=1000).contains(&parsed) {
                ini.serialize_precision = Some(parsed.to_string());
            }
        }
    } else if name.eq_ignore_ascii_case("assert.exception") {
        ini.assert_exception = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("assert.active") {
        ini.assert_active = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("assert.bail") {
        ini.assert_bail = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("assert.callback") {
        ini.assert_callback = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("assert.warning") {
        ini.assert_warning = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("auto_detect_line_endings") {
        ini.auto_detect_line_endings = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("disable_functions") {
        ini.disable_functions = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("date.timezone") {
        ini.date_timezone = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("default_charset") {
        ini.default_charset = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("display_errors") {
        ini.display_errors = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("html_errors") {
        ini.html_errors = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("highlight.comment") {
        ini.highlight_comment = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("highlight.default") {
        ini.highlight_default = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("highlight.html") {
        ini.highlight_html = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("highlight.keyword") {
        ini.highlight_keyword = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("highlight.string") {
        ini.highlight_string = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("error_reporting") {
        if let Some(parsed) = parse_error_reporting_level(raw_value) {
            ini.error_reporting = Some(parsed);
        }
    } else if name.eq_ignore_ascii_case("ignore_repeated_errors") {
        ini.ignore_repeated_errors = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("ignore_repeated_source") {
        ini.ignore_repeated_source = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("arg_separator.input") {
        ini.arg_separator_input = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("arg_separator.output") {
        ini.arg_separator_output = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("filter.default") {
        ini.filter_default = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("pcre.backtrack_limit") {
        ini.pcre_backtrack_limit = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("pcre.recursion_limit") {
        ini.pcre_recursion_limit = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("pcre.jit") {
        ini.pcre_jit = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("open_basedir") {
        ini.open_basedir = Some(normalize_ini_scalar(raw_value));
    } else if let Some(canonical_name) = canonical_session_ini_name(name) {
        ini.session
            .push((canonical_name.to_string(), normalize_ini_scalar(raw_value)));
    } else if let Some(canonical_name) = canonical_opcache_ini_name(name) {
        let value = normalize_ini_scalar(raw_value);
        ini.opcache
            .push((canonical_name.to_string(), value.clone()));
        if canonical_name.eq_ignore_ascii_case("opcache.save_comments") {
            ini.opcache_save_comments = Some(value);
        }
    } else if name.eq_ignore_ascii_case("phar.readonly") {
        ini.phar_readonly = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("phar.require_hash") {
        ini.phar_require_hash = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("phar.cache_list") {
        ini.phar_cache_list = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("bcmath.scale") {
        if let Ok(parsed) = raw_value.parse::<i64>() {
            if (0..=2_147_483_647).contains(&parsed) {
                ini.bcmath_scale = Some(parsed.to_string());
            }
        }
    } else if name.eq_ignore_ascii_case("sendmail_path") {
        ini.sendmail_path = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("mail.add_x_header") {
        ini.mail_add_x_header = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("output_handler") {
        ini.output_handler = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("internal_encoding") {
        ini.internal_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("input_encoding") {
        ini.input_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("output_encoding") {
        ini.output_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("iconv.internal_encoding")
        || name.eq_ignore_ascii_case("iconv.internal_charset")
    {
        ini.iconv_internal_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("iconv.input_encoding") {
        ini.iconv_input_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("iconv.output_encoding") {
        ini.iconv_output_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.internal_encoding") {
        ini.mbstring_internal_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.http_input") {
        ini.mbstring_http_input = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.http_output") {
        ini.mbstring_http_output = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.language") {
        ini.mbstring_language = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.detect_order") {
        ini.mbstring_detect_order = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.substitute_character") {
        ini.mbstring_substitute_character = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("mbstring.encoding_translation") {
        ini.mbstring_encoding_translation = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("intl.error_level") {
        ini.intl_error_level = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("intl.use_exceptions") {
        ini.intl_use_exceptions = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("intl.default_locale") {
        ini.intl_default_locale = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("zend.multibyte") {
        ini.zend_multibyte = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("zend.script_encoding") {
        ini.zend_script_encoding = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("zend.assertions") {
        ini.zend_assertions = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("zend.enable_gc") {
        ini.zend_enable_gc = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("zend.exception_ignore_args") {
        ini.exception_ignore_args = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("zend.exception_string_param_max_len") {
        if let Ok(parsed) = raw_value.parse::<i64>() {
            if (0..=1_000_000).contains(&parsed) {
                ini.exception_string_param_max_len = Some(parsed.to_string());
            }
        }
    } else if name.eq_ignore_ascii_case("memory_limit") {
        ini.memory_limit = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("max_memory_limit") {
        ini.max_memory_limit = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("fiber.stack_size") {
        ini.fiber_stack_size = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("variables_order") {
        ini.variables_order = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("register_argc_argv") {
        ini.register_argc_argv = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("enable_post_data_reading") {
        ini.enable_post_data_reading = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("file_uploads") {
        ini.file_uploads = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("max_input_vars") {
        ini.max_input_vars = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("max_input_nesting_level") {
        ini.max_input_nesting_level = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("post_max_size") {
        ini.post_max_size = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("always_populate_raw_post_data") {
        ini.always_populate_raw_post_data = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("upload_tmp_dir") {
        ini.upload_tmp_dir = Some(raw_value.to_string());
    } else if name.eq_ignore_ascii_case("expose_php") {
        ini.expose_php = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("user_agent") {
        ini.user_agent = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("allow_url_fopen") {
        ini.allow_url_fopen = Some(normalize_ini_scalar(raw_value));
    } else if name.eq_ignore_ascii_case("allow_url_include") {
        ini.allow_url_include = Some(normalize_ini_scalar(raw_value));
        ini.allow_url_include_deprecated =
            ini.allow_url_include_deprecated || ini_scalar_truthy(raw_value);
    }
}

fn ini_scalar_truthy(raw_value: &str) -> bool {
    let normalized = raw_value.trim();
    if normalized.is_empty() {
        return false;
    }
    !matches!(
        normalized.to_ascii_lowercase().as_str(),
        "0" | "off" | "false" | "no"
    )
}

fn canonical_session_ini_name(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "session.auto_start" => Some("session.auto_start"),
        "session.cache_expire" => Some("session.cache_expire"),
        "session.cache_limiter" => Some("session.cache_limiter"),
        "session.cookie_domain" => Some("session.cookie_domain"),
        "session.cookie_httponly" => Some("session.cookie_httponly"),
        "session.cookie_lifetime" => Some("session.cookie_lifetime"),
        "session.cookie_partitioned" => Some("session.cookie_partitioned"),
        "session.cookie_path" => Some("session.cookie_path"),
        "session.cookie_samesite" => Some("session.cookie_samesite"),
        "session.cookie_secure" => Some("session.cookie_secure"),
        "session.gc_divisor" => Some("session.gc_divisor"),
        "session.gc_maxlifetime" => Some("session.gc_maxlifetime"),
        "session.gc_probability" => Some("session.gc_probability"),
        "session.lazy_write" => Some("session.lazy_write"),
        "session.name" => Some("session.name"),
        "session.referer_check" => Some("session.referer_check"),
        "session.save_handler" => Some("session.save_handler"),
        "session.save_path" => Some("session.save_path"),
        "session.serialize_handler" => Some("session.serialize_handler"),
        "session.sid_bits_per_character" => Some("session.sid_bits_per_character"),
        "session.sid_length" => Some("session.sid_length"),
        "session.trans_sid_hosts" => Some("session.trans_sid_hosts"),
        "session.upload_progress.cleanup" => Some("session.upload_progress.cleanup"),
        "session.upload_progress.enabled" => Some("session.upload_progress.enabled"),
        "session.upload_progress.freq" => Some("session.upload_progress.freq"),
        "session.upload_progress.min_freq" => Some("session.upload_progress.min_freq"),
        "session.upload_progress.name" => Some("session.upload_progress.name"),
        "session.upload_progress.prefix" => Some("session.upload_progress.prefix"),
        "session.use_cookies" => Some("session.use_cookies"),
        "session.use_only_cookies" => Some("session.use_only_cookies"),
        "session.use_strict_mode" => Some("session.use_strict_mode"),
        "session.use_trans_sid" => Some("session.use_trans_sid"),
        _ => None,
    }
}

fn session_ini_env_name(name: &str) -> Option<String> {
    canonical_session_ini_name(name).map(|canonical| {
        let suffix = canonical
            .strip_prefix("session.")
            .unwrap_or(canonical)
            .chars()
            .map(|ch| match ch {
                'a'..='z' => ch.to_ascii_uppercase(),
                '0'..='9' => ch,
                _ => '_',
            })
            .collect::<String>();
        format!("PTN_SESSION_{suffix}")
    })
}

fn canonical_opcache_ini_name(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "opcache.blacklist_filename" => Some("opcache.blacklist_filename"),
        "opcache.enable" => Some("opcache.enable"),
        "opcache.enable_cli" => Some("opcache.enable_cli"),
        "opcache.fast_shutdown" => Some("opcache.fast_shutdown"),
        "opcache.file_cache_only" => Some("opcache.file_cache_only"),
        "opcache.file_update_protection" => Some("opcache.file_update_protection"),
        "opcache.interned_strings_buffer" => Some("opcache.interned_strings_buffer"),
        "opcache.log_verbosity_level" => Some("opcache.log_verbosity_level"),
        "opcache.optimization_level" => Some("opcache.optimization_level"),
        "opcache.opt_debug_level" => Some("opcache.opt_debug_level"),
        "opcache.preload" => Some("opcache.preload"),
        "opcache.preload_user" => Some("opcache.preload_user"),
        "opcache.revalidate_path" => Some("opcache.revalidate_path"),
        "opcache.save_comments" => Some("opcache.save_comments"),
        "opcache.validate_timestamps" => Some("opcache.validate_timestamps"),
        _ => None,
    }
}

fn opcache_ini_env_name(name: &str) -> Option<String> {
    canonical_opcache_ini_name(name)
        .map(|canonical| format!("PTN_{}", canonical.to_ascii_uppercase().replace('.', "_")))
}

fn opcache_preload_files(ini: &RuntimeIni, script: &Path) -> Vec<PathBuf> {
    let script_dir = script.parent().unwrap_or_else(|| Path::new(""));
    let mut files = Vec::new();
    for (name, value) in &ini.opcache {
        if !name.eq_ignore_ascii_case("opcache.preload") || value.is_empty() {
            continue;
        }
        let path = PathBuf::from(value);
        if path.is_absolute() {
            files.push(path);
        } else {
            files.push(script_dir.join(path));
        }
    }
    files
}

fn normalize_ini_scalar(raw_value: &str) -> String {
    let trimmed = raw_value.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
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
        "E_ALL" => Some(30719),
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

fn invalid_zend_script_encoding_warning(ini: &RuntimeIni) -> Option<String> {
    let script_encoding = ini.zend_script_encoding.as_deref()?;
    script_encoding
        .eq_ignore_ascii_case("pass")
        .then(|| {
            format!(
                "Warning: PHP Startup: INI setting contains invalid encoding \"{script_encoding}\" in Unknown on line 0"
            )
        })
}

fn session_save_handler_startup_warning(ini: &RuntimeIni) -> Option<String> {
    ini.session
        .iter()
        .rev()
        .find(|(name, _)| name.eq_ignore_ascii_case("session.save_handler"))
        .and_then(|(_, value)| {
            value.eq_ignore_ascii_case("user").then(|| {
                "Fatal error: PHP Startup: Session save handler \"user\" cannot be set by ini_set() in Unknown on line 0".to_string()
            })
        })
}

fn session_startup_deprecations(ini: &RuntimeIni) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    if ini
        .session
        .iter()
        .rev()
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case("session.use_only_cookies")
                .then_some(value)
        })
        .is_some_and(|value| !ini_scalar_truthy(value))
    {
        warnings.push(
            "Deprecated: PHP Startup: Disabling session.use_only_cookies INI setting is deprecated in Unknown on line 0",
        );
    }
    if ini
        .session
        .iter()
        .rev()
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case("session.use_trans_sid")
                .then_some(value)
        })
        .is_some_and(|value| ini_scalar_truthy(value))
    {
        warnings.push(
            "Deprecated: PHP Startup: Enabling session.use_trans_sid INI setting is deprecated in Unknown on line 0",
        );
    }
    if ini
        .session
        .iter()
        .rev()
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case("session.trans_sid_hosts")
                .then_some(value)
        })
        .is_some_and(|value| !value.is_empty())
    {
        warnings.push(
            "Deprecated: PHP Startup: Usage of session.trans_sid_hosts INI setting is deprecated in Unknown on line 0",
        );
    }
    warnings
}

fn normalize_session_upload_progress_freq(ini: &mut RuntimeIni) -> Option<&'static str> {
    let value = ini.session.iter().rev().find_map(|(name, value)| {
        name.eq_ignore_ascii_case("session.upload_progress.freq")
            .then_some(value.as_str())
    })?;
    let percent = value.strip_suffix('%')?;
    let parsed = percent.trim().parse::<u64>().ok()?;
    if parsed <= 100 {
        return None;
    }
    ini.session
        .push(("session.upload_progress.freq".to_string(), "1%".to_string()));
    Some(
        "Warning: PHP Startup: session.upload_progress.freq must be less than or equal to 100% in Unknown on line 0",
    )
}

fn assert_ini_bool_differs_from_default(value: Option<&str>, default: bool) -> bool {
    value.is_some_and(|value| ini_scalar_truthy(value) != default)
}

fn assert_ini_string_differs_from_default(value: Option<&str>) -> bool {
    value.is_some_and(|value| !normalize_ini_scalar(value).is_empty())
}

fn assert_startup_deprecations(ini: &RuntimeIni) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    if assert_ini_bool_differs_from_default(ini.assert_active.as_deref(), true) {
        warnings.push(
            "Deprecated: PHP Startup: assert.active INI setting is deprecated in Unknown on line 0",
        );
    }
    if assert_ini_bool_differs_from_default(ini.assert_warning.as_deref(), true) {
        warnings.push("Deprecated: PHP Startup: assert.warning INI setting is deprecated in Unknown on line 0");
    }
    if assert_ini_string_differs_from_default(ini.assert_callback.as_deref()) {
        warnings.push("Deprecated: PHP Startup: assert.callback INI setting is deprecated in Unknown on line 0");
    }
    if assert_ini_bool_differs_from_default(ini.assert_bail.as_deref(), false) {
        warnings.push(
            "Deprecated: PHP Startup: assert.bail INI setting is deprecated in Unknown on line 0",
        );
    }
    if assert_ini_bool_differs_from_default(ini.assert_exception.as_deref(), true) {
        warnings.push("Deprecated: PHP Startup: assert.exception INI setting is deprecated in Unknown on line 0");
    }
    warnings
}

fn mbstring_ini_encoding_items(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .map(str::trim)
        .filter(|item| !item.is_empty())
}

fn mbstring_ini_encoding_is_valid(encoding: &str) -> bool {
    let encoding = encoding.trim();
    if encoding.is_empty() {
        return false;
    }
    let lower = encoding.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "pass"
            | "auto"
            | "wchar"
            | "byte2be"
            | "byte2le"
            | "byte4be"
            | "byte4le"
            | "base64"
            | "uuencode"
            | "html-entities"
            | "quoted-printable"
            | "7bit"
            | "8bit"
            | "binary"
            | "ascii"
            | "us-ascii"
            | "utf-8"
            | "utf8"
            | "utf-7"
            | "utf7-imap"
            | "utf-16"
            | "utf-16be"
            | "utf-16le"
            | "utf-32"
            | "utf-32be"
            | "utf-32le"
            | "ucs-2"
            | "ucs-2be"
            | "ucs-2le"
            | "ucs-4"
            | "ucs-4be"
            | "ucs-4le"
            | "sjis"
            | "shift_jis"
            | "euc-jp"
            | "jis"
            | "iso-2022-jp"
            | "iso-8859-1"
            | "iso-8859-15"
            | "windows-1251"
            | "windows-1252"
            | "windows-1254"
            | "cp1251"
            | "cp1252"
            | "cp1254"
            | "koi8-r"
            | "koi8-u"
            | "big-5"
            | "big5"
            | "euc-kr"
            | "gb18030"
            | "gb2312"
            | "hz"
            | "hz-gb-2312"
            | "cp936"
            | "cp866"
            | "cp850"
            | "armscii-8"
            | "euc-tw"
            | "iso-8859-5"
            | "iso-8859-7"
            | "iso-8859-9"
            | "iso-8859-10"
            | "iso-8859-13"
            | "iso-8859-14"
            | "iso-8859-16"
    )
}

fn mbstring_ini_first_invalid_encoding(value: &str) -> Option<&str> {
    mbstring_ini_encoding_items(value).find(|item| !mbstring_ini_encoding_is_valid(item))
}

fn mbstring_startup_messages(ini: &RuntimeIni) -> Vec<String> {
    let mut messages = Vec::new();
    if let Some(detect_order) = ini.mbstring_detect_order.as_deref() {
        if let Some(invalid) = mbstring_ini_first_invalid_encoding(detect_order) {
            messages.push(format!(
                "Warning: PHP Startup: INI setting contains invalid encoding \"{invalid}\" in Unknown on line 0"
            ));
        }
    }
    if let Some(http_input) = ini.mbstring_http_input.as_deref() {
        messages.push(
            "Deprecated: PHP Startup: Use of mbstring.http_input is deprecated in Unknown on line 0"
                .to_string(),
        );
        if let Some(invalid) = mbstring_ini_first_invalid_encoding(http_input) {
            messages.push(format!(
                "Warning: PHP Startup: INI setting contains invalid encoding \"{invalid}\" in Unknown on line 0"
            ));
        }
    }
    if ini.mbstring_http_output.is_some() {
        messages.push(
            "Deprecated: PHP Startup: Use of mbstring.http_output is deprecated in Unknown on line 0"
                .to_string(),
        );
    }
    if let Some(internal_encoding) = ini.mbstring_internal_encoding.as_deref() {
        messages.push(
            "Deprecated: PHP Startup: Use of mbstring.internal_encoding is deprecated in Unknown on line 0"
                .to_string(),
        );
        if !internal_encoding.is_empty() && !mbstring_ini_encoding_is_valid(internal_encoding) {
            messages.push(format!(
                "Warning: PHP Startup: Unknown encoding \"{internal_encoding}\" in ini setting in Unknown on line 0"
            ));
        }
    }
    messages
}

fn compile_and_run(
    script: &Path,
    args: &[String],
    ini: &RuntimeIni,
    sapi: Sapi,
) -> Result<i32, PhpcError> {
    let mut ini = RuntimeIni {
        loaded_file_path: ini.loaded_file_path.clone(),
        precision: ini.precision,
        serialize_precision: ini.serialize_precision.clone(),
        default_charset: ini.default_charset.clone(),
        arg_separator_input: ini.arg_separator_input.clone(),
        arg_separator_output: ini.arg_separator_output.clone(),
        highlight_comment: ini.highlight_comment.clone(),
        highlight_default: ini.highlight_default.clone(),
        highlight_html: ini.highlight_html.clone(),
        highlight_keyword: ini.highlight_keyword.clone(),
        highlight_string: ini.highlight_string.clone(),
        date_timezone: ini.date_timezone.clone(),
        assert_active: ini.assert_active.clone(),
        assert_bail: ini.assert_bail.clone(),
        assert_callback: ini.assert_callback.clone(),
        assert_exception: ini.assert_exception.clone(),
        assert_warning: ini.assert_warning.clone(),
        auto_detect_line_endings: ini.auto_detect_line_endings.clone(),
        disable_functions: ini.disable_functions.clone(),
        display_errors: ini.display_errors.clone(),
        html_errors: ini.html_errors.clone(),
        error_reporting: ini.error_reporting,
        ignore_repeated_errors: ini.ignore_repeated_errors.clone(),
        ignore_repeated_source: ini.ignore_repeated_source.clone(),
        output_handler: ini.output_handler.clone(),
        filter_default: ini.filter_default.clone(),
        pcre_backtrack_limit: ini.pcre_backtrack_limit.clone(),
        pcre_recursion_limit: ini.pcre_recursion_limit.clone(),
        pcre_jit: ini.pcre_jit.clone(),
        open_basedir: ini.open_basedir.clone(),
        session: ini.session.clone(),
        opcache: ini.opcache.clone(),
        opcache_save_comments: ini.opcache_save_comments.clone(),
        phar_readonly: ini.phar_readonly.clone(),
        phar_require_hash: ini.phar_require_hash.clone(),
        phar_cache_list: ini.phar_cache_list.clone(),
        bcmath_scale: ini.bcmath_scale.clone(),
        sendmail_path: ini.sendmail_path.clone(),
        mail_add_x_header: ini.mail_add_x_header.clone(),
        internal_encoding: ini.internal_encoding.clone(),
        input_encoding: ini.input_encoding.clone(),
        output_encoding: ini.output_encoding.clone(),
        iconv_internal_encoding: ini.iconv_internal_encoding.clone(),
        iconv_input_encoding: ini.iconv_input_encoding.clone(),
        iconv_output_encoding: ini.iconv_output_encoding.clone(),
        mbstring_internal_encoding: ini.mbstring_internal_encoding.clone(),
        mbstring_http_input: ini.mbstring_http_input.clone(),
        mbstring_http_output: ini.mbstring_http_output.clone(),
        mbstring_language: ini.mbstring_language.clone(),
        mbstring_detect_order: ini.mbstring_detect_order.clone(),
        mbstring_substitute_character: ini.mbstring_substitute_character.clone(),
        mbstring_encoding_translation: ini.mbstring_encoding_translation.clone(),
        intl_error_level: ini.intl_error_level.clone(),
        intl_use_exceptions: ini.intl_use_exceptions.clone(),
        intl_default_locale: ini.intl_default_locale.clone(),
        zend_multibyte: ini.zend_multibyte.clone(),
        zend_script_encoding: ini.zend_script_encoding.clone(),
        zend_assertions: ini.zend_assertions.clone(),
        zend_enable_gc: ini.zend_enable_gc.clone(),
        memory_limit: ini.memory_limit.clone(),
        max_memory_limit: ini.max_memory_limit.clone(),
        fiber_stack_size: ini.fiber_stack_size.clone(),
        variables_order: ini.variables_order.clone(),
        register_argc_argv: ini.register_argc_argv.clone(),
        enable_post_data_reading: ini.enable_post_data_reading.clone(),
        file_uploads: ini.file_uploads.clone(),
        max_input_vars: ini.max_input_vars.clone(),
        max_input_nesting_level: ini.max_input_nesting_level.clone(),
        post_max_size: ini.post_max_size.clone(),
        always_populate_raw_post_data: ini.always_populate_raw_post_data.clone(),
        upload_tmp_dir: ini.upload_tmp_dir.clone(),
        expose_php: ini.expose_php.clone(),
        user_agent: ini.user_agent.clone(),
        exception_ignore_args: ini.exception_ignore_args.clone(),
        exception_string_param_max_len: ini.exception_string_param_max_len.clone(),
        allow_url_fopen: ini.allow_url_fopen.clone(),
        allow_url_include: ini.allow_url_include.clone(),
        allow_url_include_deprecated: ini.allow_url_include_deprecated,
    };
    if ini.default_charset.is_none() {
        if let Some(internal_encoding) = &ini.internal_encoding {
            if !internal_encoding.is_empty() {
                ini.default_charset = Some(internal_encoding.clone());
            }
        }
    }
    let memory_limit_warning = apply_memory_limit_bounds(&mut ini);
    let zend_script_encoding_warning = invalid_zend_script_encoding_warning(&ini);
    let session_upload_progress_freq_warning = normalize_session_upload_progress_freq(&mut ini);
    let session_save_handler_warning = session_save_handler_startup_warning(&ini);
    let session_startup_deprecations = session_startup_deprecations(&ini);
    let assert_startup_deprecations = assert_startup_deprecations(&ini);
    let mbstring_startup_messages = mbstring_startup_messages(&ini);
    let mut source_options = CompileSourceOptions {
        zend_multibyte: ini.zend_multibyte.as_deref().is_some_and(ini_scalar_truthy),
        script_encoding: ini
            .zend_script_encoding
            .as_ref()
            .filter(|encoding| !encoding.eq_ignore_ascii_case("pass"))
            .cloned(),
        internal_encoding: ini
            .internal_encoding
            .as_ref()
            .filter(|encoding| !encoding.is_empty())
            .cloned()
            .or_else(|| {
                ini.mbstring_internal_encoding
                    .as_ref()
                    .filter(|encoding| !encoding.is_empty())
                    .cloned()
            }),
        encoding_translation: ini
            .mbstring_encoding_translation
            .as_deref()
            .is_some_and(ini_scalar_truthy),
        force_internal_function_dispatch: ini
            .output_handler
            .as_deref()
            .is_some_and(|handler| !handler.trim().is_empty()),
    };
    let native = TempPath::new("ptn-phpc-native", "bin");
    let archive_wrapper = phar_archive_main_wrapper(script)?;
    if archive_wrapper.is_some() {
        source_options.force_internal_function_dispatch = true;
    }
    let compile_script = archive_wrapper
        .as_ref()
        .map(|wrapper| wrapper.path())
        .unwrap_or(script);
    let preload_files = opcache_preload_files(&ini, script);
    compile_file_with_preloads_and_source_options(
        compile_script,
        native.path(),
        CompileOptions { emit_c: false },
        &preload_files,
        source_options,
    )
    .map_err(|error| {
        if error.span.is_some() {
            PhpcError::SourceFatal {
                diagnostic: error,
                script: script.to_path_buf(),
            }
        } else {
            PhpcError::Message(error.to_string())
        }
    })?;

    let php_binary = std::env::current_exe()
        .ok()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| {
            std::env::args_os()
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("phpc"))
        });
    let php_binary_wrapper = php_binary_wrapper(&php_binary, native.path(), script)?;
    let mut command = Command::new(native.path());
    command.args(args);
    command.env("PTN_PHP_BINARY", php_binary_wrapper.path());
    command.env("PTN_SCRIPT_FILENAME", script);
    command.env("PTN_RUNTIME_SOURCE_PATH", script);
    if let Some(loaded_file_path) = &ini.loaded_file_path {
        command.env("PTN_PHP_INI_LOADED_FILE", loaded_file_path);
    }
    if let Some(precision) = ini.precision {
        command.env("PTN_PHP_PRECISION", precision.to_string());
    }
    if let Some(serialize_precision) = &ini.serialize_precision {
        command.env("PTN_PHP_SERIALIZE_PRECISION", serialize_precision);
    }
    if let Some(date_timezone) = &ini.date_timezone {
        command.env("PTN_DATE_TIMEZONE", date_timezone);
    }
    if let Some(default_charset) = &ini.default_charset {
        command.env("PTN_DEFAULT_CHARSET", default_charset);
    }
    if let Some(arg_separator_input) = &ini.arg_separator_input {
        command.env("PTN_ARG_SEPARATOR_INPUT", arg_separator_input);
    }
    if let Some(arg_separator_output) = &ini.arg_separator_output {
        command.env("PTN_ARG_SEPARATOR_OUTPUT", arg_separator_output);
    }
    if let Some(highlight_comment) = &ini.highlight_comment {
        command.env("PTN_HIGHLIGHT_COMMENT", highlight_comment);
    }
    if let Some(highlight_default) = &ini.highlight_default {
        command.env("PTN_HIGHLIGHT_DEFAULT", highlight_default);
    }
    if let Some(highlight_html) = &ini.highlight_html {
        command.env("PTN_HIGHLIGHT_HTML", highlight_html);
    }
    if let Some(highlight_keyword) = &ini.highlight_keyword {
        command.env("PTN_HIGHLIGHT_KEYWORD", highlight_keyword);
    }
    if let Some(highlight_string) = &ini.highlight_string {
        command.env("PTN_HIGHLIGHT_STRING", highlight_string);
    }
    if let Some(assert_active) = &ini.assert_active {
        command.env("PTN_ASSERT_ACTIVE", assert_active);
    }
    if let Some(assert_bail) = &ini.assert_bail {
        command.env("PTN_ASSERT_BAIL", assert_bail);
    }
    if let Some(assert_callback) = &ini.assert_callback {
        command.env("PTN_ASSERT_CALLBACK", assert_callback);
    }
    if let Some(assert_exception) = &ini.assert_exception {
        command.env("PTN_ASSERT_EXCEPTION", assert_exception);
    }
    if let Some(assert_warning) = &ini.assert_warning {
        command.env("PTN_ASSERT_WARNING", assert_warning);
    }
    if let Some(auto_detect_line_endings) = &ini.auto_detect_line_endings {
        command.env("PTN_AUTO_DETECT_LINE_ENDINGS", auto_detect_line_endings);
    }
    if let Some(disable_functions) = &ini.disable_functions {
        command.env("PTN_DISABLE_FUNCTIONS", disable_functions);
    }
    if let Some(display_errors) = &ini.display_errors {
        command.env("PTN_PHP_DISPLAY_ERRORS", display_errors);
    }
    if let Some(html_errors) = &ini.html_errors {
        command.env("PTN_PHP_HTML_ERRORS", html_errors);
    }
    if let Some(error_reporting) = ini.error_reporting {
        command.env("PTN_PHP_ERROR_REPORTING", error_reporting.to_string());
    }
    if let Some(ignore_repeated_errors) = &ini.ignore_repeated_errors {
        command.env("PTN_IGNORE_REPEATED_ERRORS", ignore_repeated_errors);
    }
    if let Some(ignore_repeated_source) = &ini.ignore_repeated_source {
        command.env("PTN_IGNORE_REPEATED_SOURCE", ignore_repeated_source);
    }
    if let Some(output_handler) = &ini.output_handler {
        command.env("PTN_OUTPUT_HANDLER", output_handler);
    }
    if let Some(filter_default) = &ini.filter_default {
        command.env("PTN_FILTER_DEFAULT", filter_default);
    }
    if let Some(pcre_backtrack_limit) = &ini.pcre_backtrack_limit {
        command.env("PTN_PCRE_BACKTRACK_LIMIT", pcre_backtrack_limit);
    }
    if let Some(pcre_recursion_limit) = &ini.pcre_recursion_limit {
        command.env("PTN_PCRE_RECURSION_LIMIT", pcre_recursion_limit);
    }
    if let Some(pcre_jit) = &ini.pcre_jit {
        command.env("PTN_PCRE_JIT", pcre_jit);
    }
    if let Some(open_basedir) = &ini.open_basedir {
        command.env("PTN_OPEN_BASEDIR", open_basedir);
    }
    for (name, value) in &ini.session {
        if let Some(env_name) = session_ini_env_name(name) {
            command.env(env_name, value);
        }
    }
    for (name, value) in &ini.opcache {
        if let Some(env_name) = opcache_ini_env_name(name) {
            command.env(env_name, value);
        }
    }
    if let Some(opcache_save_comments) = &ini.opcache_save_comments {
        command.env("PTN_OPCACHE_SAVE_COMMENTS", opcache_save_comments);
    }
    if let Some(phar_readonly) = &ini.phar_readonly {
        command.env("PTN_PHAR_READONLY", phar_readonly);
    }
    if let Some(phar_require_hash) = &ini.phar_require_hash {
        command.env("PTN_PHAR_REQUIRE_HASH", phar_require_hash);
    }
    if let Some(phar_cache_list) = &ini.phar_cache_list {
        command.env("PTN_PHAR_CACHE_LIST", phar_cache_list);
    }
    if let Some(bcmath_scale) = &ini.bcmath_scale {
        command.env("PTN_BCMATH_SCALE", bcmath_scale);
    }
    if let Some(sendmail_path) = &ini.sendmail_path {
        command.env("PTN_SENDMAIL_PATH", sendmail_path);
    }
    if let Some(mail_add_x_header) = &ini.mail_add_x_header {
        command.env("PTN_MAIL_ADD_X_HEADER", mail_add_x_header);
    }
    if let Some(internal_encoding) = &ini.internal_encoding {
        command.env("PTN_INTERNAL_ENCODING", internal_encoding);
    }
    if let Some(input_encoding) = &ini.input_encoding {
        command.env("PTN_INPUT_ENCODING", input_encoding);
    }
    if let Some(output_encoding) = &ini.output_encoding {
        command.env("PTN_OUTPUT_ENCODING", output_encoding);
    }
    if let Some(iconv_internal_encoding) = &ini.iconv_internal_encoding {
        command.env("PTN_ICONV_INTERNAL_ENCODING", iconv_internal_encoding);
    }
    if let Some(iconv_input_encoding) = &ini.iconv_input_encoding {
        command.env("PTN_ICONV_INPUT_ENCODING", iconv_input_encoding);
    }
    if let Some(iconv_output_encoding) = &ini.iconv_output_encoding {
        command.env("PTN_ICONV_OUTPUT_ENCODING", iconv_output_encoding);
    }
    if let Some(mbstring_internal_encoding) = &ini.mbstring_internal_encoding {
        command.env("PTN_MBSTRING_INTERNAL_ENCODING", mbstring_internal_encoding);
    }
    if let Some(mbstring_http_input) = &ini.mbstring_http_input {
        command.env("PTN_MBSTRING_HTTP_INPUT", mbstring_http_input);
    }
    if let Some(mbstring_http_output) = &ini.mbstring_http_output {
        command.env("PTN_MBSTRING_HTTP_OUTPUT", mbstring_http_output);
    }
    if let Some(mbstring_language) = &ini.mbstring_language {
        command.env("PTN_MBSTRING_LANGUAGE", mbstring_language);
    }
    if let Some(mbstring_detect_order) = &ini.mbstring_detect_order {
        command.env("PTN_MBSTRING_DETECT_ORDER", mbstring_detect_order);
    }
    if let Some(mbstring_substitute_character) = &ini.mbstring_substitute_character {
        command.env(
            "PTN_MBSTRING_SUBSTITUTE_CHARACTER",
            mbstring_substitute_character,
        );
    }
    if let Some(intl_error_level) = &ini.intl_error_level {
        command.env("PTN_INTL_ERROR_LEVEL", intl_error_level);
    }
    if let Some(intl_use_exceptions) = &ini.intl_use_exceptions {
        command.env("PTN_INTL_USE_EXCEPTIONS", intl_use_exceptions);
    }
    if let Some(intl_default_locale) = &ini.intl_default_locale {
        command.env("PTN_INTL_DEFAULT_LOCALE", intl_default_locale);
    }
    if let Some(zend_assertions) = &ini.zend_assertions {
        command.env("PTN_ZEND_ASSERTIONS", zend_assertions);
    }
    if let Some(zend_enable_gc) = &ini.zend_enable_gc {
        command.env("PTN_ZEND_ENABLE_GC", zend_enable_gc);
    }
    if let Some(exception_ignore_args) = &ini.exception_ignore_args {
        command.env("PTN_EXCEPTION_IGNORE_ARGS", exception_ignore_args);
    }
    if let Some(exception_string_param_max_len) = &ini.exception_string_param_max_len {
        command.env(
            "PTN_EXCEPTION_STRING_PARAM_MAX_LEN",
            exception_string_param_max_len,
        );
    }
    if let Some(memory_limit) = &ini.memory_limit {
        command.env("PTN_MEMORY_LIMIT", memory_limit);
    }
    if let Some(max_memory_limit) = &ini.max_memory_limit {
        command.env("PTN_MAX_MEMORY_LIMIT", max_memory_limit);
    }
    if let Some(fiber_stack_size) = &ini.fiber_stack_size {
        command.env("PTN_FIBER_STACK_SIZE", fiber_stack_size);
    }
    if let Some(variables_order) = &ini.variables_order {
        command.env("PTN_VARIABLES_ORDER", variables_order);
    }
    if let Some(register_argc_argv) = &ini.register_argc_argv {
        command.env("PTN_REGISTER_ARGC_ARGV", register_argc_argv);
    }
    if let Some(enable_post_data_reading) = &ini.enable_post_data_reading {
        command.env("PTN_ENABLE_POST_DATA_READING", enable_post_data_reading);
    }
    if let Some(file_uploads) = &ini.file_uploads {
        command.env("PTN_FILE_UPLOADS", file_uploads);
    }
    if let Some(max_input_vars) = &ini.max_input_vars {
        command.env("PTN_MAX_INPUT_VARS", max_input_vars);
    }
    if let Some(max_input_nesting_level) = &ini.max_input_nesting_level {
        command.env("PTN_MAX_INPUT_NESTING_LEVEL", max_input_nesting_level);
    }
    if let Some(post_max_size) = &ini.post_max_size {
        command.env("PTN_POST_MAX_SIZE", post_max_size);
    }
    if let Some(always_populate_raw_post_data) = &ini.always_populate_raw_post_data {
        command.env(
            "PTN_ALWAYS_POPULATE_RAW_POST_DATA",
            always_populate_raw_post_data,
        );
    }
    if let Some(upload_tmp_dir) = &ini.upload_tmp_dir {
        command.env("PTN_UPLOAD_TMP_DIR", upload_tmp_dir);
    }
    if let Some(expose_php) = &ini.expose_php {
        command.env("PTN_EXPOSE_PHP", expose_php);
    }
    if let Some(user_agent) = &ini.user_agent {
        command.env("PTN_USER_AGENT", user_agent);
    }
    if let Some(allow_url_fopen) = &ini.allow_url_fopen {
        command.env("PTN_ALLOW_URL_FOPEN", allow_url_fopen);
    }
    if let Some(allow_url_include) = &ini.allow_url_include {
        command.env("PTN_ALLOW_URL_INCLUDE", allow_url_include);
    }
    if sapi == Sapi::Cgi {
        command.env("PTN_REQUEST_MODE", "cgi");
    } else {
        command.env("PTN_REQUEST_MODE", "cli");
    }
    let startup_warning_emitted = memory_limit_warning.is_some()
        || zend_script_encoding_warning.is_some()
        || session_upload_progress_freq_warning.is_some()
        || session_save_handler_warning.is_some()
        || !session_startup_deprecations.is_empty()
        || !assert_startup_deprecations.is_empty()
        || !mbstring_startup_messages.is_empty()
        || ini.allow_url_include_deprecated;
    if startup_warning_emitted {
        command.env("PTN_STARTUP_WARNING_EMITTED", "1");
    }
    if let Some(warning) = memory_limit_warning {
        print!("{warning}");
    }
    if let Some(warning) = zend_script_encoding_warning {
        println!("{warning}");
    }
    if let Some(warning) = session_upload_progress_freq_warning {
        println!("{warning}");
    }
    if let Some(warning) = session_save_handler_warning {
        println!("{warning}");
    }
    for (index, warning) in session_startup_deprecations.iter().enumerate() {
        println!("{warning}");
        if index + 1 < session_startup_deprecations.len() {
            println!();
        }
    }
    for (index, warning) in assert_startup_deprecations.iter().enumerate() {
        println!("{warning}");
        if index + 1 < assert_startup_deprecations.len() {
            println!();
        }
    }
    if ini.allow_url_include_deprecated {
        println!("Deprecated: Directive 'allow_url_include' is deprecated in Unknown on line 0");
    }
    for (index, warning) in mbstring_startup_messages.iter().enumerate() {
        println!("{warning}");
        if index + 1 < mbstring_startup_messages.len() {
            println!();
        }
    }
    let status = command
        .status()
        .map_err(|error| PhpcError::Message(format!("failed to run native binary: {error}")))?;
    Ok(status.code().unwrap_or(255))
}

fn shell_single_quote(value: &str) -> String {
    let mut quoted = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            quoted.push_str("'\\''");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('\'');
    quoted
}

fn php_binary_wrapper(phpc: &Path, native: &Path, script: &Path) -> Result<TempPath, PhpcError> {
    let wrapper = TempPath::new("ptn-phpc-binary", "sh");
    let canonical_script = fs::canonicalize(script).unwrap_or_else(|_| script.to_path_buf());
    let content = format!(
        r#"#!/usr/bin/env bash
phpc={phpc}
native={native}
script={script}
canonical_script={canonical_script}
args=("$@")

dispatch_current_script() {{
    local candidate="$1"
    local next_index="$2"
    if [[ "$candidate" == "$script" || "$candidate" == "$canonical_script" ]]; then
        export PTN_SCRIPT_FILENAME="$script"
        export PTN_RUNTIME_SOURCE_PATH="$script"
        exec "$native" "${{args[@]:$next_index}}"
    fi
    exec "$phpc" "${{args[@]}}"
}}

i=0
while (( i < ${{#args[@]}} )); do
    arg="${{args[$i]}}"
    case "$arg" in
        -q|-n|-C)
            ((i += 1))
            ;;
        -d|-c)
            ((i += 2))
            ;;
        -d*|-c*)
            ((i += 1))
            ;;
        -r)
            exec "$phpc" "${{args[@]}}"
            ;;
        -f)
            ((i += 1))
            if (( i >= ${{#args[@]}} )); then
                exec "$phpc" "${{args[@]}}"
            fi
            candidate="${{args[$i]}}"
            ((i += 1))
            dispatch_current_script "$candidate" "$i"
            ;;
        --)
            ((i += 1))
            if (( i >= ${{#args[@]}} )); then
                exec "$phpc" "${{args[@]}}"
            fi
            candidate="${{args[$i]}}"
            ((i += 1))
            dispatch_current_script "$candidate" "$i"
            ;;
        -*)
            ((i += 1))
            ;;
        *)
            candidate="$arg"
            ((i += 1))
            dispatch_current_script "$candidate" "$i"
            ;;
    esac
done

exec "$phpc" "${{args[@]}}"
"#,
        phpc = shell_single_quote(&phpc.to_string_lossy()),
        native = shell_single_quote(&native.to_string_lossy()),
        script = shell_single_quote(&script.to_string_lossy()),
        canonical_script = shell_single_quote(&canonical_script.to_string_lossy()),
    );
    fs::write(wrapper.path(), content).map_err(|error| {
        PhpcError::Message(format!("failed to write PHP_BINARY wrapper: {error}"))
    })?;
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(wrapper.path())
            .map_err(|error| {
                PhpcError::Message(format!("failed to stat PHP_BINARY wrapper: {error}"))
            })?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(wrapper.path(), permissions).map_err(|error| {
            PhpcError::Message(format!(
                "failed to make PHP_BINARY wrapper executable: {error}"
            ))
        })?;
    }
    Ok(wrapper)
}

fn phar_archive_main_wrapper(script: &Path) -> Result<Option<TempPath>, PhpcError> {
    if !script_looks_like_binary_phar_archive(script)? {
        return Ok(None);
    }
    let wrapper = TempPath::new("ptn-phpc-phar-main", "php");
    fs::write(
        wrapper.path(),
        "<?php\nextension_loaded('Phar');\ninclude $_SERVER['SCRIPT_FILENAME'];\n",
    )
    .map_err(|error| {
        PhpcError::Message(format!(
            "failed to write PHAR archive wrapper for {}: {error}",
            script.display()
        ))
    })?;
    Ok(Some(wrapper))
}

fn script_looks_like_binary_phar_archive(script: &Path) -> Result<bool, PhpcError> {
    let bytes = fs::read(script).map_err(|error| {
        PhpcError::Message(format!("failed to read {}: {error}", script.display()))
    })?;
    if bytes.starts_with(b"PK\x03\x04") {
        return Ok(true);
    }
    if bytes.len() >= 262 && &bytes[257..262] == b"ustar" {
        return Ok(true);
    }
    Ok(false)
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
