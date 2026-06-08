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
fn min_and_max_reject_current_subset_gaps() {
    let empty_array = run_source("<?php\nmin([]);\n").unwrap_err();
    assert_eq!(empty_array.phase, Phase::Runtime);
    assert_eq!(empty_array.line, 2);
    assert_eq!(empty_array.column, 1);
    assert_eq!(
        empty_array.message,
        "unsupported call min(): empty array argument forms are not implemented in the current subset"
    );

    let array_to_array = run_source("<?php\nmax([1], [2]);\n").unwrap_err();
    assert_eq!(array_to_array.phase, Phase::Runtime);
    assert_eq!(array_to_array.line, 2);
    assert_eq!(array_to_array.column, 1);
    assert_eq!(
        array_to_array.message,
        "unsupported call min()/max(): array-to-array ordering is not implemented in the current subset"
    );

    let too_few = run_source("<?php\nmin(3);\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for min(): expected at least 2 argument(s), got 1"
    );

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
