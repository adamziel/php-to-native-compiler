use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_key_exists_checks_keys_without_null_filtering() {
    let source = r#"<?php
$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["2"] = "two";
$items["02"] = "zero two";
$key = "present";

if (array_key_exists($key, $items)) {
    echo "present:exists\n";
}
if (array_key_exists("null", $items)) {
    echo "null:exists\n";
}
if (isset($items["null"])) {
    echo "null:isset\n";
} else {
    echo "null:not-set\n";
}
if (array_key_exists("missing", $items)) {
    echo "missing:exists\n";
} else {
    echo "missing:absent\n";
}
if (array_key_exists(2, $items)) {
    echo "int-normalized:exists\n";
}
if (array_key_exists("02", $items)) {
    echo "leading-zero-string:exists\n";
}
$exists = "array_key_exists";
if ($exists("present", $items)) {
    echo "dynamic:exists";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "present:exists\nnull:exists\nnull:not-set\nmissing:absent\nint-normalized:exists\nleading-zero-string:exists\ndynamic:exists"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_rejects_unsupported_key_types() {
    let error = runtime_error("<?php\n$items = [];\necho array_key_exists([], $items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid array key: array keys are not supported for array_key_exists(); only null, bool, int, string, and integral finite float keys are implemented"
    );
}

#[test]
fn array_key_exists_accepts_null_and_bool_key_coercions() {
    let source = r#"<?php
$items = [];
$items[""] = "empty";
$items[0] = "zero";
$items[1] = "one";
$items["01"] = "string one";

if (array_key_exists(null, $items)) {
    echo "null:exists\n";
}
if (array_key_exists(false, $items)) {
    echo "false:exists\n";
}
if (array_key_exists(true, $items)) {
    echo "true:exists\n";
}
if (array_key_exists("01", $items)) {
    echo "string-one:exists\n";
}

$call = "array_key_exists";
if ($call(false, $items)) {
    echo "dynamic:false";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "null:exists\nfalse:exists\ntrue:exists\nstring-one:exists\ndynamic:false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_accepts_integral_float_key_coercions() {
    let source = r#"<?php
$items = [];
$items[-1] = "minus";
$items[0] = "zero";
$items[1] = "one";
$items[2] = "two";

if (array_key_exists(1.0, $items)) {
    echo "one:exists\n";
}
if (array_key_exists(2.0, $items)) {
    echo "two:exists\n";
}
if (array_key_exists(-1.0, $items)) {
    echo "minus:exists\n";
}

$call = "array_key_exists";
if ($call(0.0, $items)) {
    echo "dynamic:zero";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "one:exists\ntwo:exists\nminus:exists\ndynamic:zero"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_rejects_lossy_float_key_coercions() {
    let error =
        runtime_error("<?php\n$items = [1 => \"one\"];\necho array_key_exists(1.5, $items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid array key: lossy or non-finite float keys are not supported for array_key_exists(); only null, bool, int, string, and integral finite float keys are implemented"
    );
}

#[test]
fn array_key_exists_requires_array_second_argument() {
    let error = runtime_error("<?php\necho array_key_exists(\"name\", 42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_key_exists(): second argument must be array, got int"
    );
}
