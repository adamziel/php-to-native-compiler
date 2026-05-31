use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_sum_accumulates_supported_scalar_values() {
    let source = r#"<?php
$integers = [null, false, true, 2, " 4 ", "-3"];
echo array_sum($integers), "\n";

$mixed = [];
$mixed["int"] = 2;
$mixed["float"] = 3.5;
$mixed["exponent"] = "6e1";
$mixed["decimal"] = ".25";
echo array_sum($mixed), "\n";

$empty = [];
echo array_sum($empty), "\n";
echo $mixed["exponent"], "|", $mixed["decimal"], "\n";

$call = "array_sum";
echo $call($mixed);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "4\n65.75\n0\n6e1|.25\n65.75");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_sum_requires_array_argument() {
    let error = runtime_error("<?php\necho array_sum(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_sum(): argument must be array, got int"
    );
}

#[test]
fn array_sum_treats_non_numeric_strings_as_zero() {
    let execution =
        run_source("<?php\n$items = [\"ok\", \"abc\"];\necho array_sum($items);\n").unwrap();

    assert_eq!(
        execution
            .stdout
            .matches("Warning: array_sum(): Addition is not supported on type string")
            .count(),
        2
    );
    assert!(execution.stdout.ends_with("\n0"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_sum_warns_for_non_scalar_values() {
    let execution = run_source("<?php\n$items = [[]];\necho array_sum($items);\n").unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: array_sum(): Addition is not supported on type array in Command line code on line 3\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_sum_warns_for_resources_and_reduce_catches_resource_addition_type_error() {
    let source = r#"<?php
$input = [10, STDERR];

echo "array_sum() version:\n";
var_dump(array_sum($input));

echo "array_reduce() version:\n";
try {
    var_dump(array_reduce($input, fn($carry, $value) => $carry + $value, 0));
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        "array_sum() version:\n\nWarning: array_sum(): Addition is not supported on type resource in Command line code on line 5\nint(13)\narray_reduce() version:\nUnsupported operand types: int + resource"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_sum_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_sum([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
