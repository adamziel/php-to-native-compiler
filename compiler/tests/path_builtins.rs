use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_BASENAME_REJECTION: &str = "LLVM basename lowering rejects direct path basename calls until native PHP path string conversion, suffix handling, trailing-separator normalization, Windows/UNC and stream-wrapper path semantics, locale/codepage behavior, argument diagnostics, references/copy-on-write, and exact native basename diagnostics exist; phpc run handles current bounded basename behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn basename_executes_current_unix_path_subset() {
    let execution = run_source(
        r#"<?php
echo basename("/tmp/wordpress/wp-includes/plugin.php"), "\n";
echo basename("/tmp/wordpress/wp-includes/"), "\n";
echo "[", basename("autoload.php"), "]\n";
echo "[", basename(""), "]\n";
echo "[", basename("/"), "]\n";
echo basename("/a/b/c.php", ".php"), "\n";
echo basename("bar.gz", "bar.gz"), "\n";
echo basename("/foo/.gz", ".gz"), "\n";
$call = "basename";
echo $call("/a/b//c.php");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "plugin.php\nwp-includes\n[autoload.php]\n[]\n[]\nc\nbar.gz\n.gz\nc.php"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn basename_reports_current_argument_boundaries() {
    let non_string_path = run_source("<?php\necho basename(42);\n").unwrap_err();
    assert_eq!(non_string_path.phase, Phase::Runtime);
    assert_eq!(non_string_path.line, 2);
    assert_eq!(non_string_path.column, 6);
    assert_eq!(
        non_string_path.message,
        "unsupported call basename(): path argument must be string in the current subset, got int"
    );

    let non_string_suffix = run_source("<?php\necho basename('/a/b.php', 42);\n").unwrap_err();
    assert_eq!(non_string_suffix.phase, Phase::Runtime);
    assert_eq!(non_string_suffix.line, 2);
    assert_eq!(non_string_suffix.column, 6);
    assert_eq!(
        non_string_suffix.message,
        "unsupported call basename(): suffix argument must be string in the current subset, got int"
    );

    let too_many = run_source("<?php\necho basename('/a/b.php', '.php', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for basename(): expected 1 to 2 argument(s), got 3"
    );
}

#[test]
fn basename_is_available_through_function_lookup() {
    let execution = run_source(
        r#"<?php
echo function_exists("basename") ? "exists" : "missing";
echo "\n";
echo is_callable("basename") ? "callable" : "not-callable";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "exists\ncallable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pathinfo_executes_current_lexical_path_subset() {
    let execution = run_source(
        r#"<?php
var_dump(pathinfo(""));
var_dump(pathinfo("/"));
var_dump(pathinfo("./"));
var_dump(pathinfo("/usr/include/arpa/inet.h"));
var_dump(pathinfo(".cvsignore"));
var_dump(pathinfo("c:\..\dir1"));
echo pathinfo("/dir/test.tar.gz", PATHINFO_DIRNAME), "\n";
echo pathinfo("/dir/test.tar.gz", PATHINFO_BASENAME), "\n";
echo pathinfo("/dir/test.tar.gz", PATHINFO_EXTENSION), "\n";
echo pathinfo("/dir/test.tar.gz", PATHINFO_FILENAME), "\n";
echo PATHINFO_DIRNAME, ":", PATHINFO_BASENAME, ":", PATHINFO_EXTENSION, ":", PATHINFO_FILENAME, ":", PATHINFO_ALL;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        r#"array(2) {
  ["basename"]=>
  string(0) ""
  ["filename"]=>
  string(0) ""
}
array(3) {
  ["dirname"]=>
  string(1) "/"
  ["basename"]=>
  string(0) ""
  ["filename"]=>
  string(0) ""
}
array(4) {
  ["dirname"]=>
  string(1) "."
  ["basename"]=>
  string(1) "."
  ["extension"]=>
  string(0) ""
  ["filename"]=>
  string(0) ""
}
array(4) {
  ["dirname"]=>
  string(17) "/usr/include/arpa"
  ["basename"]=>
  string(6) "inet.h"
  ["extension"]=>
  string(1) "h"
  ["filename"]=>
  string(4) "inet"
}
array(4) {
  ["dirname"]=>
  string(1) "."
  ["basename"]=>
  string(10) ".cvsignore"
  ["extension"]=>
  string(9) "cvsignore"
  ["filename"]=>
  string(0) ""
}
array(4) {
  ["dirname"]=>
  string(1) "."
  ["basename"]=>
  string(10) "c:\..\dir1"
  ["extension"]=>
  string(5) "\dir1"
  ["filename"]=>
  string(4) "c:\."
}
/dir
test.tar.gz
gz
test.tar
1:2:4:8:15"#
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pathinfo_rejects_invalid_flag_shapes_as_value_errors() {
    let execution = run_source(
        r#"<?php
foreach (array(PATHINFO_EXTENSION | PATHINFO_FILENAME, PATHINFO_DIRNAME - 1, PATHINFO_ALL + 1) as $flag) {
    try {
        pathinfo("/usr/include/arpa/inet.h", $flag);
    } catch (ValueError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
$call = "pathinfo";
echo $call("/x/y.z", PATHINFO_FILENAME), "\n";
echo function_exists("pathinfo") ? "exists" : "missing";
echo ":";
echo is_callable("pathinfo") ? "callable" : "not-callable";
echo "\n";
var_dump((new ReflectionFunction("pathinfo"))->invoke("/x/y.z", PATHINFO_BASENAME));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:pathinfo(): Argument #2 ($flags) must be only one of the PATHINFO_* constants\n\
ValueError:pathinfo(): Argument #2 ($flags) must be one of the PATHINFO_* constants\n\
ValueError:pathinfo(): Argument #2 ($flags) must be one of the PATHINFO_* constants\n\
y\n\
exists:callable\n\
string(3) \"y.z\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_direct_basename_until_native_path_lowering_exists() {
    let error = emit_ir_source("<?php\necho basename('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_BASENAME_REJECTION);
}

#[test]
fn emit_ir_rejects_basename_before_lowering_arguments() {
    let error = emit_ir_source("<?php\necho basename(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_BASENAME_REJECTION);
}

#[test]
fn emit_asm_rejects_basename_before_backend_execution() {
    let error = emit_asm_source("<?php\necho basename('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_BASENAME_REJECTION);
}

#[test]
fn native_basename_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1198/native_basename_boundary_emit_ir.cli",
    );
}

#[test]
fn native_basename_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1198/native_basename_boundary_emit_asm.cli",
    );
}

#[test]
fn dirname_executes_current_unix_path_subset() {
    let execution = run_source(
        r#"<?php
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/autoload.php"), "\n";
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/"), "\n";
echo "[", dirname("autoload.php"), "]\n";
echo "[", dirname(""), "]\n";
echo dirname("/a/b/c.php", 2), "\n";
$call = "dirname";
echo $call("/a/b//c.php");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "/tmp/wordpress/wp-includes/sodium_compat\n/tmp/wordpress/wp-includes\n[.]\n[]\n/a\n/a/b"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dirname_reports_current_argument_boundaries() {
    let scalar_path = run_source("<?php\necho dirname(42);\n").unwrap();
    assert_eq!(scalar_path.stdout, ".");
    assert_eq!(scalar_path.stderr, "");
    assert_eq!(scalar_path.exit_code, 0);

    let non_int_levels = run_source("<?php\necho dirname('/a', '2');\n").unwrap_err();
    assert_eq!(non_int_levels.phase, Phase::Runtime);
    assert_eq!(non_int_levels.line, 2);
    assert_eq!(non_int_levels.column, 6);
    assert_eq!(
        non_int_levels.message,
        "unsupported call dirname(): levels argument must be int in the current subset, got string"
    );
}

#[test]
fn dirname_levels_zero_is_catchable_value_error_and_large_levels_saturate() {
    let execution = run_source(
        r#"<?php
try {
    dirname("/foo/bar/baz", 0);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
echo dirname("/foo/bar/baz", 1), "\n";
echo dirname("/foo/bar/baz", 2), "\n";
echo dirname("/foo/bar/baz", PHP_INT_MAX);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:dirname(): Argument #2 ($levels) must be greater than or equal to 1\n/foo/bar\n/foo\n/"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_direct_dirname_until_native_path_lowering_exists() {
    let error = emit_ir_source("<?php\necho dirname('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1198/native_basename_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, mode])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(snapshot_path))
        .expect("native basename CLI snapshot is readable");
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
