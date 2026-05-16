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
fn foreach_by_reference_copy_back_updates_direct_array_variable_elements() {
    let source = r#"<?php
$items = ["a", ["nested" => "b"]];

foreach ($items as $key => &$item) {
    if (is_array($item)) {
        $item["seen"] = $key;
    } else {
        $item = $item . "!";
    }
}
unset($item);

echo $items[0], "|", $items[1]["nested"], "|", $items[1]["seen"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_lingers_as_last_direct_array_slot_after_loop() {
    let source = r#"<?php
$items = ["a", "b", "c"];

foreach ($items as $key => &$item) {
    $item = $item . $key;
}

$items[2] = "direct";
echo $item;
echo "|";
$item = "tail";
echo $items[0], "|", $items[1], "|", $items[2], "|", $key, "|", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "direct|a0|b1|tail|2|tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_lingering_reference_is_cleared_by_unset_or_empty_iteration() {
    let source = r#"<?php
$items = ["a", "b", "c"];

foreach ($items as &$item) {
    $item = $item . "!";
}

unset($item);
$item = "tail";
echo $items[0], "|", $items[1], "|", $items[2], "|", $item;
echo "\n";

$empty = [];
$value = "before";
foreach ($empty as &$value) {
    $value = "unreached";
}
$value = "after";
echo count($empty), "|", $value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b!|c!|tail\n0|after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_value_assignment_reuses_existing_lingering_reference_like_php() {
    let source = r#"<?php
$items = ["a", "b", "c"];

foreach ($items as &$item) {
    $item = $item . "!";
}

foreach (["x"] as $item) {
}

echo $items[0], "|", $items[1], "|", $items[2], "|", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b!|x|x");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_rejects_non_direct_iterables_as_stable_boundary() {
    let error = runtime_error(
        r#"<?php
foreach ([1] as &$item) {
    echo $item;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call foreach: by-reference iteration currently requires a direct array variable"
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
