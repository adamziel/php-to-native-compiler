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

fn remove_env_var(name: &str) {
    env::remove_var(name);
}

#[test]
fn getenv_reads_present_empty_and_missing_environment_variables() {
    let _guard = env_lock();
    set_env_var("PHPC_GETENV_PRESENT", "visible");
    set_env_var("PHPC_GETENV_EMPTY", "");
    remove_env_var("PHPC_GETENV_MISSING");

    let execution = run_source(
        r#"<?php
echo getenv("PHPC_GETENV_PRESENT"), "|";
echo getenv("PHPC_GETENV_EMPTY") === "" ? "empty" : "not-empty";
echo "|";
echo getenv("PHPC_GETENV_MISSING") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "visible|empty|false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getenv_without_name_returns_string_keyed_environment_snapshot() {
    let _guard = env_lock();
    set_env_var("PHPC_GETENV_ARRAY_PRESENT", "array-visible");

    let execution = run_source(
        r#"<?php
$env = getenv();
echo is_array($env) ? "array" : "not-array";
echo "|";
echo array_key_exists("PHPC_GETENV_ARRAY_PRESENT", $env) ? $env["PHPC_GETENV_ARRAY_PRESENT"] : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|array-visible");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getenv_is_available_through_string_valued_calls() {
    let _guard = env_lock();
    set_env_var("PHPC_GETENV_DYNAMIC", "dynamic-visible");

    let execution = run_source(
        r#"<?php
$call = "getenv";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("PHPC_GETENV_DYNAMIC");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|dynamic-visible");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getenv_rejects_forms_outside_current_subset() {
    let non_string = run_source("<?php\ngetenv([]);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call getenv(): name argument must be string or null in the current subset, got array"
    );

    let non_bool_local_only = run_source("<?php\ngetenv('PATH', 'yes');\n").unwrap_err();
    assert_eq!(non_bool_local_only.phase, Phase::Runtime);
    assert_eq!(non_bool_local_only.line, 2);
    assert_eq!(non_bool_local_only.column, 1);
    assert_eq!(
        non_bool_local_only.message,
        "unsupported call getenv(): local_only argument must be bool in the current subset, got string"
    );

    let too_many = run_source("<?php\ngetenv('PATH', false, false);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for getenv(): expected 0 to 2 argument(s), got 3"
    );
}

#[test]
fn emit_ir_folds_getenv_metadata_but_rejects_direct_getenv_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("getenv") ? "1" : "0";
echo is_callable("getenv") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho getenv('PATH');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn putenv_sets_updates_and_unsets_environment_variables() {
    let _guard = env_lock();
    remove_env_var("PHPC_PUTENV_VALUE");

    let execution = run_source(
        r#"<?php
var_dump(function_exists("putenv"));
var_dump(is_callable("putenv"));
var_dump(putenv("PHPC_PUTENV_VALUE=alpha"));
echo getenv("PHPC_PUTENV_VALUE"), "|";
var_dump(putenv("PHPC_PUTENV_VALUE=beta"));
echo getenv("PHPC_PUTENV_VALUE"), "|";
var_dump(putenv("PHPC_PUTENV_VALUE"));
var_dump(getenv("PHPC_PUTENV_VALUE"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(true)\nalpha|bool(true)\nbeta|bool(true)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn putenv_rejects_forms_outside_current_subset() {
    let non_string = run_source("<?php\nputenv([]);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call putenv(): assignment argument must be string in the current subset, got array"
    );

    let too_many = run_source("<?php\nputenv('A=B', 'C=D');\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for putenv(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_putenv_metadata_but_rejects_direct_putenv_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("putenv") ? "1" : "0";
echo is_callable("putenv") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nputenv('PHPC_PUTENV_IR=value');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
