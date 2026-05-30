use std::env;
use std::sync::{Mutex, MutexGuard, OnceLock};

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("environment lock is not poisoned")
}

fn set_env_var(name: &str, value: &str) {
    env::set_var(name, value);
}

fn restore_env_var(name: &str, previous: Option<std::ffi::OsString>) {
    if let Some(value) = previous {
        env::set_var(name, value);
    } else {
        env::remove_var(name);
    }
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn date_default_timezone_set_accepts_current_utc_slice() {
    let execution = run_source(
        r#"<?php
echo date_default_timezone_set("UTC") ? "utc" : "missing";
echo "|";
echo date_default_timezone_set("Nope/Zone") ? "valid" : "invalid";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "utc|\nNotice: date_default_timezone_set(): Timezone ID 'Nope/Zone' is invalid in Command line code on line 4\ninvalid"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn phpt_ini_date_timezone_initializes_default_timezone_and_startup_warnings() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    set_env_var("PHPC_PHPT_INI_FLAGS", "-d date.timezone=Europe/Rome");
    let rome = run_source(
        r#"<?php
echo date_default_timezone_get(), "|", date("T", strtotime("2020-01-01 00:00:00")), "\n";
"#,
    )
    .unwrap();
    assert_eq!(rome.stdout, "Europe/Rome|CET\n");
    assert_eq!(rome.exit_code, 0);

    set_env_var("PHPC_PHPT_INI_FLAGS", "-d date.timezone=Incorrect/Zone");
    let invalid = run_source(
        r#"<?php
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "\n";
"#,
    )
    .unwrap();
    assert_eq!(
        invalid.stdout,
        "Warning: PHP Startup: Invalid date.timezone value 'Incorrect/Zone', using 'UTC' instead in Unknown on line 0\nUTC|UTC\n"
    );
    assert_eq!(invalid.exit_code, 0);

    set_env_var("PHPC_PHPT_INI_FLAGS", "-d date.timezone=");
    let empty = run_source(
        r#"<?php
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "|", date("e"), "\n";
"#,
    )
    .unwrap();
    assert_eq!(
        empty.stdout,
        "Warning: PHP Startup: Invalid date.timezone value '', using 'UTC' instead in Unknown on line 0\nUTC|UTC|UTC\n"
    );
    assert_eq!(empty.exit_code, 0);

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);
}

#[test]
fn ini_set_date_timezone_updates_default_and_rejects_invalid_values() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    set_env_var("PHPC_PHPT_INI_FLAGS", "-d date.timezone=UTC");
    let execution = run_source(
        r#"<?php
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "|", date("e"), "\n";
var_dump(ini_set("date.timezone", "Europe/London"));
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "|", date("e"), "\n";
date_default_timezone_set("Europe/Rome");
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "|", date("e"), "\n";
var_dump(ini_set("date.timezone", "UTC"));
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "|", date("e"), "\n";
var_dump(ini_set("date.timezone", "Mars/Valles_Marineris"));
echo ini_get("date.timezone"), "|", date_default_timezone_get(), "|", date("e"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "UTC|UTC|UTC\nstring(3) \"UTC\"\nEurope/London|Europe/London|Europe/London\nEurope/London|Europe/Rome|Europe/Rome\nstring(13) \"Europe/London\"\nUTC|Europe/Rome|Europe/Rome\n\nWarning: ini_set(): Invalid date.timezone value 'Mars/Valles_Marineris', using 'UTC' instead in Command line code on line 9\nbool(false)\nUTC|Europe/Rome|Europe/Rome\n"
    );
    assert_eq!(execution.exit_code, 0);

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);
}

#[test]
fn date_default_timezone_set_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "date_default_timezone_set";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("UTC") ? "utc" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|utc");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_default_timezone_set_rejects_forms_outside_current_subset() {
    let arity = run_source(
        r#"<?php
date_default_timezone_set();
"#,
    )
    .unwrap();
    assert_eq!(arity.exit_code, 255);
    assert!(
        arity.stdout.contains("Too few arguments to function date_default_timezone_set(), 0 passed in Command line code on line 2 and exactly 1 expected"),
        "{}",
        arity.stdout
    );
    assert_eq!(arity.stderr, "");

    let type_error = runtime_error(
        r#"<?php
date_default_timezone_set(42);
"#,
    );
    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call date_default_timezone_set(): timezone identifier must be string in the current subset, got int"
    );
}

#[test]
fn emit_ir_rejects_date_default_timezone_set_until_native_time_state_exists() {
    let error = emit_ir_source(
        r#"<?php
date_default_timezone_set("UTC");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_date_default_timezone_set_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("date_default_timezone_set") ? "1" : "0";
echo is_callable("date_default_timezone_set") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
