use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_values_reindexes_ordered_array_values() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

$values = array_values($items);
echo count($values), "\n";
echo $values[0], "|", $values[1], "|", $values[2], "|", $values[3], "|", $values[4], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[6], "\n";

$call = "array_values";
$again = $call($items);
echo $again[0], "|", $again[4];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "5\nAda|five|two updated|zero two|next\nAda|five|two updated|zero two|next\nAda|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_values_requires_array_argument() {
    let error = runtime_error("<?php\necho array_values(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_values(): argument must be array, got int"
    );
}
