use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn in_array_uses_loose_scalar_comparison_in_insertion_order() {
    let source = r#"<?php
$items = [];
$items[] = null;
$items[] = false;
$items[] = 10;
$items[] = "10.0";
$items[] = "abc";

if (in_array("", $items)) {
    echo "empty-matches-null\n";
}
if (in_array("0", $items)) {
    echo "zero-matches-false\n";
}
if (in_array("10", $items)) {
    echo "numeric-string-matches-int\n";
}
if (in_array(10.0, $items)) {
    echo "float-matches-int\n";
}
if (in_array("abc", $items)) {
    echo "string-match\n";
}
if (in_array(11, $items)) {
    echo "unexpected-int\n";
} else {
    echo "missing-int\n";
}
if (in_array("missing", $items)) {
    echo "unexpected-string\n";
} else {
    echo "missing-string\n";
}

$call = "in_array";
if ($call("abc", $items)) {
    echo "dynamic-match";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "empty-matches-null\nzero-matches-false\nnumeric-string-matches-int\nfloat-matches-int\nstring-match\nmissing-int\nmissing-string\ndynamic-match"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_requires_array_second_argument() {
    let error = runtime_error("<?php\necho in_array(\"name\", 42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call in_array(): second argument must be array, got int"
    );
}

#[test]
fn in_array_strict_mode_uses_scalar_identity() {
    let source = r#"<?php
$items = [];
$items[] = false;
$items[] = 0;
$items[] = "0";
$items[] = 10;
$items[] = "10";
$items[] = null;
$items[] = "abc";

if (in_array("", $items, true)) {
    echo "unexpected-empty\n";
} else {
    echo "empty-missing\n";
}
if (in_array(false, $items, true)) {
    echo "false-match\n";
}
if (in_array(0, $items, true)) {
    echo "int-zero-match\n";
}
if (in_array("0", $items, true)) {
    echo "string-zero-match\n";
}
if (in_array(10.0, $items, true)) {
    echo "unexpected-float\n";
} else {
    echo "float-missing\n";
}
if (in_array(10, $items, true)) {
    echo "int-ten-match\n";
}
if (in_array("10", $items, true)) {
    echo "string-ten-match\n";
}
if (in_array(null, $items, true)) {
    echo "null-match\n";
}
if (in_array("missing", $items, true)) {
    echo "unexpected-missing\n";
} else {
    echo "string-missing\n";
}
if (in_array("10.0", $items, false)) {
    echo "false-flag-uses-loose\n";
}

$call = "in_array";
if ($call("abc", $items, true)) {
    echo "dynamic-strict-match";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "empty-missing\nfalse-match\nint-zero-match\nstring-zero-match\nfloat-missing\nint-ten-match\nstring-ten-match\nnull-match\nstring-missing\nfalse-flag-uses-loose\ndynamic-strict-match"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_rejects_non_bool_strict_mode_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho in_array(1, $items, \"yes\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call in_array(): strict mode argument must be bool in the current subset, got string"
    );
}

#[test]
fn in_array_rejects_array_and_object_comparison_gaps() {
    let array_error = runtime_error("<?php\n$items = [[]];\necho in_array(\"needle\", $items);\n");

    assert_eq!(array_error.line, 3);
    assert_eq!(array_error.column, 6);
    assert_eq!(
        array_error.message,
        "unsupported call in_array(): array needles and array values are not implemented"
    );

    let object_error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$items = [$box];
echo in_array("needle", $items);
"#,
    );

    assert_eq!(object_error.line, 5);
    assert_eq!(object_error.column, 6);
    assert_eq!(
        object_error.message,
        "unsupported call in_array(): object needles and object values are not implemented"
    );
}

#[test]
fn emit_ir_rejects_in_array_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho in_array(1, [1], true);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
