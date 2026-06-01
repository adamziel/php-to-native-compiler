use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn rand_and_mt_rand_support_ranges_and_metadata() {
    let execution = run_source(
        r#"<?php
$names = ["rand", "mt_rand", "getrandmax", "mt_getrandmax", "srand", "mt_srand"];
foreach ($names as $name) {
    echo function_exists($name) ? "yes" : "no";
    echo is_callable($name) ? ":callable|" : ":missing|";
}
echo rand();
echo "|";
echo mt_rand();
echo "|";
echo rand(20, 10);
echo "|";
echo mt_rand(10, 20);
echo "|";
echo getrandmax();
echo "|";
echo mt_getrandmax();
srand(1234);
mt_srand(null, 0);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes:callable|yes:callable|yes:callable|yes:callable|yes:callable|yes:callable|123456789|123456789|15|15|2147483647|2147483647"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mt_rand_reports_php_value_error_for_inverted_range() {
    let execution = run_source(
        r#"<?php
try {
    mt_rand(20, 10);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "mt_rand(): Argument #2 ($max) must be greater than or equal to argument #1 ($min)"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn random_int_bytes_and_lcg_value_use_php_shapes() {
    let execution = run_source(
        r#"<?php
echo is_int(random_int(10, 100)) ? "int" : "bad";
echo "|";
echo random_int(42, 42);
echo "|";
echo strlen(bin2hex(random_bytes(16)));
echo "|";
echo is_string(random_bytes(10)) ? "string" : "bad";
echo "|";
$lcg = lcg_value();
echo is_float($lcg) && $lcg >= 0 && $lcg <= 1 ? "float" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int|42|32|string|float");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn random_int_and_bytes_report_php_value_errors() {
    let execution = run_source(
        r#"<?php
try {
    random_int(42, 0);
} catch (ValueError $e) {
    echo $e->getMessage() . "\n";
}
try {
    random_bytes(0);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "random_int(): Argument #1 ($min) must be less than or equal to argument #2 ($max)\nrandom_bytes(): Argument #1 ($length) must be greater than 0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_reports_random_function_arity() {
    let execution = run_source(
        r#"<?php
$rf = new ReflectionFunction('random_bytes');
echo $rf->getNumberOfParameters();
echo "|";
echo $rf->getNumberOfRequiredParameters();
echo "|";
$rf = new ReflectionFunction('random_int');
echo $rf->getNumberOfParameters();
echo "|";
echo $rf->getNumberOfRequiredParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|1|2|2");
    assert_eq!(execution.exit_code, 0);
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
