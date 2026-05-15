use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";

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
fn emit_ir_rejects_mutation_forms_with_specific_boundary() {
    for source in [
        "<?php\n$value = 1;\n$value += 2;\n",
        "<?php\n$value = null;\n$value ??= 2;\n",
        "<?php\n$value = 1;\n$value++;\n",
        "<?php\n$value = 1;\necho ($value = 2);\n",
        "<?php\n$a = 1;\n$b = 2;\n$a =& $b;\n",
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
fn emit_ir_rejects_mutation_before_lowering_nested_operands() {
    for source in [
        "<?php\n$value = 1;\n$value += missing_call();\n",
        "<?php\n$value = null;\n$value ??= missing_call();\n",
        "<?php\necho ($value = missing_call());\n",
        "<?php\necho ($value += missing_call());\n",
        "<?php\necho ($value ??= missing_call());\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_MUTATION_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_mutation_before_backend_execution() {
    let error = emit_asm_source("<?php\n$value = 1;\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
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
