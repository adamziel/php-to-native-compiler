use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source, run_source_with_source_file};

const LLVM_REALPATH_REJECTION: &str = "LLVM realpath lowering rejects direct filesystem canonicalization calls until native filesystem canonicalization, symlink/path policy, warning/false recovery, include_path/open_basedir/stat cache, non-UTF-8 path handling, references/COW, and exact native realpath diagnostics exist; phpc run handles current bounded realpath behavior";
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

fn milestone1601_fixture_source_file() -> String {
    workspace_root()
        .join("tests/fixtures/milestone1601/realpath_cache_include_reads.php")
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
fn realpath_cache_get_exposes_bounded_successful_realpath_entries() {
    let execution = run_source_with_source_file(
        r#"<?php
$target = __DIR__ . "/realpath_target.txt";
$resolved = realpath($target);
$cache = realpath_cache_get();
echo $resolved === false ? "unresolved" : "resolved";
echo "|";
echo array_key_exists($resolved, $cache) ? "cached" : "missing";
$entry = $cache[$resolved];
echo "|";
echo $entry["realpath"] === $resolved ? "same" : "different";
echo "|";
echo $entry["is_dir"] === false ? "file" : "dir";
echo "|";
echo is_int($entry["expires"]) ? "expires-int" : "expires-other";
echo "|";
clearstatcache(false);
echo array_key_exists($resolved, realpath_cache_get()) ? "kept" : "cleared";
echo "|";
clearstatcache(true);
echo array_key_exists($resolved, realpath_cache_get()) ? "kept" : "cleared";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "resolved|cached|same|file|expires-int|kept|cleared"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clearstatcache_true_with_filename_invalidates_one_realpath_cache_entry() {
    let execution = run_source_with_source_file(
        r#"<?php
$target = __DIR__ . "/realpath_target.txt";
$resolved_target = realpath($target);
$resolved_source = realpath(__FILE__);

echo array_key_exists($resolved_target, realpath_cache_get()) ? "target-cached" : "target-missing";
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "source-cached" : "source-missing";

clearstatcache(true, $resolved_target);
echo "|";
echo array_key_exists($resolved_target, realpath_cache_get()) ? "target-kept" : "target-cleared";
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "source-kept" : "source-cleared";

clearstatcache(true, "");
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "empty-kept" : "empty-cleared";

clearstatcache(true);
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "all-kept" : "all-cleared";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "target-cached|source-cached|target-cleared|source-kept|empty-kept|all-cleared"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn realpath_cache_size_reports_bounded_request_local_cache_bytes() {
    let execution = run_source_with_source_file(
        r#"<?php
clearstatcache(true);
$call = "realpath_cache_size";
echo function_exists($call) ? "known" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call() === 0 ? "empty" : "not-empty";
$resolved_target = realpath(__DIR__ . "/realpath_target.txt");
$resolved_source = realpath(__FILE__);
$size_two = realpath_cache_size();
echo "|";
echo is_int($size_two) ? "int" : "other";
echo "|";
echo $size_two > 0 ? "positive" : "zero";
clearstatcache(true, $resolved_target);
$size_one = realpath_cache_size();
echo "|";
echo $size_one > 0 ? "one-positive" : "one-zero";
echo "|";
echo $size_one < $size_two ? "smaller" : "not-smaller";
clearstatcache(true, "");
echo "|";
echo realpath_cache_size() === $size_one ? "empty-kept" : "empty-changed";
clearstatcache(true);
echo "|";
echo realpath_cache_size() === 0 ? "cleared" : "still-sized";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "known|callable|empty|int|positive|one-positive|smaller|empty-kept|cleared"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn local_stream_reads_populate_bounded_realpath_cache_entries() {
    let execution = run_source_with_source_file(
        r#"<?php
$target = __DIR__ . "/realpath_target.txt";
clearstatcache(true);
$contents = file_get_contents($target);
$cache = realpath_cache_get();
echo str_contains($contents, "resolved") ? "read" : "missing-read";
echo "|";
echo array_key_exists($target, $cache) ? "fgc-cached" : "fgc-missing";
echo "|";
clearstatcache(true);
$handle = fopen($target, "r");
fclose($handle);
$cache = realpath_cache_get();
echo array_key_exists($target, $cache) ? "fopen-cached" : "fopen-missing";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "read|fgc-cached|fopen-cached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn successful_local_includes_populate_bounded_realpath_cache_entries() {
    let execution = run_source_with_source_file(
        r#"<?php
$target = __DIR__ . "/realpath_cache_include_target.inc";
$cache_key = realpath($target);
clearstatcache(true);
include $target;
$cache = realpath_cache_get();
echo "|";
echo array_key_exists($cache_key, $cache) ? "include-cached" : "include-missing";
echo "|";
echo realpath_cache_size() > 0 ? "include-sized" : "include-empty";
clearstatcache(true);
echo "|";
echo realpath_cache_size() === 0 ? "cleared" : "still-sized";
"#,
        milestone1601_fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "included|include-cached|include-sized|cleared"
    );
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

    let cache_size_arity = run_source("<?php\necho realpath_cache_size('extra');\n").unwrap_err();
    assert_eq!(cache_size_arity.phase, Phase::Runtime);
    assert_eq!(cache_size_arity.line, 2);
    assert_eq!(cache_size_arity.column, 6);
    assert_eq!(
        cache_size_arity.message,
        "arity mismatch for realpath_cache_size(): expected 0 argument(s), got 1"
    );
}

#[test]
fn native_metadata_recognizes_realpath_but_direct_calls_stay_unsupported() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("realpath") ? "1" : "0";
echo is_callable("realpath") ? "1" : "0";
echo function_exists("realpath_cache_size") ? "1" : "0";
echo is_callable("realpath_cache_size") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let ir_error = emit_ir_source("<?php\necho realpath('/tmp');\n").unwrap_err();
    assert_eq!(ir_error.phase, Phase::Codegen);
    assert_eq!(ir_error.line, 2);
    assert_eq!(ir_error.column, 6);
    assert_eq!(ir_error.message, LLVM_REALPATH_REJECTION);

    let asm_error = emit_asm_source("<?php\necho realpath('/tmp');\n").unwrap_err();
    assert_eq!(asm_error.phase, Phase::Codegen);
    assert_eq!(asm_error.line, 2);
    assert_eq!(asm_error.column, 6);
    assert_eq!(asm_error.message, LLVM_REALPATH_REJECTION);

    let cache_size_ir_error = emit_ir_source("<?php\necho realpath_cache_size();\n").unwrap_err();
    assert_eq!(cache_size_ir_error.phase, Phase::Codegen);
    assert_eq!(cache_size_ir_error.line, 2);
    assert_eq!(cache_size_ir_error.column, 6);
    assert_eq!(cache_size_ir_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let cache_size_asm_error = emit_asm_source("<?php\necho realpath_cache_size();\n").unwrap_err();
    assert_eq!(cache_size_asm_error.phase, Phase::Codegen);
    assert_eq!(cache_size_asm_error.line, 2);
    assert_eq!(cache_size_asm_error.column, 6);
    assert_eq!(cache_size_asm_error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_rejects_realpath_before_lowering_arguments() {
    let error = emit_ir_source("<?php\necho realpath(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_REALPATH_REJECTION);
}

#[test]
fn native_realpath_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1213/native_realpath_boundary_emit_ir.cli",
    );
}

#[test]
fn native_realpath_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1213/native_realpath_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let workspace_root = workspace_root();
    let fixture =
        workspace_root.join("tests/fixtures/milestone1213/native_realpath_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(&workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(&workspace_root)
        .args(["compile", &relative_fixture, mode])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(snapshot_path))
        .expect("native realpath CLI snapshot is readable");
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
