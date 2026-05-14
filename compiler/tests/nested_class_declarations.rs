use std::fs;

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source, run_source_with_source_file};

const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, goto labels, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, goto jumps, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";

#[test]
fn nested_class_declarations_register_when_executed() {
    let execution = run_source(
        r#"<?php
if (true) {
    class ConditionalBox {
        public $value;

        public function label() {
            return "box:" . $this->value;
        }
    }
}

$box = new ConditionalBox();
$box->value = 7;
echo class_exists("ConditionalBox"), "\n";
echo $box->label(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\nbox:7\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn skipped_nested_class_declarations_do_not_register() {
    let execution = run_source(
        r#"<?php
if (false) {
    class SkippedBox {}
}

echo class_exists("SkippedBox"), "\n";
echo "after\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "\nafter\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_local_class_declarations_register_when_function_runs() {
    let execution = run_source(
        r#"<?php
function declare_box() {
    class FunctionBox {
        public static function label() {
            return "function-box";
        }
    }
}

echo class_exists("FunctionBox"), "\n";
declare_box();
echo class_exists("FunctionBox"), "\n";
echo FunctionBox::label(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "\n1\nfunction-box\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn required_files_can_execute_guarded_nested_class_declarations() {
    let root = std::env::temp_dir().join(format!(
        "phpc-nested-class-{}-{}",
        std::process::id(),
        "require"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create nested class require fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
if (!class_exists("NestedFromRequire", false)) {
    class NestedFromRequire {
        public static function label() {
            return "required";
        }
    }
}
"#,
    )
    .expect("write required nested class fixture");

    let source = r#"<?php
require 'lib.php';
require 'lib.php';
echo NestedFromRequire::label(), "\n";
"#;
    fs::write(&main, source).expect("write main nested class fixture");

    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "required\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn repeated_nested_class_declarations_report_duplicate_class() {
    let error = run_source(
        r#"<?php
if (true) {
    class RepeatedBox {}
}

if (true) {
    class RepeatedBox {}
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 5);
    assert_eq!(error.message, "class RepeatedBox is already defined");
}

#[test]
fn unbraced_nested_class_declarations_remain_unsupported() {
    let error = run_source(
        r#"<?php
if (true)
    class UnbracedNestedBox {}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 5);
    assert_eq!(
        error.message,
        "unsupported unbraced nested class declaration: nested class declarations require a braced statement body in the current subset"
    );
}

#[test]
fn emit_ir_rejects_conditional_nested_class_without_native_execution_claim() {
    let error = emit_ir_source(
        r#"<?php
if (true) {
    class ConditionalBox {}
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
}

#[test]
fn emit_asm_rejects_conditional_nested_class_without_backend_execution() {
    let error = emit_asm_source(
        r#"<?php
if (true) {
    class ConditionalBox {}
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
}

#[test]
fn emit_ir_rejects_function_local_nested_class_without_native_function_lowering() {
    let error = emit_ir_source(
        r#"<?php
function declare_box() {
    class FunctionBox {}
}
declare_box();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn emit_asm_rejects_function_local_nested_class_without_backend_execution() {
    let error = emit_asm_source(
        r#"<?php
function declare_box() {
    class FunctionBox {}
}
declare_box();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}
