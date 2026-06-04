use php_compiler::error::{Diagnostic, Phase};
use php_compiler::run_source;

fn runtime_error(source: &str) -> Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn ascii_string_offset_writes_keep_copy_and_reference_semantics() {
    let execution = run_source(
        r#"<?php
$s = "abc";
$copy = $s;
$s[1] = "X";
echo $s, "|", $copy, "|";

$alias =& $s;
$alias[2] = "Y";
echo $s, "|", $alias, "|";

$s[5] = "Z";
echo $s, "|", $s[0], "|", $s[5], "|";

$s[-1] = "Q";
echo $s, "|", $s[-1];

class Box {
    public $text = "cat";
}

$box = new Box();
$propertyCopy = $box->text;
$box->text[1] = "U";
echo "|", $box->text, "|", $propertyCopy;

$propertyAlias =& $box->text;
$propertyAlias[2] = "T";
echo "|", $box->text, "|", $propertyAlias;

$items = ["name" => "abcd"];
$itemsCopy = $items;
$items["name"][1] = "X";
echo "|", $items["name"], "|", $itemsCopy["name"], "|", $items["name"][1];

$itemAlias =& $items["name"];
$itemAlias[2] = "Y";
echo "|", $items["name"], "|", $itemAlias;

$numeric = "wxyz";
$numericCopy = $numeric;
$numeric["01"] = "A";
$numeric["+2"] = "B";
echo "|", $numeric, "|", $numericCopy, "|", $numeric[" 1"];

$box->text = "dog";
$box->text["02"] = "G";
echo "|", $box->text;

$items["name"] = "rust";
$items["name"][" 2"] = "S";
echo "|", $items["name"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "aXc|abc|aXY|aXY|aXY  Z|a|Z|aXY  Q|Q|cUt|cat|cUT|cUT|aXcd|abcd|X|aXYd|aXYd|wABz|wxyz|A|doG|ruSt"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn binary_and_multibyte_string_offset_reads_are_byte_based() {
    let execution = run_source(
        r#"<?php
$binary = html_entity_decode("&#xA0;", ENT_QUOTES, "ISO-8859-1");
echo bin2hex($binary[0]), "|";
$utf8 = "å";
echo bin2hex($utf8[0]), "|", bin2hex($utf8[1]);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "a0|c3|a5");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn out_of_range_string_offset_reads_and_writes_recover_with_php_warnings() {
    let execution = run_source(
        r#"<?php
set_error_handler(function($code, $message) {
    echo "W:$message\n";
});
$s = "abcdef";
echo "[", $s[-10], "]|", $s[-3], "|";
$s[-20] = "Y";
echo $s, "|";
$s[-2] = "UFO";
echo $s;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "[W:Uninitialized string offset -10\n]|d|W:Illegal string offset -20\nabcdef|W:Only the first byte will be assigned to the string offset\nabcdUf"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn isset_and_empty_probe_string_offsets_without_array_key_fatals() {
    let execution = run_source(
        r#"<?php
set_error_handler(function($code, $message) {
    echo "W:$message\n";
});
$s = "test0123";
var_dump(isset($s[-1]));
var_dump(isset($s[-10]));
var_dump(empty($s[-4]));
var_dump(empty($s["good"]));
var_dump(isset($s[[]]));
var_dump(empty($s[new stdClass()]));
var_dump(isset($s[1.5]));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\nW:Implicit conversion from float 1.5 to int loses precision\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_literal_reference_to_string_offset_reports_php_error() {
    let execution = run_source(
        r#"<?php
$a = "aaa";
$x = array(&$a[1]);
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Cannot create references to/from string offsets"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn string_offset_reference_errors_are_catchable() {
    let execution = run_source(
        r#"<?php
function &test() : string {
    $str = "foo";
    return $str[0];
}

function &gen() {
    $str = "foo";
    yield $str[0];
}

try {
    test();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}

try {
    $str = "foo";
    $str[0] =& $str[1];
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}

try {
    foreach (gen() as $value) {}
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Cannot create references to/from string offsets\nCannot create references to/from string offsets\nCannot create references to/from string offsets\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_offset_cast_writes_respect_error_handler_target_mutation() {
    let direct = run_source(
        r#"<?php
set_error_handler(function($code, $message) {
    echo "Err: $message\n";
    $GLOBALS["a"] = null;
});
$a[$y] = $a .= ($y);
var_dump($a);
"#,
    )
    .unwrap();

    assert_eq!(
        direct.stdout,
        "Err: Undefined variable $y\nErr: Undefined variable $y\nErr: String offset cast occurred\nNULL\n"
    );
    assert_eq!(direct.exit_code, 0);

    let nested = run_source(
        r#"<?php
set_error_handler(function($code, $message) {
    echo "Err: $message\n";
    $GLOBALS["a"] = "";
});
$a = ["a"];
$a[0][$d] = "b";
var_dump($a);
"#,
    )
    .unwrap();

    assert_eq!(
        nested.stdout,
        "Err: Undefined variable $d\nErr: String offset cast occurred\nstring(0) \"\"\n"
    );
    assert_eq!(nested.exit_code, 0);
}

#[test]
fn ascii_string_offset_writes_reject_uncovered_edges() {
    let empty = runtime_error(
        r#"<?php
$s = "abc";
$s[1] = "";
"#,
    );
    assert_eq!(
        empty.message,
        "invalid array access: cannot assign an empty string to a string offset"
    );

    let string_key = run_source(
        r#"<?php
$s = "abc";
$s["name"] = "X";
"#,
    )
    .unwrap();
    assert!(string_key.stdout.contains(
        "Fatal error: Uncaught TypeError: Cannot access offset of type string on string"
    ));
    assert_eq!(string_key.exit_code, 255);

    let decimal_key = run_source(
        r#"<?php
$s = "abc";
$s["1.0"] = "X";
"#,
    )
    .unwrap();
    assert!(decimal_key.stdout.contains(
        "Fatal error: Uncaught TypeError: Cannot access offset of type string on string"
    ));
    assert_eq!(decimal_key.exit_code, 255);
}
