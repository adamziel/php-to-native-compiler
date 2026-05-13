use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn phpc_run_still_handles_current_array_subset() {
    let execution = run_source(
        r#"<?php
$items = ["name" => "Ada", 2 => "two", "02" => "zero two"];
$items[] = "next";
$items["name"] = "Grace";
echo $items["name"], "|", $items[2], "|", $items[3], "\n";
unset($items["02"]);
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
print_r(array_values($items));
print_r(array_keys($items));
echo count(array_filter($items)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Grace|two|next\nname=Grace\n2=two\n3=next\nArray\n(\n    [0] => Grace\n    [1] => two\n    [2] => next\n)\nArray\n(\n    [0] => name\n    [1] => 2\n    [2] => 3\n)\n3\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_literals_offsets_iteration_unset_and_builtins() {
    for source in [
        "<?php\necho [1];\n",
        "<?php\n$items = [1];\necho $items[0];\n",
        "<?php\n$items[0] = 1;\n",
        "<?php\nforeach ([1] as $value) { echo $value; }\n",
        "<?php\n$items = [1];\nunset($items[0]);\n",
        "<?php\necho array_values([1]);\n",
        "<?php\necho array_filter([1], \"strlen\");\n",
        "<?php\necho count([1]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_ARRAY_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_arrays_before_backend_execution() {
    let error = emit_asm_source("<?php\necho [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}

#[test]
fn native_array_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone174/native_array_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone174/native_array_boundary_emit_ir.cli"),
    )
    .expect("native array CLI snapshot is readable");
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
