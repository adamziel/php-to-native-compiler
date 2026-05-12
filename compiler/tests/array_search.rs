use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

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
fn array_search_strict_mode_uses_scalar_identity_and_returns_keys() {
    let source = r#"<?php
$items = [];
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["null"] = null;
$items[2] = "int-key";
$items["text"] = "abc";

var_dump(array_search("", $items, true));
var_dump(array_search(false, $items, true));
var_dump(array_search(0, $items, true));
var_dump(array_search("0", $items, true));
var_dump(array_search(10.0, $items, true));
var_dump(array_search(10, $items, true));
var_dump(array_search("10", $items, true));
var_dump(array_search(null, $items, true));
var_dump(array_search("int-key", $items, true));
var_dump(array_search("missing", $items, true));
var_dump(array_search("10.0", $items, false));

$call = "array_search";
var_dump($call("abc", $items, true));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(false)\nstring(5) \"false\"\nstring(8) \"int-zero\"\nstring(11) \"string-zero\"\nbool(false)\nstring(7) \"int-ten\"\nstring(10) \"string-ten\"\nstring(4) \"null\"\nint(2)\nbool(false)\nstring(7) \"int-ten\"\nstring(4) \"text\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_search_rejects_non_bool_strict_mode_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_search(1, $items, \"yes\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_search(): strict mode argument must be bool in the current subset, got string"
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

#[test]
fn emit_ir_rejects_array_search_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_search(1, [1], true);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
