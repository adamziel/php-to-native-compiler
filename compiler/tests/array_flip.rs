use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

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
    let execution = run_source(
        r#"<?php
try {
    array_flip(42);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
$call = "array_flip";
try {
    $call(new stdClass());
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array_flip(): Argument #1 ($array) must be of type array, int given\narray_flip(): Argument #1 ($array) must be of type array, stdClass given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_flip_warns_and_skips_unsupported_value_types() {
    let execution = run_source(
        r#"<?php
$items = ["ok" => "name", "skip_bool" => true, "skip_null" => null, "keep" => 7];
$flipped = array_flip($items);
print_r($flipped);
"#,
    )
    .unwrap();

    assert_eq!(
        execution
            .stdout
            .matches(
                "Warning: array_flip(): Can only flip string and integer values, entry skipped"
            )
            .count(),
        2
    );
    assert!(execution.stdout.contains("    [name] => ok"));
    assert!(execution.stdout.contains("    [7] => keep"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
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
