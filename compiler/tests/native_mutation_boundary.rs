use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, non-direct assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_REFERENCE_ASSIGNMENT_REJECTION: &str = "LLVM reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";

#[test]
fn phpc_run_still_handles_current_mutation_subset() {
    let execution = run_source(
        r#"<?php
$value = 1;
$value += 4;
echo $value, "\n";
$value ??= 99;
echo $value, "\n";
$missing ??= "created";
echo $missing, "\n";
echo ($assigned = "expr"), ":", $assigned, "\n";
echo ($value *= 2), ":", $value, "\n";
echo $value++, ":", $value, "\n";
unset($assigned, $missing);
if (isset($assigned)) {
    echo "assigned\n";
} else {
    echo "unset\n";
}
if (isset($missing)) {
    echo "missing\n";
} else {
    echo "unset-missing";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5\n5\ncreated\nexpr:expr\n10:10\n10:11\nunset\nunset-missing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn phpc_run_still_handles_current_reference_assignment_subset() {
    let execution = run_source(
        r#"<?php
class MagicBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$value = "start";
$alias =& $value;
$alias = "direct";
echo $value, "\n";

$items = ["slot" => "array"];
$arrayAlias =& $items["slot"];
$arrayAlias = "array-updated";
echo $items["slot"], "\n";

$storage = "magic";
$box = new MagicBox();
$magicAlias =& $box->missing;
$magicAlias = "magic-updated";
echo $storage;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "direct\narray-updated\nmagic-updated");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_mutation_forms_with_specific_boundary() {
    for source in [
        "<?php\n$value = 1;\n$value += 2;\n",
        "<?php\n$value = null;\n$value ??= 2;\n",
        "<?php\n$value = 1;\n$value++;\n",
        "<?php\n$value = 1;\necho ($value += 2);\n",
        "<?php\n$value = null;\necho ($value ??= 2);\n",
        "<?php\n$value = 1;\necho ++$value;\n",
        "<?php\n$value = 1;\nunset($value);\n",
        "<?php\nunset(Box::$cache);\n",
        "<?php\n$left = 1;\n$right = 2;\nunset($left, $right);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_MUTATION_REJECTION);
    }
}

#[test]
fn emit_ir_lowers_direct_variable_assignment_expression_values() {
    let ir = emit_ir_source(
        "<?php\n$left = 1;\necho ($left = 2), $left;\n$right = (($middle = 3) + 4);\necho $middle, $right;\n$text = \"old\";\necho ($text = \"new\"), $text;\n$flag = false;\necho ($flag = true);\n",
    )
    .expect("direct variable assignment expressions should lower for primitive value families");

    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 2)")
            && ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 3)")
            && ir.contains(" = add i64 3, 4")
            && ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %"),
        "integer assignment-expression values and later reads should be emitted:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @")
            && ir.matches("new\\00").count() >= 1,
        "string assignment-expression values should be emitted through ordinary string output:\n{ir}"
    );
    assert!(
        ir.contains("c\"1\\00\"")
            && ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @"),
        "bool assignment-expression values should remain available to echo:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_MUTATION_REJECTION),
        "direct variable assignment expressions should not fall through the mutation blocker:\n{ir}"
    );
}

#[test]
fn emit_ir_rejects_reference_assignment_forms_with_specific_boundary() {
    for source in [
        "<?php\n$a = 1;\n$b = 2;\n$a =& $b;\n",
        "<?php\n$alias =& $items[0];\n",
        "<?php\n$alias =& $items[];\n",
        "<?php\n$alias =& $box->items[0];\n",
        "<?php\n$property = \"items\";\n$alias =& $box->$property;\n",
        "<?php\n$alias =& $box->missing;\n",
        "<?php\n$alias =& identity($value);\n",
        "<?php\n$alias =& $box->identity($value);\n",
        "<?php\n$alias =& Box::identity($value);\n",
        "<?php\n$alias =& self::identity($value);\n",
        "<?php\n$class = \"Box\";\n$alias =& $class::identity($value);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REFERENCE_ASSIGNMENT_REJECTION);
    }
}

#[test]
fn emit_ir_routes_reference_assignment_source_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$alias =& $items[missing_call()];\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& $box->items[missing_call()];\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& identity(missing_call());\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_statement_mutation_rhs_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$value = 1;\n$value += missing_call();\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = null;\n$value ??= missing_call();\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho ($value = missing_call());\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho ($value += missing_call());\n",
            LLVM_MUTATION_REJECTION,
        ),
        (
            "<?php\necho ($value ??= missing_call());\n",
            LLVM_MUTATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_asm_rejects_reference_assignment_before_backend_execution() {
    let error = emit_asm_source("<?php\n$alias =& $box->missing;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_REFERENCE_ASSIGNMENT_REJECTION);
}

#[test]
fn emit_asm_rejects_mutation_before_backend_execution() {
    let error = emit_asm_source("<?php\n$value = 1;\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn native_reference_assignment_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1096/native_reference_assignment_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1096/native_reference_assignment_boundary_emit_ir.cli"),
    )
    .expect("native reference-assignment CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_mutation_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone176/native_mutation_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone176/native_mutation_boundary_emit_ir.cli"),
    )
    .expect("native mutation CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
