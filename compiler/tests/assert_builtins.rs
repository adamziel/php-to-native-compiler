use std::env;
use std::sync::{Mutex, MutexGuard, OnceLock};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("environment lock is not poisoned")
}

fn restore_env_var(name: &str, previous: Option<std::ffi::OsString>) {
    if let Some(value) = previous {
        env::set_var(name, value);
    } else {
        env::remove_var(name);
    }
}

#[test]
fn assert_builtin_accepts_truthy_assertions() {
    let execution = run_source(
        r#"<?php
class ParagonIE_Sodium_Compat {}
echo assert(true) ? "1" : "0";
echo assert(1, "ok") ? "1" : "0";
echo assert("false") ? "1" : "0";
echo assert(class_exists("ParagonIE_Sodium_Compat"), "Possible filesystem/autoloader bug?") ? "1" : "0";
$call = "assert";
echo $call(true, null) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11111");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn assert_builtin_evaluates_arguments_left_to_right() {
    let execution = run_source(
        r#"<?php
function mark($label) {
    echo $label;
    return true;
}

assert(mark("A"), mark("B"));
echo "C";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ABC");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn assert_builtin_reports_current_exception_failure_boundary() {
    let error = run_source(
        r#"<?php
assert(false, "boom");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call assert(): AssertionError exceptions are not implemented in the current subset"
    );
}

#[test]
fn assert_builtin_warns_and_returns_false_when_exception_disabled() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    env::set_var("PHPC_PHPT_INI_FLAGS", "-d assert.exception=0");
    let execution = run_source(
        r#"<?php
var_dump(assert(false));
"#,
    )
    .unwrap();
    let stdout = execution.stdout.clone();
    let exit_code = execution.exit_code;

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);

    assert!(stdout.contains("Deprecated: PHP Startup: assert.exception INI setting is deprecated"));
    assert!(stdout.contains("Warning: assert(): assert(false) failed"));
    assert!(stdout.ends_with("bool(false)\n"));
    assert_eq!(exit_code, 0);
}

#[test]
fn assert_builtin_rejects_unsupported_description_values() {
    let error = run_source(
        r#"<?php
assert(true, []);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call assert(): description argument must be null, bool, int, float, or string in the current subset, got array"
    );
}

#[test]
fn emit_ir_rejects_direct_assert_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\nassert(true);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
