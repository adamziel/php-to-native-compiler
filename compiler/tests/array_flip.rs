use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_flip_uses_int_string_values_as_keys_and_overwrites_duplicates() {
    let source = r#"<?php
$items = [];
$items["first"] = "name";
$items[5] = "2";
$items["two"] = 2;
$items["02"] = "02";
$items[] = -1;
$items["dup-string"] = "name";

$flipped = array_flip($items);
print_r($flipped);
echo count($flipped), "\n";
echo $flipped["name"], "|", $flipped[2], "|", $flipped["02"], "|", $flipped[-1], "\n";
$flipped[] = "after";
echo $flipped[3], "\n";
print_r($items);

$call = "array_flip";
$again = $call($items);
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => dup-string\n    [2] => two\n    [02] => 02\n    [-1] => 6\n)\n4\ndup-string|two|02|6\nafter\nArray\n(\n    [first] => name\n    [5] => 2\n    [two] => 2\n    [02] => 02\n    [6] => -1\n    [dup-string] => name\n)\ndup-string|two|02|6"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_flip_requires_array_argument() {
    let error = runtime_error("<?php\necho array_flip(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_flip(): argument must be array, got int"
    );
}

#[test]
fn array_flip_rejects_unsupported_value_types() {
    let error = runtime_error("<?php\n$items = [\"ok\", true];\necho array_flip($items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_flip(): values must be int or string in the current subset, got bool"
    );
}

#[test]
fn emit_ir_rejects_array_flip_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_flip([\"name\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
