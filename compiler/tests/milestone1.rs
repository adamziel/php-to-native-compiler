use std::path::Path;
use std::process::Command;
use std::thread;

use php_compiler::test_runner::run_fixture_dir;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

#[test]
fn milestone1_fixtures_pass() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
    let summary = thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || run_fixture_dir(&fixture_dir).unwrap())
        .expect("large-stack fixture test thread should spawn")
        .join()
        .expect("large-stack fixture test thread should not panic");
    assert_eq!(summary.failed, 0, "{:#?}", summary.failures);
    assert!(summary.passed >= 13);
}

#[test]
fn run_executes_function_and_loop() {
    let source = r#"<?php
function twice($x) {
    return $x * 2;
}
$i = 1;
while ($i < 4) {
    echo twice($i), ",";
    $i = $i + 1;
}
"#;
    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "2,4,6,");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn run_executes_break_for_innermost_while_loop() {
    let source = r#"<?php
$i = 0;
while ($i < 2) {
    $j = 0;
    while (true) {
        echo $i, ":", $j, ";";
        break;
        echo "unreachable";
    }
    $i = $i + 1;
}
echo "done";
"#;
    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "0:0;1:0;done");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn run_executes_continue_for_innermost_while_loop() {
    let source = r#"<?php
$i = 0;
while ($i < 2) {
    $j = 0;
    while ($j < 3) {
        $j = $j + 1;
        if ($j == 2) {
            continue;
        }
        echo $i, ":", $j, ";";
    }
    echo "outer;";
    $i = $i + 1;
}
echo "done";
"#;
    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "0:1;0:3;outer;1:1;1:3;outer;done");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_integer_division_until_native_numeric_checks_exist() {
    let error = emit_ir_source("<?php\n$x = 6 / 2;\necho $x;\n").unwrap_err();

    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("division lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_unsupported_control_flow() {
    let error = emit_ir_source("<?php\nif (1) { echo 1; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("if/else"));
}

#[test]
fn emit_ir_rejects_elseif_until_native_conditional_lowering_exists() {
    let error =
        emit_ir_source("<?php\nif (false) { echo 0; } elseif (true) { echo 1; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("elseif"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_break_until_native_loop_control_lowering_exists() {
    let error = emit_ir_source("<?php\nbreak;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("break"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_continue_until_native_loop_control_lowering_exists() {
    let error = emit_ir_source("<?php\ncontinue;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("continue"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_foreach_until_native_iteration_lowering_exists() {
    let error = emit_ir_source("<?php\nforeach ([1] as $value) { echo $value; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("foreach"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_foreach_key_value_until_native_iteration_lowering_exists() {
    let error =
        emit_ir_source("<?php\nforeach ([\"name\" => \"Ada\"] as $key => $value) { echo $key; }\n")
            .unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("foreach"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_for_until_native_loop_lowering_exists() {
    let error =
        emit_ir_source("<?php\nfor ($i = 0; $i < 3; $i = $i + 1) { echo $i; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("for loops"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_do_while_until_native_loop_lowering_exists() {
    let error = emit_ir_source("<?php\ndo { echo 1; } while (false);\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("do-while loops"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_switch_until_native_switch_lowering_exists() {
    let error = emit_ir_source("<?php\nswitch (1) { case 1: echo 1; break; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("switch statements"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_alternate_switch_until_native_switch_lowering_exists() {
    let error =
        emit_ir_source("<?php\nswitch (1): case 1: echo 1; break; endswitch;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("switch statements"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_arrays_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho [1];\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("arrays"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_long_arrays_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho array(1);\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("arrays"), "{}", error.message);
}

#[test]
fn emit_ir_rejects_array_indexing_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho $items[0];\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("array indexing"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_array_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$items[0] = 1;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("array assignment"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_array_offset_unset_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nunset($items[0]);\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("array offset unset"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_lowers_direct_variable_unset_through_native_symbol_table() {
    let ir = emit_ir_source("<?php\nunset($value);\n").unwrap();
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_unset"),
        "{ir}"
    );
}

#[test]
fn emit_ir_rejects_multiple_unset_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nunset($value, $items[0]);\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("multiple-operand unset"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_global_declarations_until_scope_imports_exist() {
    let error = emit_ir_source("<?php\nglobal $value;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("global declarations"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_dynamic_function_calls_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$call = \"strlen\";\necho $call(\"abc\");\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("dynamic function-call"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_class_declarations_until_native_metadata_lowering_exists() {
    let error = emit_ir_source("<?php\nclass Box {}\necho 1;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("class declarations"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_instantiation_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box = new Box();\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_property_access_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho $box->name;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("object-property lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_property_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box->name = \"Ada\";\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("object-property lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_asm_through_available_native_toolchain() {
    let has_backend = ["clang", "llc", "cc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok());
    if !has_backend {
        return;
    }

    let asm = emit_asm_source("<?php\necho 3;\n").unwrap();
    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn emit_asm_rejects_dynamic_integer_modulo_until_native_runtime_checks_exist() {
    let has_backend = ["clang", "llc", "cc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok());
    if !has_backend {
        return;
    }

    let error = emit_asm_source("<?php\n$divisor = 4 - 2;\necho 10 % $divisor;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("modulo lowering"),
        "{}",
        error.message
    );
}
