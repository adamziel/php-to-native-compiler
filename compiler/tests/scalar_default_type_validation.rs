use php_compiler::run_source_with_source_file;

fn startup_fatal(source: &str, file: &str) -> String {
    let execution = run_source_with_source_file(source, file).unwrap();
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn float_parameter_rejects_bool_literal_default_at_startup() {
    let stderr = startup_fatal(
        r#"<?php

function test(float $arg = true)
{
    var_dump($arg);
}

test();
"#,
        "tests/type_declarations/scalar_float_with_invalid_default.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use bool as default value for parameter $arg of type float in tests/type_declarations/scalar_float_with_invalid_default.php on line 3"
    );
}

#[test]
fn scalar_literal_defaults_are_validated_by_type_category() {
    let stderr = startup_fatal(
        r#"<?php
function takesInt(int $value = 1.5) {}
"#,
        "tests/type_declarations/scalar_int_invalid_default.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Cannot use float as default value for parameter $value of type int in tests/type_declarations/scalar_int_invalid_default.php on line 2"
    );
}

#[test]
fn float_parameter_allows_integer_literal_default() {
    let execution = run_source_with_source_file(
        r#"<?php
function test(float $arg = 0)
{
    var_dump($arg);
}

test();
"#,
        "tests/type_declarations/scalar_float_with_integer_default.php",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.stdout, "float(0)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn union_parameter_defaults_are_validated_and_preserve_matching_scalar_type() {
    let execution = run_source_with_source_file(
        r#"<?php
function test(
    int|float $a = 1,
    int|float $b = 2.0,
    float|string $c = 3,
    float|string $d = 4.0,
    float|string $e = "5"
) {
    var_dump($a, $b, $c, $d, $e);
}

test();
"#,
        "Zend/tests/type_declarations/union_types/legal_default_values.php",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(
        execution.stdout,
        "int(1)\nfloat(2)\nfloat(3)\nfloat(4)\nstring(1) \"5\"\n"
    );
    assert_eq!(execution.exit_code, 0);

    let stderr = startup_fatal(
        r#"<?php
function test(int|float $arg = "0") {}
"#,
        "Zend/tests/type_declarations/union_types/illegal_default_value_argument.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use string as default value for parameter $arg of type int|float in Zend/tests/type_declarations/union_types/illegal_default_value_argument.php on line 2"
    );
}

#[test]
fn literal_bool_union_parameter_defaults_match_only_literal_arm() {
    let accepted_false = run_source_with_source_file(
        r#"<?php
function a(false|int $x = false) { var_dump($x); }
a();
"#,
        "literal-false-union-default.php",
    )
    .unwrap();
    assert_eq!(accepted_false.stderr, "");
    assert_eq!(accepted_false.stdout, "bool(false)\n");
    assert_eq!(accepted_false.exit_code, 0);

    let accepted_true = run_source_with_source_file(
        r#"<?php
function a(true|int $x = true) { var_dump($x); }
a();
"#,
        "literal-true-union-default.php",
    )
    .unwrap();
    assert_eq!(accepted_true.stderr, "");
    assert_eq!(accepted_true.stdout, "bool(true)\n");
    assert_eq!(accepted_true.exit_code, 0);

    let rejected_false = startup_fatal(
        r#"<?php
function a(false|int $x = true) { var_dump($x); }
a();
"#,
        "literal-false-union-opposite-default.php",
    );
    assert_eq!(
        rejected_false,
        "Fatal error: Cannot use bool as default value for parameter $x of type false|int in literal-false-union-opposite-default.php on line 2"
    );

    let rejected_true = startup_fatal(
        r#"<?php
function a(true|int $x = false) { var_dump($x); }
a();
"#,
        "literal-true-union-opposite-default.php",
    );
    assert_eq!(
        rejected_true,
        "Fatal error: Cannot use bool as default value for parameter $x of type true|int in literal-true-union-opposite-default.php on line 2"
    );
}

#[test]
fn bool_parameter_allows_boolean_literal_defaults() {
    let execution = run_source_with_source_file(
        r#"<?php
function foo(bool $x = true, bool $y = false) {
    var_dump($x, $y);
}
foo();
"#,
        "tests/type_declarations/default_boolean_hint_values.php",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.stdout, "bool(true)\nbool(false)\n");
    assert_eq!(execution.exit_code, 0);
}
