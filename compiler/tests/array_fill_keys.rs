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
fn array_fill_keys_accepts_null_bool_and_integral_float_key_values() {
    let source = r#"<?php
$keys = [null, false, true, 1.0, 2.0, "2", "02", -3.0];
$filled = array_fill_keys($keys, "x");
print_r($filled);
echo count($filled), "\n";
echo $filled[""], "|", $filled[1], "|", $filled[2], "|", $filled["02"], "|", $filled[-3], "\n";

$call = "array_fill_keys";
$again = $call([0.0, 1.0], "y");
echo count($again), "\n";
echo $again[0], "|", $again[1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [] => x\n    [1] => x\n    [2] => x\n    [02] => x\n    [-3] => x\n)\n5\nx|x|x|x|x\n2\ny|y"
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
fn array_fill_keys_rejects_objects_without_string_conversion() {
    let error = runtime_error(
        "<?php\nclass Plain {}\n$keys = [\"ok\", new Plain()];\necho array_fill_keys($keys, \"value\");\n",
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_fill_keys(): Object of class Plain could not be converted to string"
    );
}

#[test]
fn array_fill_keys_stringifies_array_object_and_resource_key_values() {
    let source = r#"<?php
class classA {
    public function __toString() {
        return "Class A object";
    }
}

$stream = fopen("php://memory", "w+");
$keys = [[1], new classA(), $stream, "simple", false, 2.4];
$filled = array_fill_keys($keys, "value");
print_r($filled);
echo count($filled), "\n";
echo $filled["Array"], "|", $filled["Class A object"], "|", $filled["simple"], "|", $filled[""], "|", $filled["2.4"], "\n";
foreach ($filled as $key => $value) {
    if (str_starts_with($key, "Resource id #")) {
        echo "resource=", $value;
    }
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution
            .stdout
            .matches("Warning: Array to string conversion")
            .count(),
        1
    );
    assert!(execution.stdout.contains("    [Array] => value\n"));
    assert!(execution.stdout.contains("    [Class A object] => value\n"));
    assert!(execution.stdout.contains("    [Resource id #"));
    assert!(execution.stdout.contains("    [simple] => value\n"));
    assert!(execution.stdout.contains("    [] => value\n"));
    assert!(execution.stdout.contains("    [2.4] => value\n"));
    assert!(execution
        .stdout
        .contains("6\nvalue|value|value|value|value\nresource=value"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_fill_keys_reads_reference_backed_key_slots_by_value() {
    let source = r#"<?php
$simple = "simple";
$refValue = &$simple;
$filled = array_fill_keys(["one", "two"], $refValue);
print_r($filled);

$refKeys = ["one", &$simple];
$result = array_fill_keys($refKeys, $simple);
print_r($result);
$simple = "bob";
print_r($result);

$source = ["one", "two"];
$refArray = &$source;
print_r(array_fill_keys($refArray, $simple));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [one] => simple\n    [two] => simple\n)\nArray\n(\n    [one] => simple\n    [simple] => simple\n)\nArray\n(\n    [one] => simple\n    [simple] => simple\n)\nArray\n(\n    [one] => bob\n    [two] => bob\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_fill_keys_stringifies_float_key_values() {
    let source = r#"<?php
$keys = [-0.0, 0.0, 1.0, 1.5, -3.25, INF, -INF, NAN];
$filled = array_fill_keys($keys, "x");
print_r($filled);
echo count($filled), "\n";
echo $filled["-0"], "|", $filled[0], "|", $filled[1], "|", $filled["1.5"], "|", $filled["-3.25"], "|", $filled["INF"], "|", $filled["-INF"], "|", $filled["NAN"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [-0] => x\n    [0] => x\n    [1] => x\n    [1.5] => x\n    [-3.25] => x\n    [INF] => x\n    [-INF] => x\n    [NAN] => x\n)\n8\nx|x|x|x|x|x|x|x"
    );
    assert_eq!(execution.exit_code, 0);
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
