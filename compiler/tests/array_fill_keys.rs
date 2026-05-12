use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_fill_keys_uses_int_string_key_values_and_overwrites_duplicates() {
    let source = r#"<?php
$keys = [];
$keys["first"] = "name";
$keys[5] = "2";
$keys["two"] = 2;
$keys["02"] = "02";
$keys[] = -1;
$keys["dup-string"] = "name";

$filled = array_fill_keys($keys, "value");
print_r($filled);
echo count($filled), "\n";
echo $filled["name"], "|", $filled[2], "|", $filled["02"], "|", $filled[-1], "\n";
$filled[] = "after";
echo $filled[3], "\n";
print_r($keys);

$call = "array_fill_keys";
$again = $call($keys, "again");
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => value\n    [2] => value\n    [02] => value\n    [-1] => value\n)\n4\nvalue|value|value|value\nafter\nArray\n(\n    [first] => name\n    [5] => 2\n    [two] => 2\n    [02] => 02\n    [6] => -1\n    [dup-string] => name\n)\nagain|again|again|again"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_fill_keys_requires_array_keys_argument() {
    let error = runtime_error("<?php\necho array_fill_keys(42, \"value\");\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_fill_keys(): first argument must be array, got int"
    );
}

#[test]
fn array_fill_keys_rejects_unsupported_key_value_types() {
    let error =
        runtime_error("<?php\n$keys = [\"ok\", true];\necho array_fill_keys($keys, \"value\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_fill_keys(): key values must be int or string in the current subset, got bool"
    );
}

#[test]
fn emit_ir_rejects_array_fill_keys_until_native_call_lowering_exists() {
    let error =
        emit_ir_source("<?php\necho array_fill_keys([\"name\"], \"value\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
