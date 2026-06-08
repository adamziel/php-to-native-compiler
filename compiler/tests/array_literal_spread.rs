use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_literal_spread_reindexes_int_keys_and_overwrites_string_keys() {
    let execution = run_source(
        r#"<?php
$left = [2 => "two", "a" => "left", "b" => "keep"];
$middle = ["a" => "middle", 9 => "nine"];
$result = ["start", ...$left, "a" => "explicit", ...$middle, "tail"];
foreach ($result as $key => $value) {
    echo $key, "=", $value, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "0=start\n",
            "1=two\n",
            "a=middle\n",
            "b=keep\n",
            "2=nine\n",
            "3=tail\n"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_literal_spread_preserves_reference_backed_slots() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
$source = [&$name, "x" => &$name];
$copy = [...$source];
$name = "Lin";
var_dump($copy[0], $copy["x"]);
$copy[0] = "Grace";
var_dump($name, $source[0], $copy["x"]);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(3) \"Lin\"\n",
            "string(3) \"Lin\"\n",
            "string(5) \"Grace\"\n",
            "string(5) \"Grace\"\n",
            "string(5) \"Grace\"\n"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_literal_spread_reports_non_array_operands() {
    let error = runtime_error("<?php\n$items = [...42];\n");
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 14);
    assert!(
        error
            .message
            .contains("Only arrays and Traversables can be unpacked, int given"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_array_literal_spread_at_existing_array_lowering_boundary() {
    let error = emit_ir_source("<?php\n$items = [...[1]];\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert!(error.message.contains("LLVM array lowering rejects arrays"));
}

#[test]
fn array_literal_spread_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone2343/array_literal_spread.php");

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "run",
            "tests/fixtures/milestone2343/array_literal_spread.php",
        ])
        .output()
        .expect("phpc run should execute array literal spread fixture");

    let expected = fs::read_to_string(fixture.with_extension("cli"))
        .expect("array literal spread CLI snapshot is readable");
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
