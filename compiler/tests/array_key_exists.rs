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
    let error = runtime_error("<?php\n$items = [];\necho array_key_exists(true, $items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid array key: bool keys are not supported; only int and string keys are implemented"
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
