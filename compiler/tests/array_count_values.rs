use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_count_values_counts_int_string_values_and_normalizes_keys() {
    let source = r#"<?php
$items = [];
$items["first"] = "name";
$items[5] = "2";
$items["two"] = 2;
$items["02"] = "02";
$items[] = -1;
$items["dup-string"] = "name";
$items["dup-int"] = 2;

$counted = array_count_values($items);
print_r($counted);
echo count($counted), "\n";
echo $counted["name"], "|", $counted[2], "|", $counted["02"], "|", $counted[-1], "\n";
$counted[] = "after";
echo $counted[3], "\n";
print_r($items);

$call = "array_count_values";
$again = $call($items);
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => 2\n    [2] => 3\n    [02] => 1\n    [-1] => 1\n)\n4\n2|3|1|1\nafter\nArray\n(\n    [first] => name\n    [5] => 2\n    [two] => 2\n    [02] => 02\n    [6] => -1\n    [dup-string] => name\n    [dup-int] => 2\n)\n2|3|1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_count_values_non_arrays_raise_catchable_type_errors() {
    let source = r#"<?php
function check($label, $value) {
    try {
        var_dump(array_count_values($value));
    } catch (TypeError $e) {
        echo $label, ": ", $e->getMessage(), "\n";
    }
}

check("null", null);
check("int", 42);
check("float", 1.25);
check("string", "items");
check("object", new stdClass());
check("true", true);
check("false", false);

$call = "array_count_values";
try {
    $call(42);
} catch (TypeError $e) {
    echo "dynamic: ", $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "null: array_count_values(): Argument #1 ($array) must be of type array, null given\n\
int: array_count_values(): Argument #1 ($array) must be of type array, int given\n\
float: array_count_values(): Argument #1 ($array) must be of type array, float given\n\
string: array_count_values(): Argument #1 ($array) must be of type array, string given\n\
object: array_count_values(): Argument #1 ($array) must be of type array, stdClass given\n\
true: array_count_values(): Argument #1 ($array) must be of type array, true given\n\
false: array_count_values(): Argument #1 ($array) must be of type array, false given\n\
dynamic: array_count_values(): Argument #1 ($array) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_count_values_warns_and_skips_unsupported_value_types() {
    let execution = run_source(
        r#"<?php
$items = ["ok", true, false, null, []];
print_r(array_count_values($items));
$quiet = @array_count_values([[], "quiet", null]);
print_r($quiet);
"#,
    )
    .unwrap();

    assert_eq!(
        execution
            .stdout
            .matches(
                "array_count_values(): Can only count string and integer values, entry skipped"
            )
            .count(),
        4
    );
    assert!(execution.stdout.contains("    [ok] => 1\n"));
    assert!(execution.stdout.contains("    [quiet] => 1\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_count_values_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_count_values([\"name\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
