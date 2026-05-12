use std::path::Path;
use std::process::Command;

use php_compiler::test_runner::run_fixture_dir;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

#[test]
fn milestone1_fixtures_pass() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
    let summary = run_fixture_dir(&fixture_dir).unwrap();
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
fn emit_ir_for_integer_arithmetic() {
    let ir = emit_ir_source("<?php\n$x = 1 + 2;\necho $x;\n").unwrap();
    assert!(ir.contains("add i64 1, 2"), "{ir}");
    assert!(ir.contains("@printf"), "{ir}");
    assert!(ir.contains("define i32 @main()"), "{ir}");
}

#[test]
fn emit_ir_rejects_unsupported_control_flow() {
    let error = emit_ir_source("<?php\nif (1) { echo 1; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("if/else"));
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
fn emit_ir_rejects_variable_unset_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nunset($value);\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("variable unset"),
        "{}",
        error.message
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
        error.message.contains("function calls"),
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
        error.message.contains("object instantiation"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_property_access_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho $box->name;\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("object property access"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_property_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box->name = \"Ada\";\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(
        error.message.contains("object property assignment"),
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

    let asm = emit_asm_source("<?php\necho 1 + 2;\n").unwrap();
    assert!(asm.contains("main"), "{asm}");
}
