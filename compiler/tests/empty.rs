use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn empty_direct_variables_and_array_offsets_match_supported_php_subset() {
    let source = r#"<?php
$null = null;
$false = false;
$true = true;
$zero = 0;
$one = 1;
$empty_string = "";
$zero_string = "0";
$text = "text";
$empty_array = [];
$filled_array = [0];

$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["false"] = false;
$items["zero"] = 0;
$items["empty"] = "";
$items["zero_string"] = "0";
$items["empty_array"] = [];
$items["filled_array"] = [0];
$items["2"] = "two";
$key = "present";

if (empty($missing)) {
    echo "missing:empty\n";
}
if (empty($null)) {
    echo "null:empty\n";
}
if (empty($false)) {
    echo "false:empty\n";
}
if (empty($zero)) {
    echo "zero:empty\n";
}
if (empty($empty_string)) {
    echo "empty-string:empty\n";
}
if (empty($zero_string)) {
    echo "zero-string:empty\n";
}
if (empty($true)) {
    echo "true:empty\n";
} else {
    echo "true:not-empty\n";
}
if (empty($one)) {
    echo "one:empty\n";
} else {
    echo "one:not-empty\n";
}
if (empty($text)) {
    echo "text:empty\n";
} else {
    echo "text:not-empty\n";
}
if (empty($empty_array)) {
    echo "empty-array:empty\n";
}
if (empty($filled_array)) {
    echo "filled-array:empty\n";
} else {
    echo "filled-array:not-empty\n";
}
if (empty($items[$key])) {
    echo "offset-present:empty\n";
} else {
    echo "offset-present:not-empty\n";
}
if (empty($items["null"])) {
    echo "offset-null:empty\n";
}
if (empty($items["false"])) {
    echo "offset-false:empty\n";
}
if (empty($items["zero"])) {
    echo "offset-zero:empty\n";
}
if (empty($items["empty"])) {
    echo "offset-empty-string:empty\n";
}
if (empty($items["zero_string"])) {
    echo "offset-zero-string:empty\n";
}
if (empty($items["missing"])) {
    echo "offset-missing:empty\n";
}
if (empty($missing_array[0])) {
    echo "offset-undefined-array:empty\n";
}
$number = 42;
if (empty($number[0])) {
    echo "offset-scalar-target:empty\n";
}
if (empty($items[2])) {
    echo "offset-int-normalized:empty";
} else {
    echo "offset-int-normalized:not-empty";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "missing:empty\nnull:empty\nfalse:empty\nzero:empty\nempty-string:empty\nzero-string:empty\ntrue:not-empty\none:not-empty\ntext:not-empty\nempty-array:empty\nfilled-array:not-empty\noffset-present:not-empty\noffset-null:empty\noffset-false:empty\noffset-zero:empty\noffset-empty-string:empty\noffset-zero-string:empty\noffset-missing:empty\noffset-undefined-array:empty\noffset-scalar-target:empty\noffset-int-normalized:not-empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn empty_nested_and_object_property_array_offsets_match_reached_subset() {
    let source = r#"<?php
class Bag {
    public $items = [];
}

$items = [["value"], [null], [""]];
$bag = new Bag();
$bag->items = [["value"], [0], [""]];
$key = 0;

echo empty($items[0][$key]) ? "nested-present-empty" : "nested-present-set";
echo "|";
echo empty($items[1][0]) ? "nested-null-empty" : "nested-null-set";
echo "|";
echo empty($items[2][0]) ? "nested-string-empty" : "nested-string-set";
echo "|";
echo empty($items[3][0]) ? "nested-missing-empty" : "nested-missing-set";
echo "|";
echo empty($bag->items[0][$key]) ? "object-present-empty" : "object-present-set";
echo "|";
echo empty($bag->items[1][0]) ? "object-zero-empty" : "object-zero-set";
echo "|";
echo empty($bag->items[2][0]) ? "object-string-empty" : "object-string-set";
echo "|";
echo empty($bag->items[3][0]) ? "object-missing-empty" : "object-missing-set";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "nested-present-set|nested-null-empty|nested-string-empty|nested-missing-empty|object-present-set|object-zero-empty|object-string-empty|object-missing-empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn empty_accepts_direct_call_expression_values() {
    let source = r#"<?php
function blank() {
    return "";
}
function filled() {
    return "value";
}
echo empty(blank()) ? "blank-empty" : "blank-set";
echo "|";
echo empty(filled()) ? "filled-empty" : "filled-set";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "blank-empty|filled-set");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn empty_accepts_non_lvalue_expression_values() {
    let source = r#"<?php
function getEmptyArray() { return []; }
function getNonEmptyArray() { return [1, 2, 3]; }

var_dump(empty([]));
var_dump(empty([1, 2, 3]));
var_dump(empty(getEmptyArray()));
var_dump(empty(getNonEmptyArray()));
var_dump(empty([] + []));
var_dump(empty([1, 2, 3] + []));
var_dump(empty("string"));
var_dump(empty(""));
var_dump(empty(true));
var_dump(empty(false));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn complex_empty_operands_remain_explicitly_unsupported() {
    let error =
        runtime_error("<?php\nfunction items() { return [[1]]; }\necho empty(items()[0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call empty(): only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported"
    );
}
