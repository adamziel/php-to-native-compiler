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
fn array_sum_rejects_non_numeric_strings_until_warning_recovery_exists() {
    let error = runtime_error("<?php\n$items = [\"ok\", \"abc\"];\necho array_sum($items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_sum(): values must be numeric in the current subset, got non-numeric string"
    );
}

#[test]
fn array_sum_rejects_non_scalar_values() {
    let error = runtime_error("<?php\n$items = [[]];\necho array_sum($items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_sum(): values must be numeric scalar in the current subset, got array"
    );
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
