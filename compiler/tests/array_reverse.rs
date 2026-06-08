use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_reverse_reverses_order_reindexes_int_keys_and_preserves_string_keys() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$reversed = array_reverse($items);
echo count($reversed), "\n";
echo $reversed[0], "|", $reversed[1], "|", $reversed["02"], "|", $reversed[2], "|", $reversed[3], "|", $reversed["name"], "\n";
$reversed[] = "after";
echo $reversed[4], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$call = "array_reverse";
$again = $call($items);
echo $again[0], "|", $again["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "6\nnext|negative|zero two|two updated|five|Ada\nafter\nAda|five|two updated|zero two|negative|next\nnext|Ada"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reverse_can_preserve_integer_and_string_keys() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$preserved = array_reverse($items, true);
echo count($preserved), "\n";
echo $preserved[6], "|", $preserved[-1], "|", $preserved["02"], "|", $preserved[2], "|", $preserved[5], "|", $preserved["name"], "\n";
$preserved[] = "after";
echo $preserved[7], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$default = array_reverse($items, false);
echo $default[0], "|", $default["name"], "\n";

$call = "array_reverse";
$again = $call($items, true);
echo $again[6], "|", $again["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "6\nnext|negative|zero two|two updated|five|Ada\nafter\nAda|five|two updated|zero two|negative|next\nnext|Ada\nnext|Ada"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reverse_coerces_scalar_preserve_keys_and_reports_bool_type_errors() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$truthy_int = array_reverse($items, 1);
echo $truthy_int[6], "|", $truthy_int[-1], "|", $truthy_int["name"], "\n";

$falsey_int = array_reverse($items, 0);
echo $falsey_int[0], "|", $falsey_int[1], "|", $falsey_int["name"], "\n";

$truthy_string = array_reverse($items, "1");
echo $truthy_string[6], "|", $truthy_string[-1], "|", $truthy_string["name"], "\n";

$falsey_string = array_reverse($items, "0");
echo $falsey_string[0], "|", $falsey_string[1], "|", $falsey_string["name"], "\n";

$falsey_null = array_reverse($items, null);
echo $falsey_null[0], "|", $falsey_null[1], "|", $falsey_null["name"], "\n";

$truthy_float = array_reverse($items, 2.5);
echo $truthy_float[6], "|", $truthy_float[-1], "|", $truthy_float["name"], "\n";

$call = "array_reverse";
$dynamic = $call($items, "yes");
echo $dynamic[6], "|", $dynamic[-1], "|", $dynamic["name"], "\n";

try {
    array_reverse($items, []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "next|negative|Ada\nnext|negative|Ada\nnext|negative|Ada\nnext|negative|Ada\nnext|negative|Ada\nnext|negative|Ada\nnext|negative|Ada\narray_reverse(): Argument #2 ($preserve_keys) must be of type bool, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reverse_requires_array_argument() {
    let error = runtime_error("<?php\necho array_reverse(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reverse(): argument must be array, got int"
    );
}

#[test]
fn array_reverse_reports_bool_type_error_for_invalid_preserve_keys_argument() {
    let source = r#"<?php
$items = [1];

try {
    array_reverse($items, []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_reverse(): Argument #2 ($preserve_keys) must be of type bool, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_reverse_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_reverse([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
