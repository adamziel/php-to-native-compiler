use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";

#[test]
fn phpc_run_still_handles_current_control_flow_subset() {
    let execution = run_source(
        r#"<?php
if (false) {
    echo "bad\n";
} elseif (true) {
    echo "elseif\n";
} else {
    echo "else\n";
}

$i = 0;
while ($i < 4) {
    $i = $i + 1;
    if ($i == 2) {
        continue;
    }
    if ($i == 4) {
        break;
    }
    echo "w", $i, "\n";
}

for ($j = 0; $j < 3; $j = $j + 1) {
    echo "f", $j, "\n";
}

$k = 0;
do {
    echo "d", $k, "\n";
    $k = $k + 1;
} while ($k < 2);

    switch ($i) {
    case 4:
        echo "switch";
        break;
    default:
        echo "default\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "elseif\nw1\nw3\nf0\nf1\nf2\nd0\nd1\nswitch"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_structured_control_flow_with_specific_boundary() {
    for source in [
        "<?php\nif (true) { echo 1; }\n",
        "<?php\nif (false) { echo 0; } elseif (true) { echo 1; }\n",
        "<?php\nwhile (true) { echo 1; break; }\n",
        "<?php\nfor ($i = 0; $i < 1; $i = $i + 1) { echo $i; }\n",
        "<?php\ndo { echo 1; } while (false);\n",
        "<?php\nswitch (1) { case 1: echo 1; break; }\n",
        "<?php\nbreak;\n",
        "<?php\ncontinue;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_control_flow_before_lowering_nested_operands_or_bodies() {
    for source in [
        "<?php\nif (missing_call()) { echo [1]; }\n",
        "<?php\nwhile (missing_call()) { echo [1]; }\n",
        "<?php\nfor ($i = missing_call(); $i < 1; $i = $i + 1) { echo [1]; }\n",
        "<?php\ndo { echo [1]; } while (missing_call());\n",
        "<?php\nswitch (missing_call()) { case [1]: echo 1; }\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_control_flow_before_backend_execution() {
    let error = emit_asm_source("<?php\nif (true) { echo 1; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
}

#[test]
fn native_control_flow_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone175/native_control_flow_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone175/native_control_flow_boundary_emit_ir.cli"),
    )
    .expect("native control-flow CLI snapshot is readable");
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
