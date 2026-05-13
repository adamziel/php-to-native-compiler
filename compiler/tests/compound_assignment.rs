use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn direct_variable_compound_assignments_update_scalar_values() {
    let execution = run_source(
        r#"<?php
$value = 10;
$value += 5;
echo $value, "\n";
$value -= 3;
echo $value, "\n";
$value *= "2";
echo $value, "\n";
$value /= 4;
echo $value, "\n";
$text = "php";
$text .= "-native";
echo $text, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "15\n12\n24\n6\nphp-native\n");
}

#[test]
fn for_headers_accept_direct_variable_compound_assignment() {
    let execution = run_source(
        r#"<?php
$sum = 0;
for ($i = 0; $i < 5; $i += 2) {
    $sum += $i;
}
echo $sum, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "6\n");
}

#[test]
fn compound_assignment_reads_left_side_before_writing() {
    let execution = run_source(
        r#"<?php
$value = "a";
function next_value() {
    echo "rhs\n";
    return "b";
}
$value .= next_value();
echo $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "rhs\nab\n");
}

#[test]
fn undefined_compound_assignment_left_side_is_runtime_error() {
    let error = runtime_error("<?php\n$missing += 1;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn compound_assignment_reuses_arithmetic_diagnostics() {
    let error = runtime_error("<?php\n$value = 'abc';\n$value += 1;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid arithmetic for +: string is not numeric"
    );
}

#[test]
fn emit_ir_rejects_compound_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "compound assignment is supported by phpc run for direct static variables but not LLVM IR emission yet"
    );
}
