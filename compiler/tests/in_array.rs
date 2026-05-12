use php_compiler::error::Phase;
use php_compiler::run_source;

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
fn in_array_rejects_strict_mode_argument_until_implemented() {
    let error = runtime_error("<?php\n$items = [1];\necho in_array(1, $items, true);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call in_array(): strict mode argument is not implemented"
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
