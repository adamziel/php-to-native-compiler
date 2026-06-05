use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn ini_get_reads_current_deterministic_registry() {
    let execution = run_source(
        r#"<?php
echo ini_get("memory_limit"), "|";
echo ini_get("MEMORY_LIMIT"), "|";
echo ini_get("mbstring.func_overload"), "|";
echo ini_get("missing.option") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "128M|128M|0|false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_set_updates_current_deterministic_registry() {
    let execution = run_source(
        r#"<?php
echo ini_set("display_errors", 0), "|";
echo ini_get("display_errors"), "|";
echo ini_set("DISPLAY_ERRORS", true), "|";
echo ini_get("display_errors"), "|";
echo ini_set("display_errors", null), "|";
echo ini_get("display_errors"), "|";
echo ini_set("missing.option", "x") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "|0|0|1|1||false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_restore_returns_null_and_restores_startup_values() {
    let execution = run_source(
        r#"<?php
echo ini_get("user_agent") === "" ? "default" : "unexpected";
echo "|";
echo ini_set("user_agent", "phpc");
echo "|";
echo ini_get("user_agent");
echo "|";
echo ini_restore("user_agent") === null ? "null" : "other";
echo "|";
echo ini_get("user_agent") === "" ? "restored" : ini_get("user_agent");
echo "|";
set_include_path("/tmp/phpc-include-path");
echo get_include_path();
echo "|";
ini_restore("include_path");
echo get_include_path();
echo "|";
echo ini_restore("missing.option") === null ? "missing-null" : "missing-other";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "default||phpc|null|restored|/tmp/phpc-include-path|.|missing-null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_set_rejects_startup_only_max_memory_limit() {
    let execution = run_source(
        r#"<?php
var_dump(ini_set("max_memory_limit", "128M"));
var_dump(ini_set("MAX_MEMORY_LIMIT", "256M"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "bool(false)\nbool(false)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_builtins_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$get = "ini_get";
$set = "ini_set";
$restore = "ini_restore";
echo function_exists($get) ? "yes" : "no";
echo "|";
echo is_callable($set) ? "callable" : "missing";
echo "|";
echo $set("memory_limit", "256M"), "|";
echo $get("memory_limit");
echo "|";
echo function_exists($restore) ? "yes" : "no";
echo "|";
echo $restore("memory_limit") === null ? "restored" : "bad";
echo "|";
echo $get("memory_limit");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|128M|256M|yes|restored|128M");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_builtins_reject_forms_outside_current_subset() {
    let non_string = run_source("<?php\nini_get(42);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call ini_get(): option argument must be string in the current subset, got int"
    );

    let non_string_set = run_source("<?php\nini_set(42, 'x');\n").unwrap_err();
    assert_eq!(non_string_set.phase, Phase::Runtime);
    assert_eq!(non_string_set.line, 2);
    assert_eq!(non_string_set.column, 1);
    assert_eq!(
        non_string_set.message,
        "unsupported call ini_set(): option argument must be string in the current subset, got int"
    );

    let array_value = run_source("<?php\nini_set('display_errors', []);\n").unwrap_err();
    assert_eq!(array_value.phase, Phase::Runtime);
    assert_eq!(array_value.line, 2);
    assert_eq!(array_value.column, 1);
    assert_eq!(
        array_value.message,
        "unsupported call ini_set(): value argument must be null or scalar in the current subset, got array"
    );

    let non_string_restore = run_source("<?php\nini_restore(42);\n").unwrap_err();
    assert_eq!(non_string_restore.phase, Phase::Runtime);
    assert_eq!(non_string_restore.line, 2);
    assert_eq!(non_string_restore.column, 1);
    assert_eq!(
        non_string_restore.message,
        "unsupported call ini_restore(): option argument must be string in the current subset, got int"
    );

    let too_many = run_source("<?php\nini_get('memory_limit', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for ini_get(): expected 1 argument(s), got 2"
    );

    let too_few = run_source("<?php\nini_set('memory_limit');\n").unwrap();
    assert_eq!(too_few.exit_code, 255);
    assert!(
        too_few.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function ini_set(), 1 passed"
        ),
        "{}",
        too_few.stdout
    );

    let restore_too_many = run_source("<?php\nini_restore('memory_limit', true);\n").unwrap_err();
    assert_eq!(restore_too_many.phase, Phase::Runtime);
    assert_eq!(restore_too_many.line, 2);
    assert_eq!(restore_too_many.column, 1);
    assert_eq!(
        restore_too_many.message,
        "arity mismatch for ini_restore(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_ini_get_metadata_but_rejects_direct_ini_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("ini_get") ? "1" : "0";
echo is_callable("ini_set") ? "1" : "0";
echo function_exists("ini_restore") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nini_get('memory_limit');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let set_error = emit_ir_source("<?php\nini_set('memory_limit', '256M');\n").unwrap_err();
    assert_eq!(set_error.phase, Phase::Codegen);
    assert_eq!(set_error.line, 2);
    assert_eq!(set_error.column, 1);
    assert_eq!(set_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let restore_error = emit_ir_source("<?php\nini_restore('memory_limit');\n").unwrap_err();
    assert_eq!(restore_error.phase, Phase::Codegen);
    assert_eq!(restore_error.line, 2);
    assert_eq!(restore_error.column, 1);
    assert_eq!(restore_error.message, LLVM_FUNCTION_CALL_REJECTION);
}
