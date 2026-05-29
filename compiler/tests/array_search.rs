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
    let source = r#"<?php
try {
    echo array_search("name", 42);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_search(): Argument #2 ($haystack) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
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
fn array_search_strict_mode_uses_array_and_object_identity() {
    let source = r#"<?php
class Box {}

$box = new Box();
$other = new Box();
$items = [];
$items["array"] = ["value" => 1];
$items["object"] = $box;

var_dump(array_search(["value" => 1], $items, true));
var_dump(array_search(["value" => "1"], $items, true));
var_dump(array_search($box, $items, true));
var_dump(array_search($other, $items, true));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "string(5) \"array\"\nbool(false)\nstring(6) \"object\"\nbool(false)\n"
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
fn array_search_uses_loose_array_and_object_membership() {
    let source = r#"<?php
class Box {}
$box = new Box();
$items = ["empty" => [], "array" => ["value" => 1], "object" => $box, "truthy" => true];

var_dump(array_search(null, $items));
var_dump(array_search([], $items));
var_dump(array_search(["value" => "1"], $items));
var_dump(array_search($box, $items));
var_dump(array_search(new Box(), $items, true));
var_dump(array_search(new Box(), $items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "string(5) \"empty\"\nstring(5) \"empty\"\nstring(5) \"array\"\nstring(6) \"object\"\nbool(false)\nstring(6) \"object\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_search_uses_singleton_unbacked_enum_cases() {
    let source = r#"<?php
enum Sample { case A; case B; }
$items = ["a" => Sample::A];

var_dump(array_search(Sample::A, $items, true));
var_dump(array_search(Sample::B, $items, true));
var_dump(array_search(Sample::A, $items));
var_dump(array_search(Sample::B, $items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "string(1) \"a\"\nbool(false)\nstring(1) \"a\"\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
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
