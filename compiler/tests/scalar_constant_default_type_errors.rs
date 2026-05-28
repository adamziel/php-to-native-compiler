use php_compiler::{run_source, run_source_with_source_file};

#[test]
fn constant_default_type_mismatch_reports_uncaught_type_error() {
    let execution = run_source_with_source_file(
        r#"<?php
const STRING_VAL = "test";

function int_val(int $a = STRING_VAL): int {
    return $a;
}

var_dump(int_val());
"#,
        "/tmp/scalar_constant_defaults_error.php",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
    assert!(
        execution.stdout.contains(
            "Fatal error: Uncaught TypeError: int_val(): Argument #1 ($a) must be of type int, string given, called in /tmp/scalar_constant_defaults_error.php:4"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Stack trace:
#0 /tmp/scalar_constant_defaults_error.php(4): int_val()
#1 {main}"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("thrown in /tmp/scalar_constant_defaults_error.php on line 4"),
        "{}",
        execution.stdout
    );
}

#[test]
fn call_argument_type_mismatch_is_catchable_type_error() {
    let execution = run_source(
        r#"<?php
function typed(int $value) {
    echo "unreached";
}

try {
    typed("x");
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:typed(): Argument #1 ($value) must be of type int, string given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn valid_constant_defaults_still_materialize_at_call_time() {
    let execution = run_source(
        r#"<?php
const INT_VAL = 7;
function int_val(int $a = INT_VAL): int {
    return $a;
}
var_dump(int_val());
var_dump(int_val(9));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(7)
int(9)
"
    );
    assert_eq!(execution.exit_code, 0);
}
