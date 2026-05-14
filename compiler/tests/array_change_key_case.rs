use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_change_key_case_changes_ascii_string_keys_and_preserves_integer_keys() {
    let source = r#"<?php
$items = [];
$items["Name"] = "Ada";
$items["name"] = "lower";
$items[7] = "seven";
$items["MiXeD"] = "mixed";
$items["02"] = "numeric string";

$lower = array_change_key_case($items);
print_r($lower);
$upper = array_change_key_case($items, CASE_UPPER);
print_r($upper);
echo $lower["name"], "|", $lower[7], "|", $lower["mixed"], "|", $lower["02"], "\n";
echo $upper["NAME"], "|", $upper[7], "|", $upper["MIXED"], "|", $upper["02"], "\n";
$lower[] = "after";
echo $lower[8], "\n";
print_r($items);

$call = "array_change_key_case";
$again = $call($items, CASE_LOWER);
echo $again["name"], "|", constant("CASE_UPPER"), "|", defined("CASE_LOWER");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => lower\n    [7] => seven\n    [mixed] => mixed\n    [02] => numeric string\n)\nArray\n(\n    [NAME] => lower\n    [7] => seven\n    [MIXED] => mixed\n    [02] => numeric string\n)\nlower|seven|mixed|numeric string\nlower|seven|mixed|numeric string\nafter\nArray\n(\n    [Name] => Ada\n    [name] => lower\n    [7] => seven\n    [MiXeD] => mixed\n    [02] => numeric string\n)\nlower|1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_change_key_case_requires_array_argument() {
    let error = runtime_error("<?php\necho array_change_key_case(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_change_key_case(): argument must be array, got int"
    );
}

#[test]
fn array_change_key_case_rejects_non_int_case_flag() {
    let error = runtime_error(
        "<?php\n$items = [\"Name\" => \"Ada\"];\necho array_change_key_case($items, true);\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_change_key_case(): case flag must be int in the current subset, got bool"
    );
}

#[test]
fn array_change_key_case_treats_nonzero_int_case_flags_as_uppercase() {
    let source = r#"<?php
$items = ["Name" => "Ada", "name" => "lower", "MiXeD" => "mixed"];
$positive = array_change_key_case($items, 2);
$negative = array_change_key_case($items, -1);
echo $positive["NAME"], "|", $positive["MIXED"], "\n";
echo $negative["NAME"], "|", $negative["MIXED"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "lower|mixed\nlower|mixed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_change_key_case_until_native_call_lowering_exists() {
    let error =
        emit_ir_source("<?php\necho array_change_key_case([\"Name\" => \"Ada\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("array lowering rejects arrays"),
        "{}",
        error.message
    );
}
