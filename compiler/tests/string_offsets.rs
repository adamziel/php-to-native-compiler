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

    let string_key = runtime_error(
        r#"<?php
$s = "abc";
$s["name"] = "X";
"#,
    );
    assert_eq!(
        string_key.message,
        "invalid array access: cannot write offset on string"
    );

    let decimal_key = runtime_error(
        r#"<?php
$s = "abc";
$s["1.0"] = "X";
"#,
    );
    assert_eq!(
        decimal_key.message,
        "invalid array access: cannot write offset on string"
    );
}
