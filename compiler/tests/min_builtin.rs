use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn min_and_max_execute_scalar_and_array_value_forms() {
    let execution = run_source(
        r#"<?php
echo min(128, PHP_INT_MAX), "|";
echo max(128, PHP_INT_MAX), "|";
echo min(5, -2, 9), "|";
echo max(5, -2, 9), "|";
echo min(3, 2.5), "|";
echo max(3, 2.5), "|";
$values = [3, 2, 5];
echo min($values), "|";
echo max($values), "|";
echo PHP_INT_MAX > 0 ? "max" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "128|9223372036854775807|-2|9|2.5|3|2|5|max"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn min_and_max_compare_arrays_after_scalars_like_php() {
    let execution = run_source(
        r#"<?php
$maximum = max(0, 1, [2, 3]);
$minimum = min(0, 1, [2, 3]);
echo is_array($maximum) ? "array" : "not-array";
echo "|";
echo $minimum;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn min_and_max_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$min = "min";
$max = "max";
echo function_exists($min) ? "yes" : "no";
echo "|";
echo is_callable($min) ? "callable" : "missing";
echo "|";
echo $min(9, 4, 7);
echo "|";
echo function_exists($max) ? "yes" : "no";
echo "|";
echo is_callable($max) ? "callable" : "missing";
echo "|";
echo $max(9, 4, 7);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|4|yes|callable|9");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn min_and_max_match_scalar_phpt_error_recovery_and_boundaries() {
    let execution = run_source(
        r#"<?php
foreach (["min", "max"] as $fn) {
    try {
        var_dump($fn(1));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }

    try {
        var_dump($fn([]));
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }

    try {
        var_dump($fn(new stdclass));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}

var_dump(min(2, 1, 2));
var_dump(min(2.1, 2.11, 2.09));
var_dump(min("", "t", "b"));
var_dump(min(false, true, false));
var_dump(min(true, false, true));
var_dump(min(1, true, false, true));
var_dump(min(0, true, false, true));

var_dump(max(2, 1, 2));
var_dump(max(2.1, 2.11, 2.09));
var_dump(max("", "t", "b"));
var_dump(max(false, true, false));
var_dump(max(true, false, true));
var_dump(max(1, true, false, true));
var_dump(max(0, true, false, true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "min(): Argument #1 ($value) must be of type array, int given\n\
min(): Argument #1 ($value) must contain at least one element\n\
min(): Argument #1 ($value) must be of type array, stdClass given\n\
max(): Argument #1 ($value) must be of type array, int given\n\
max(): Argument #1 ($value) must contain at least one element\n\
max(): Argument #1 ($value) must be of type array, stdClass given\n\
int(1)\n\
float(2.09)\n\
string(0) \"\"\n\
bool(false)\n\
bool(false)\n\
bool(false)\n\
int(0)\n\
int(2)\n\
float(2.11)\n\
string(1) \"t\"\n\
bool(true)\n\
bool(true)\n\
int(1)\n\
bool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn min_and_max_reject_current_subset_gaps() {
    let empty_array = run_source("<?php\nmin([]);\n").unwrap();
    assert!(empty_array.stdout.contains(
        "Fatal error: Uncaught ValueError: min(): Argument #1 ($value) must contain at least one element"
    ));
    assert_eq!(empty_array.exit_code, 255);

    let array_to_array = run_source("<?php\nmax([1], [2]);\n").unwrap_err();
    assert_eq!(array_to_array.phase, Phase::Runtime);
    assert_eq!(array_to_array.line, 2);
    assert_eq!(array_to_array.column, 1);
    assert_eq!(
        array_to_array.message,
        "unsupported call min()/max(): array-to-array ordering is not implemented in the current subset"
    );

    let single_non_array = run_source("<?php\nmin(3);\n").unwrap();
    assert!(single_non_array.stdout.contains(
        "Fatal error: Uncaught TypeError: min(): Argument #1 ($value) must be of type array, int given"
    ));
    assert_eq!(single_non_array.exit_code, 255);

    let no_args = run_source("<?php\nmax();\n").unwrap_err();
    assert_eq!(no_args.phase, Phase::Runtime);
    assert_eq!(no_args.line, 2);
    assert_eq!(no_args.column, 1);
    assert_eq!(
        no_args.message,
        "arity mismatch for max(): expected at least 1 argument(s), got 0"
    );
}

#[test]
fn emit_ir_folds_min_max_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo defined("PHP_INT_MAX") ? "1" : "0";
echo function_exists("min") ? "1" : "0";
echo is_callable("min") ? "1" : "0";
echo function_exists("max") ? "1" : "0";
echo is_callable("max") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nmin(3, 2);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nmax(3, 2);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
