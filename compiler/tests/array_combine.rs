use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_combine_uses_int_string_key_values_and_overwrites_duplicates() {
    let source = r#"<?php
$keys = [];
$keys["first"] = "name";
$keys[5] = "2";
$keys["two"] = 2;
$keys["02"] = "02";
$keys[] = -1;
$keys["dup-string"] = "name";

$values = [];
$values["a"] = "Ada";
$values[10] = "two string";
$values[] = "two int";
$values["d"] = "zero two";
$values[-3] = "negative";
$values[] = "duplicate";

$combined = array_combine($keys, $values);
print_r($combined);
echo count($combined), "\n";
echo $combined["name"], "|", $combined[2], "|", $combined["02"], "|", $combined[-1], "\n";
$combined[] = "after";
echo $combined[3], "\n";
print_r($keys);
print_r($values);

$call = "array_combine";
$again = $call($keys, $values);
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1], "\n";

$empty = array_combine([], []);
print_r($empty);
echo count($empty);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => duplicate\n    [2] => two int\n    [02] => zero two\n    [-1] => negative\n)\n4\nduplicate|two int|zero two|negative\nafter\nArray\n(\n    [first] => name\n    [5] => 2\n    [two] => 2\n    [02] => 02\n    [6] => -1\n    [dup-string] => name\n)\nArray\n(\n    [a] => Ada\n    [10] => two string\n    [11] => two int\n    [d] => zero two\n    [-3] => negative\n    [12] => duplicate\n)\nduplicate|two int|zero two|negative\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_combine_accepts_null_and_bool_key_value_coercions() {
    let source = r#"<?php
$keys = [null, false, true, "01"];
$values = ["null key", "false key", "true key", "string one"];
$combined = array_combine($keys, $values);
print_r($combined);
echo count($combined), "\n";
echo $combined[""], "|", $combined[1], "|", $combined["01"], "\n";

$call = "array_combine";
$again = $call($keys, $values);
echo $again[""], "|", $again[1], "|", $again["01"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [] => false key\n    [1] => true key\n    [01] => string one\n)\n3\nfalse key|true key|string one\nfalse key|true key|string one"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_combine_accepts_float_object_resource_key_value_coercions() {
    let source = r#"<?php
class classA {
    public function __toString() {
        return "Class A object";
    }
}

$fp = fopen("php://memory", "w+");
$keys = [-0.0, 0.0, 1.0, 1.5, 2.25, new classA(), $fp, true, false, null, "04"];
$values = ["negative zero", "zero", "one", "one point five", "two point two five", "object", "resource", "true key", "false key", "null key", "leading"];
$combined = array_combine($keys, $values);
print_r($combined);
echo count($combined), "\n";
echo $combined["-0"], "|", $combined[0], "|", $combined[1], "|", $combined["1.5"], "|", $combined["2.25"], "|", $combined["Class A object"], "|", $combined["Resource id #5"], "|", $combined[""], "|", $combined["04"], "\n";

$call = "array_combine";
$again = $call([-0.0, 0.0, 1.25], ["negative zero", "zero", "fraction"]);
echo $again["-0"], "|", $again[0], "|", $again["1.25"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [-0] => negative zero\n    [0] => zero\n    [1] => true key\n    [1.5] => one point five\n    [2.25] => two point two five\n    [Class A object] => object\n    [Resource id #5] => resource\n    [] => null key\n    [04] => leading\n)\n9\nnegative zero|zero|true key|one point five|two point two five|object|resource|null key|leading\nnegative zero|zero|fraction"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_combine_requires_array_first_argument() {
    let error = runtime_error("<?php\n$values = [];\necho array_combine(42, $values);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_combine(): first argument must be array, got int"
    );
}

#[test]
fn array_combine_requires_array_second_argument() {
    let error = runtime_error("<?php\n$keys = [];\necho array_combine($keys, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_combine(): second argument must be array, got int"
    );
}

#[test]
fn array_combine_uncaught_length_mismatches_report_value_error() {
    let execution = run_source(
        "<?php\n$keys = [\"one\", \"two\"];\n$values = [1];\necho array_combine($keys, $values);\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught ValueError: array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements in Command line code:4\nStack trace:\n#0 {main}\n  thrown in Command line code on line 4"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn array_combine_length_mismatches_are_catchable_value_errors() {
    let source = r#"<?php
try {
    var_dump(array_combine([], [1]));
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_combine_rejects_unsupported_key_value_types() {
    let error = runtime_error(
        "<?php\n$keys = [\"ok\", []];\n$values = [\"yes\", \"no\"];\necho array_combine($keys, $values);\n",
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_combine(): key values must be null, bool, int, float, string, binary string, resource, or object with __toString() in the current subset, got array"
    );
}

#[test]
fn emit_ir_rejects_array_combine_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_combine([\"name\"], [\"Ada\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
