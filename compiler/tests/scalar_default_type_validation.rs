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
