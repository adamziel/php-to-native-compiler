use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_product_accumulates_supported_scalar_values() {
    let source = r#"<?php
$integers = [true, 2, " 4 ", "-3"];
echo array_product($integers), "\n";

$with_zero = [null, true, 2];
echo array_product($with_zero), "\n";

$mixed = [];
$mixed["int"] = 2;
$mixed["float"] = 3.5;
$mixed["exponent"] = "6e1";
$mixed["decimal"] = ".25";
echo array_product($mixed), "\n";

$empty = [];
echo array_product($empty), "\n";
echo $mixed["exponent"], "|", $mixed["decimal"], "\n";

$call = "array_product";
echo $call($mixed);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "-24\n0\n105\n1\n6e1|.25\n105");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_consumes_64_bit_unsigned_sprintf_values() {
    let execution = run_source(
        r#"<?php
var_dump(array_product([2, sprintf("%u", -1)]));
var_dump(array_product([8.993, 7443241, 988, sprintf("%u", -1) + 0.44]));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "float(3.6893488147419103E+19)\nfloat(1.219953680144986E+30)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_requires_array_argument() {
    let error = runtime_error("<?php\necho array_product(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_product(): argument must be array, got int"
    );
}

#[test]
fn array_product_warns_for_non_numeric_strings() {
    let execution =
        run_source("<?php\n$items = [\"ok\", \"abc\"];\necho array_product($items);\n").unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: array_product(): Multiplication is not supported on type string in Command line code on line 3\n\nWarning: array_product(): Multiplication is not supported on type string in Command line code on line 3\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_warns_for_non_scalar_values() {
    let execution = run_source("<?php\n$items = [[]];\necho array_product($items);\n").unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: array_product(): Multiplication is not supported on type array in Command line code on line 3\n1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_product_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_product([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
