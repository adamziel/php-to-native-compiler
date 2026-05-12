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
