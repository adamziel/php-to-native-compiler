use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn undefined_variable_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho $missing;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn user_function_arity_mismatch_has_stable_runtime_error() {
    let error = runtime_error(
        "<?php\nfunction identity($value) {\n    return $value;\n}\necho identity();\n",
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for identity(): expected 1 argument(s), got 0"
    );
}

#[test]
fn unsupported_builtin_call_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho count(1);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call count(): only arrays are supported"
    );
}

#[test]
fn unsupported_array_key_has_stable_runtime_error() {
    let error = runtime_error("<?php\n$items = [true => \"yes\"];\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 11);
    assert_eq!(
        error.message,
        "invalid array key: bool keys are not supported; only int and string keys are implemented"
    );
}

#[test]
fn invalid_arithmetic_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho 1 / 0;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "invalid arithmetic for /: division by zero");
}

#[test]
fn non_numeric_string_arithmetic_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho \"abc\" + 1;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for +: string is not numeric"
    );
}

#[test]
fn isset_can_check_undefined_variables_without_reading_them() {
    let execution = run_source("<?php\necho isset($missing);\n$x = 1;\necho isset($x);\n")
        .expect("isset should not throw for missing direct variables");

    assert_eq!(execution.stdout, "1");
}
