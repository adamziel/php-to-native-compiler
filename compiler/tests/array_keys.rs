use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_keys_emits_integer_and_string_keys_in_order() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$keys = array_keys($items);
echo count($keys), "\n";
echo $keys[0], "|", $keys[1], "|", $keys[2], "|", $keys[3], "|", $keys[4], "|", $keys[5], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$call = "array_keys";
$again = $call($items);
echo $again[0], "|", $again[5];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "6\nname|5|2|02|-1|6\nAda|five|two updated|zero two|negative|next\nname|6"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_requires_array_argument() {
    let error = runtime_error("<?php\necho array_keys(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_keys(): argument must be array, got int"
    );
}
