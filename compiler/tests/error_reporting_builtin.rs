use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn error_reporting_tracks_current_integer_mask_state() {
    let execution = run_source(
        r#"<?php
echo error_reporting();
echo "|";
echo error_reporting(0);
echo "|";
echo error_reporting();
echo "|";
echo error_reporting(E_ERROR | E_WARNING | E_PARSE);
echo "|";
echo error_reporting();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "30719|30719|0|0|7");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_exposes_current_php_error_constants_and_strict_deprecation() {
    let execution = run_source(
        r#"<?php
echo E_ERROR, "|", E_WARNING, "|", E_PARSE, "|", E_CORE_ERROR, "|";
echo E_CORE_WARNING, "|", E_COMPILE_ERROR, "|", E_USER_ERROR, "|";
echo E_USER_WARNING, "|", E_RECOVERABLE_ERROR, "|", E_ALL, "\n";
var_dump(E_STRICT);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|2|4|16|32|64|256|512|4096|30719\n\
\n\
Deprecated: Constant E_STRICT is deprecated since 8.4, the error level was removed in Command line code on line 5\n\
int(2048)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "error_reporting";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(0);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|30719");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_coerces_nullable_int_masks_and_reports_php_type_errors() {
    let execution = run_source(
        r#"<?php
echo error_reporting(null), "|";
echo error_reporting("7"), "|";
echo error_reporting(), "|";
echo error_reporting(false), "|";
echo error_reporting();

foreach ([[], new stdClass()] as $mask) {
    try {
        error_reporting($mask);
    } catch (Throwable $e) {
        echo "\n", $e::class, ": ", $e->getMessage();
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "30719|30719|7|7|0\n\
TypeError: error_reporting(): Argument #1 ($error_level) must be of type ?int, array given\n\
TypeError: error_reporting(): Argument #1 ($error_level) must be of type ?int, stdClass given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_rejects_forms_outside_current_subset() {
    let too_many = runtime_error(
        r#"<?php
error_reporting(0, 1);
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for error_reporting(): expected 0 to 1 argument(s), got 2"
    );
}

#[test]
fn error_control_records_last_error_and_restores_php_masks() {
    let execution = run_source(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
function enable_warnings_inside_at() {
    echo $suppressed;
    error_reporting(E_ALL);
    echo $visible;
}
@enable_warnings_inside_at();
echo "|", error_reporting(), "|";

error_reporting(E_ALL);
function disable_warnings_inside_at() {
    echo $suppressed_again;
    error_reporting(0);
}
@disable_warnings_inside_at();
echo error_reporting(), "|";

@$a = $missing;
$last = error_get_last();
echo $last["type"], ":", $last["message"];
error_clear_last();
echo "|", error_get_last() === null ? "cleared" : "set";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: Undefined variable $visible in Command line code on line 6\n\
|30719|30719|2:Undefined variable $missing|cleared"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trigger_error_levels_and_repeated_error_ini_are_php_shaped() {
    let previous = std::env::var("PHPC_PHPT_INI_FLAGS").ok();
    std::env::set_var("PHPC_PHPT_INI_FLAGS", "-d ignore_repeated_errors=1");
    let execution = run_source(
        r#"<?php
trigger_error("notice");
trigger_error("warning", E_USER_WARNING);
trigger_error("deprecated", E_USER_DEPRECATED);
try {
    trigger_error("bad", 0);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
$u + $u;
"#,
    )
    .unwrap();
    if let Some(previous) = previous {
        std::env::set_var("PHPC_PHPT_INI_FLAGS", previous);
    } else {
        std::env::remove_var("PHPC_PHPT_INI_FLAGS");
    }

    assert_eq!(
        execution.stdout,
        "Notice: notice in Command line code on line 2\n\
\n\
Warning: warning in Command line code on line 3\n\
\n\
Deprecated: deprecated in Command line code on line 4\n\
trigger_error(): Argument #2 ($error_level) must be one of E_USER_ERROR, E_USER_WARNING, E_USER_NOTICE, or E_USER_DEPRECATED\n\
\n\
Warning: Undefined variable $u in Command line code on line 10\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_error_reporting_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("error_reporting") ? "1" : "0";
echo is_callable("error_reporting") ? "1" : "0";
echo defined("E_ALL") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nerror_reporting(E_ALL);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
