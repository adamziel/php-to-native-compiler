use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment outside lowerable direct variables, null coalescing assignment, increment/decrement, non-direct assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_REFERENCE_ASSIGNMENT_REJECTION: &str = "LLVM reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";
const LLVM_REFERENCE_WRITE_THROUGH_REJECTION: &str = "LLVM reference write-through lowering rejects direct root-variable assignment after reference binding until statement assignment and assignment-expression write-through share an alias-aware reference slot boundary with copy-on-write, cleanup ownership, and exact native error behavior; phpc run handles current reference write-through behavior";
const LLVM_NATIVE_ARRAY_NON_LOCAL_UNSET_REJECTION: &str = "LLVM native array non-local unset lowering rejects object, dynamic-object, non-direct object, and static property unsets until non-local owner cells, magic __unset dispatch, typed/static property state, references/copy-on-write, and exact diagnostics share one unset owner contract; local variables and native array offset unsets use their shared native lvalue unset contracts";

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
    for (source, expected) in [
        (
            "<?php\n$value = null;\n$value ??= 2;\n",
            LLVM_MUTATION_REJECTION,
        ),
        ("<?php\n$value = 1;\n$value++;\n", LLVM_MUTATION_REJECTION),
        (
            "<?php\n$value = null;\necho ($value ??= 2);\n",
            LLVM_MUTATION_REJECTION,
        ),
        (
            "<?php\n$value = 1;\necho ++$value;\n",
            LLVM_MUTATION_REJECTION,
        ),
        (
            "<?php\n$value = 1;\nunset($value);\n",
            LLVM_MUTATION_REJECTION,
        ),
        (
            "<?php\nunset(Box::$cache);\n",
            LLVM_NATIVE_ARRAY_NON_LOCAL_UNSET_REJECTION,
        ),
        (
            "<?php\n$left = 1;\n$right = 2;\nunset($left, $right);\n",
            LLVM_MUTATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_lowers_direct_variable_compound_assignment_values() {
    let ir = emit_ir_source(
        "<?php\n$value = 4;\n$delta = 5;\n$value += $delta;\necho $value;\necho ($value *= 2), $value;\n",
    )
    .expect("direct variable compound assignments should lower for primitive value families");

    assert!(
        ir.contains(" = add i64 4, 5") && ir.matches("i64 18").count() >= 2,
        "direct variable compound assignment statements and expressions should reuse primitive binary lowering:\n{ir}"
    );
    assert!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_value_from_scalar")
            .count()
            >= 3
            && ir
                .matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
                .count()
                >= 3,
        "compound assignment expression results and later reads should remain available to native-value echo output, even when known-value tracking folds the expression result:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_MUTATION_REJECTION),
        "direct variable compound assignments should not fall through the mutation blocker:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_direct_variable_assignment_expression_values() {
    let ir = emit_ir_source(
        "<?php\n$left = 1;\necho ($left = 2), $left;\n$right = (($middle = 3) + 4);\necho $middle, $right;\n$text = \"old\";\necho ($text = \"new\"), $text;\n$flag = false;\necho ($flag = true);\n",
    )
    .expect("direct variable assignment expressions should lower for primitive value families");

    assert!(
        ir.contains("call %phpc.NativeScalarValue @phpc_native_int(i64 2)")
            && ir.contains("call %phpc.NativeScalarValue @phpc_native_int(i64 3)")
            && ir.contains(" = add i64 3, 4")
            && ir.contains("call %phpc.NativeScalarValue @phpc_native_int(i64 %")
            && ir
                .matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
                .count()
                >= 5,
        "integer assignment-expression values and later reads should be emitted through native-value echo output:\n{ir}"
    );
    assert!(
        ir.contains("@.str.0 = private unnamed_addr constant [4 x i8] c\"new\\00\"")
            && ir.contains(
                "call %phpc.NativeValueHandle @phpc_native_value_from_string_bytes_with_diagnostic(ptr @.str.0, i64 3"
            ),
        "string assignment-expression values should be emitted through native string value output:\n{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeScalarValue @phpc_native_bool(i1 true)")
            && ir.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "bool assignment-expression values should remain available to native-value echo output:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_MUTATION_REJECTION),
        "direct variable assignment expressions should not fall through the mutation blocker:\n{ir}"
    );
}

#[test]
fn emit_ir_rejects_reference_assignment_forms_with_specific_boundary() {
    for source in [
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
fn emit_ir_lowers_direct_variable_reference_assignment_without_write_through() {
    let ir = emit_ir_source("<?php\n$a = 1;\n$b = 2;\n$a =& $b;\n")
        .expect("direct variable reference assignment should lower as a native reference alias");

    assert!(
        ir.contains("call %phpc.NativeReferenceHandle @phpc_native_reference_from_value_and_free")
            && ir.contains("call %phpc.NativeReferenceHandle @phpc_native_reference_clone"),
        "direct variable reference assignment should materialize and clone a native reference slot:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_REFERENCE_ASSIGNMENT_REJECTION)
            && !ir.contains(LLVM_REFERENCE_WRITE_THROUGH_REJECTION),
        "direct variable reference assignment should not use the broader reference blockers until a write-through occurs:\n{ir}"
    );
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
    let error = emit_asm_source("<?php\n$value = null;\n$value ??= 2;\n").unwrap_err();

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
