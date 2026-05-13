use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn direct_variable_increment_decrement_updates_int_and_float_values() {
    let execution = run_source(
        r#"<?php
$int = 10;
++$int;
echo $int, "\n";
$int++;
echo $int, "\n";
--$int;
echo $int, "\n";
$int--;
echo $int, "\n";

$float = 1.5;
++$float;
echo $float, "\n";
$float--;
echo $float, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11\n12\n11\n10\n2.5\n1.5\n");
}

#[test]
fn undefined_increment_decrement_left_side_is_runtime_error() {
    let error = runtime_error("<?php\n$missing++;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn increment_decrement_rejects_non_numeric_current_gap() {
    let error = runtime_error("<?php\n$value = 'az';\n++$value;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call increment/decrement: only int and float variables are implemented, got string"
    );
}

#[test]
fn emit_ir_rejects_increment_decrement_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\n$value++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "increment/decrement is supported by phpc run for direct static int/float variables but not LLVM IR emission yet"
    );
}
