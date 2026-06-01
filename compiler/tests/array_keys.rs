use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

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

#[test]
fn array_keys_filters_values_with_loose_scalar_comparison() {
    let source = r#"<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["empty"] = "";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["numeric-string"] = "10.0";
$items["text"] = "abc";

$empty = array_keys($items, "");
echo count($empty), "\n";
echo $empty[0], "|", $empty[1], "|", $empty[2], "\n";

$zero = array_keys($items, "0");
echo count($zero), "\n";
echo $zero[0], "|", $zero[1], "|", $zero[2], "\n";

$ten = array_keys($items, "10");
echo count($ten), "\n";
echo $ten[0], "|", $ten[1], "|", $ten[2], "\n";

$text = array_keys($items, "abc");
echo count($text), "\n";
echo $text[0], "\n";

$missing = array_keys($items, "missing");
echo count($missing), "\n";

$call = "array_keys";
$dynamic = $call($items, "10.0");
echo $dynamic[0], "|", $dynamic[1], "|", $dynamic[2];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3\nnull|false|empty\n3\nfalse|int-zero|string-zero\n3\nint-ten|string-ten|numeric-string\n1\ntext\n0\nint-ten|string-ten|numeric-string"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_loose_mode_matches_empty_arrays_like_php_membership_searches() {
    let source = r#"<?php
$items = [1 => "1", 0 => "0", -1 => "-1", 2 => null, 3 => [], "php" => "php", "" => ""];

function show_keys($keys) {
    echo count($keys), ":";
    foreach ($keys as $key) {
        echo "[", $key === "" ? "<empty>" : $key, "]";
    }
    echo "\n";
}

show_keys(array_keys($items, []));
show_keys(array_keys($items, false));
show_keys(array_keys($items, true));
show_keys(array_keys($items, null));
show_keys(array_keys($items, ""));
show_keys(array_keys($items, 0));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "2:[2][3]\n4:[0][2][3][<empty>]\n3:[1][-1][php]\n3:[2][3][<empty>]\n2:[2][<empty>]\n2:[0][2]\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_strict_mode_filters_values_with_scalar_identity() {
    let source = r#"<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["empty"] = "";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["numeric-string"] = "10.0";
$items["text"] = "abc";

$empty = array_keys($items, "", true);
echo count($empty), "\n";
echo $empty[0], "\n";

$false = array_keys($items, false, true);
echo count($false), "\n";
echo $false[0], "\n";

$int_zero = array_keys($items, 0, true);
echo count($int_zero), "\n";
echo $int_zero[0], "\n";

$string_zero = array_keys($items, "0", true);
echo count($string_zero), "\n";
echo $string_zero[0], "\n";

$float_ten = array_keys($items, 10.0, true);
echo count($float_ten), "\n";

$int_ten = array_keys($items, 10, true);
echo count($int_ten), "\n";
echo $int_ten[0], "\n";

$string_ten = array_keys($items, "10", true);
echo count($string_ten), "\n";
echo $string_ten[0], "\n";

$null = array_keys($items, null, true);
echo count($null), "\n";
echo $null[0], "\n";

$missing = array_keys($items, "missing", true);
echo count($missing), "\n";

$loose = array_keys($items, "10.0", false);
echo $loose[0], "|", $loose[1], "|", $loose[2], "\n";

$call = "array_keys";
$dynamic = $call($items, "abc", true);
echo $dynamic[0];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "1\nempty\n1\nfalse\n1\nint-zero\n1\nstring-zero\n0\n1\nint-ten\n1\nstring-ten\n1\nnull\n0\nint-ten|string-ten|numeric-string\ntext"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_strict_mode_filters_array_and_object_identity() {
    let source = r#"<?php
class Box {}

$box = new Box();
$other = new Box();
$items = [];
$items["array"] = ["value" => 1];
$items["object"] = $box;

$array_keys = array_keys($items, ["value" => 1], true);
echo count($array_keys), "\n";
echo $array_keys[0], "\n";

$missing_array = array_keys($items, ["value" => "1"], true);
echo count($missing_array), "\n";

$object_keys = array_keys($items, $box, true);
echo count($object_keys), "\n";
echo $object_keys[0], "\n";

$missing_object = array_keys($items, $other, true);
echo count($missing_object);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "1\narray\n0\n1\nobject\n0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_filters_resource_values_by_identity() {
    let execution = run_source(
        r#"<?php
$file = fopen(__FILE__, "r");
$dir = opendir(".");
$items = [$file, $dir];
foreach ([array_keys($items, $file), array_keys($items, $file, true), array_keys($items, $dir), array_keys($items, $dir, true)] as $keys) {
    echo count($keys), ":", $keys[0], "\n";
}
fclose($file);
closedir($dir);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1:0\n1:0\n1:1\n1:1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_rejects_non_bool_strict_mode_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_keys($items, 1, \"yes\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_keys(): strict mode argument must be bool in the current subset, got string"
    );
}

#[test]
fn array_keys_rejects_loose_object_search_gaps() {
    let object_error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$items = [$box];
echo array_keys($items, "needle");
"#,
    );

    assert_eq!(object_error.line, 5);
    assert_eq!(object_error.column, 6);
    assert_eq!(
        object_error.message,
        "unsupported call array_keys(): object search values and object values are not implemented"
    );
}

#[test]
fn emit_ir_rejects_array_keys_filter_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_keys([1], 1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
