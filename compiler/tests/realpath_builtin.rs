use std::path::{Path, PathBuf};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source, run_source_with_source_file};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a workspace root")
        .to_path_buf()
}

fn fixture_source_file() -> String {
    workspace_root()
        .join("tests/fixtures/milestone1207/realpath.php")
        .display()
        .to_string()
}

fn target_path() -> PathBuf {
    workspace_root().join("tests/fixtures/milestone1207/realpath_target.txt")
}

#[test]
fn realpath_resolves_existing_local_paths_and_returns_false_for_missing() {
    let expected = std::fs::canonicalize(target_path())
        .expect("realpath fixture target exists")
        .into_os_string()
        .into_string()
        .expect("realpath fixture target is valid UTF-8");

    let execution = run_source(
        r#"<?php
$resolved = realpath("tests/fixtures/milestone1207/realpath_target.txt");
echo is_string($resolved) ? $resolved : "not-string";
echo "\n";
echo realpath("tests/fixtures/milestone1207/missing-target.txt") === false ? "missing" : "unexpected";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, format!("{expected}\nmissing"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn realpath_is_available_through_string_valued_calls() {
    let execution = run_source_with_source_file(
        r#"<?php
$call = "realpath";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
$resolved = $call(__DIR__ . "/realpath_target.txt");
echo basename($resolved);
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|realpath_target.txt");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn realpath_reports_current_argument_and_local_path_boundaries() {
    let non_string = run_source("<?php\necho realpath(42);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call realpath(): path argument must be string in the current subset, got int"
    );

    let stream = run_source("<?php\necho realpath('php://input');\n").unwrap_err();
    assert_eq!(stream.phase, Phase::Runtime);
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call realpath(): stream wrappers are not supported in the current subset"
    );

    let too_many = run_source("<?php\necho realpath('/tmp', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for realpath(): expected 1 argument(s), got 2"
    );
}

#[test]
fn native_metadata_recognizes_realpath_but_direct_calls_stay_unsupported() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("realpath") ? "1" : "0";
echo is_callable("realpath") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let ir_error = emit_ir_source("<?php\necho realpath('/tmp');\n").unwrap_err();
    assert_eq!(ir_error.phase, Phase::Codegen);
    assert_eq!(ir_error.line, 2);
    assert_eq!(ir_error.column, 6);
    assert_eq!(ir_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let asm_error = emit_asm_source("<?php\necho realpath('/tmp');\n").unwrap_err();
    assert_eq!(asm_error.phase, Phase::Codegen);
    assert_eq!(asm_error.line, 2);
    assert_eq!(asm_error.column, 6);
    assert_eq!(asm_error.message, LLVM_FUNCTION_CALL_REJECTION);
}
