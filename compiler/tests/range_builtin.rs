use php_compiler::emit_ir_source;
use php_compiler::run_source;

#[test]
fn range_builds_integer_float_and_byte_arrays() {
    let execution = run_source(
        r#"<?php
echo implode(",", range(1, 5)), "\n";
echo implode(",", range(5, 1, 2)), "\n";
foreach (range(1, 1.5, 0.25) as $value) {
    echo $value, ";";
}
echo "\n";
echo implode("", range("a", "e", 2)), "\n";
echo implode("", range("e", "a", -2));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1,2,3,4,5\n5,3,1\n1;1.25;1.5;\nace\neca");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn range_matches_numeric_string_and_decimal_step_edges() {
    let execution = run_source(
        r#"<?php
echo implode(",", range("1", "9")), "\n";
echo implode(",", range("1", "10")), "\n";
echo implode(",", range("9", "A")), "\n";
foreach (range("1", "2", .1) as $value) {
    echo $value, ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1,2,3,4,5,6,7,8,9\n1,2,3,4,5,6,7,8,9,10\n9,:,;,<,=,>,?,@,A\n1;1.1;1.2;1.3;1.4;1.5;1.6;1.7;1.8;1.9;2;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn range_reports_php_value_errors_for_invalid_steps() {
    let zero = run_source(
        r#"<?php
try {
    range(1, 3, 0);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    range(1, 3, -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    range("a", "c", 100);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        zero.stdout,
        "range(): Argument #3 ($step) cannot be 0\nrange(): Argument #3 ($step) must be greater than 0 for increasing ranges\nrange(): Argument #3 ($step) must be less than the range spanned by argument #1 ($start) and argument #2 ($end)"
    );
    assert_eq!(zero.exit_code, 0);
}

#[test]
fn range_reports_function_metadata() {
    let execution = run_source(
        r#"<?php
echo function_exists("range") ? "1" : "0";
echo is_callable("range") ? "1" : "0";
$rf = new ReflectionFunction("range");
echo "|", $rf->getNumberOfRequiredParameters(), "/", $rf->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11|2/3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_range_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("range") ? "1" : "0";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
}
