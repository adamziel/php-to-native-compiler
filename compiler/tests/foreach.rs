use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn foreach_iterates_ordered_array_values() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

foreach ($items as $item) {
    echo $item, "|";
}
echo "\nlast:", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Ada|five|two updated|zero two|next|\nlast:next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_iterates_ordered_array_keys_and_values() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
echo "\nlast:", $key, "=", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "name:Ada|5:five|2:two updated|02:zero two|6:next|\nlast:6=next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_consumes_innermost_break_and_continue() {
    let source = r#"<?php
$items = [1, 2, 3, 4, 5];

foreach ($items as $item) {
    if ($item == 2) {
        continue;
    }
    if ($item == 4) {
        break;
    }
    echo $item, ",";
}
echo "after:", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "1,3,after:4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_syntax_inside_unexecuted_function_body_is_registered() {
    let execution = php_compiler::run_source(
        r#"<?php
function sort_recursive(&$items) {
    foreach ($items as &$item) {
        if (is_array($item)) {
            sort_recursive($item);
        }
    }
}
echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_reports_stable_runtime_boundary_when_reached() {
    let error = runtime_error(
        r#"<?php
$items = [1];
foreach ($items as &$item) {
    echo $item;
}
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call foreach: by-reference iteration is not implemented; only by-value iteration is supported"
    );
}

#[test]
fn foreach_key_value_requires_array_iterable() {
    let error = runtime_error(
        r#"<?php
foreach (42 as $key => $value) {
    echo $key, $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays in the current subset, got int"
    );
}

#[test]
fn foreach_key_value_rejects_object_iteration() {
    let error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
foreach ($box as $key => $value) {
    echo $key, $value;
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays in the current subset, got object"
    );
}

#[test]
fn foreach_requires_array_iterable() {
    let error = runtime_error(
        r#"<?php
foreach (42 as $value) {
    echo $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays in the current subset, got int"
    );
}
