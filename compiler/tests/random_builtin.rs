use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn rand_returns_current_deterministic_no_arg_value() {
    let execution = run_source(
        r#"<?php
$call = "rand";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo rand();
echo "|";
echo $call();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|123456789|123456789");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn rand_rejects_min_max_forms_for_now() {
    let error = run_source(
        r#"<?php
rand(1, 10);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call rand(): min/max arguments are not implemented; call rand() without arguments in the current subset"
    );
}

#[test]
fn array_rand_returns_deterministic_key_subset_and_metadata() {
    let execution = run_source(
        r#"<?php
$call = "array_rand";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$array = ["first" => 1, 4 => 2, "02" => 3];
echo "|";
echo array_rand($array);
$keys = $call($array, 3);
echo "|";
echo $keys[0];
echo "|";
echo $keys[1];
echo "|";
echo $keys[2];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|first|first|4|02");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_rand_reports_php_value_errors() {
    let execution = run_source(
        r#"<?php
try {
    array_rand([], 0);
} catch (ValueError $e) {
    echo "ValueError:" . $e->getMessage() . "\n";
}
try {
    array_rand([1, 2, 3], 0);
} catch (ValueError $e) {
    echo "ValueError:" . $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:array_rand(): Argument #1 ($array) must not be empty\nValueError:array_rand(): Argument #2 ($num) must be between 1 and the number of elements in argument #1 ($array)"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_rand_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("rand") ? "1" : "0";
echo is_callable("rand") ? "1" : "0";
echo function_exists("array_rand") ? "1" : "0";
echo is_callable("array_rand") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nrand();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\narray_rand(['x']);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
