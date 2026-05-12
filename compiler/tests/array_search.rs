use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_search_returns_first_loose_scalar_match_key() {
    let source = r#"<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items[0] = "zero-key";
$items["2"] = "two-key";
$items["02"] = "zero-two-key";
$items[] = "appended";
$items["numeric"] = "10.0";
$items["text"] = "abc";

var_dump(array_search("", $items));
var_dump(array_search("0", $items));
var_dump(array_search("zero-key", $items));
var_dump(array_search("two-key", $items));
var_dump(array_search("zero-two-key", $items));
var_dump(array_search("appended", $items));
var_dump(array_search("10", $items));
var_dump(array_search("missing", $items));

$call = "array_search";
var_dump($call("abc", $items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "string(4) \"null\"\nstring(5) \"false\"\nint(0)\nint(2)\nstring(2) \"02\"\nint(3)\nstring(7) \"numeric\"\nbool(false)\nstring(4) \"text\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_search_requires_array_second_argument() {
    let error = runtime_error("<?php\necho array_search(\"name\", 42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_search(): second argument must be array, got int"
    );
}

#[test]
fn array_search_rejects_strict_mode_argument_until_implemented() {
    let error = runtime_error("<?php\n$items = [1];\necho array_search(1, $items, true);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_search(): strict mode argument is not implemented"
    );
}

#[test]
fn array_search_rejects_array_and_object_comparison_gaps() {
    let array_error =
        runtime_error("<?php\n$items = [[]];\necho array_search(\"needle\", $items);\n");

    assert_eq!(array_error.line, 3);
    assert_eq!(array_error.column, 6);
    assert_eq!(
        array_error.message,
        "unsupported call array_search(): array needles and array values are not implemented"
    );

    let object_error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$items = [$box];
echo array_search("needle", $items);
"#,
    );

    assert_eq!(object_error.line, 5);
    assert_eq!(object_error.column, 6);
    assert_eq!(
        object_error.message,
        "unsupported call array_search(): object needles and object values are not implemented"
    );
}
