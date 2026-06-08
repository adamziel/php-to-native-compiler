use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn in_array_uses_loose_scalar_comparison_in_insertion_order() {
    let source = r#"<?php
$items = [];
$items[] = null;
$items[] = false;
$items[] = 10;
$items[] = "10.0";
$items[] = "abc";

if (in_array("", $items)) {
    echo "empty-matches-null\n";
}
if (in_array("0", $items)) {
    echo "zero-matches-false\n";
}
if (in_array("10", $items)) {
    echo "numeric-string-matches-int\n";
}
if (in_array(10.0, $items)) {
    echo "float-matches-int\n";
}
if (in_array("abc", $items)) {
    echo "string-match\n";
}
if (in_array(11, $items)) {
    echo "unexpected-int\n";
} else {
    echo "missing-int\n";
}
if (in_array("missing", $items)) {
    echo "unexpected-string\n";
} else {
    echo "missing-string\n";
}

$call = "in_array";
if ($call("abc", $items)) {
    echo "dynamic-match";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "empty-matches-null\nzero-matches-false\nnumeric-string-matches-int\nfloat-matches-int\nstring-match\nmissing-int\nmissing-string\ndynamic-match"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_requires_array_second_argument() {
    let source = r#"<?php
try {
    echo in_array("name", 42);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "in_array(): Argument #2 ($haystack) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_strict_mode_uses_scalar_identity() {
    let source = r#"<?php
$items = [];
$items[] = false;
$items[] = 0;
$items[] = "0";
$items[] = 10;
$items[] = "10";
$items[] = null;
$items[] = "abc";

if (in_array("", $items, true)) {
    echo "unexpected-empty\n";
} else {
    echo "empty-missing\n";
}
if (in_array(false, $items, true)) {
    echo "false-match\n";
}
if (in_array(0, $items, true)) {
    echo "int-zero-match\n";
}
if (in_array("0", $items, true)) {
    echo "string-zero-match\n";
}
if (in_array(10.0, $items, true)) {
    echo "unexpected-float\n";
} else {
    echo "float-missing\n";
}
if (in_array(10, $items, true)) {
    echo "int-ten-match\n";
}
if (in_array("10", $items, true)) {
    echo "string-ten-match\n";
}
if (in_array(null, $items, true)) {
    echo "null-match\n";
}
if (in_array("missing", $items, true)) {
    echo "unexpected-missing\n";
} else {
    echo "string-missing\n";
}
if (in_array("10.0", $items, false)) {
    echo "false-flag-uses-loose\n";
}

$call = "in_array";
if ($call("abc", $items, true)) {
    echo "dynamic-strict-match";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "empty-missing\nfalse-match\nint-zero-match\nstring-zero-match\nfloat-missing\nint-ten-match\nstring-ten-match\nnull-match\nstring-missing\nfalse-flag-uses-loose\ndynamic-strict-match"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_strict_mode_uses_array_and_object_identity() {
    let source = r#"<?php
class Box {}

$box = new Box();
$other = new Box();
$items = [];
$items["array"] = ["value" => 1];
$items["object"] = $box;

if (in_array(["value" => 1], $items, true)) {
    echo "array-match\n";
}
if (!in_array(["value" => "1"], $items, true)) {
    echo "array-missing\n";
}
if (in_array($box, $items, true)) {
    echo "object-match\n";
}
if (!in_array($other, $items, true)) {
    echo "object-missing";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array-match\narray-missing\nobject-match\nobject-missing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_coerces_scalar_strict_mode_argument() {
    let source = r#"<?php
$items = [0, false, true, "x"];

foreach ([true, 1, 2, "yes"] as $strict) {
    var_dump(in_array("0", $items, $strict));
}
foreach ([false, 0, "", "0"] as $loose) {
    var_dump(in_array("0", $items, $loose));
}

$call = "in_array";
var_dump($call("0", $items, "1"));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_rejects_non_coercible_strict_mode_argument() {
    let source = r#"<?php
$items = [1];
try {
    var_dump(in_array(1, $items, []));
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "in_array(): Argument #3 ($strict) must be of type bool, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_uses_loose_array_and_object_membership() {
    let source = r#"<?php
class Box {}
$box = new Box();
$items = [[], ["value" => 1], $box, true];

var_dump(in_array(null, $items));
var_dump(in_array([], $items));
var_dump(in_array(["value" => "1"], $items));
var_dump(in_array($box, $items));
var_dump(in_array(new Box(), $items, true));
var_dump(in_array(new Box(), $items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn in_array_uses_singleton_unbacked_enum_cases() {
    let source = r#"<?php
enum Sample { case A; case B; }
$items = [Sample::A];

var_dump(in_array(Sample::A, $items, true));
var_dump(in_array(Sample::B, $items, true));
var_dump(in_array(Sample::A, $items));
var_dump(in_array(Sample::B, $items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_in_array_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho in_array(1, [1], true);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
