use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::emit_asm_source;
use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::{run_source, run_source_with_source_file};

const LLVM_FILE_GET_CONTENTS_REJECTION: &str = "LLVM file_get_contents lowering rejects direct filesystem reads until native PHP stream wrapper handling, local file I/O, binary string byte fidelity, warning plus false recovery, stream contexts, include-path lookup, open_basedir/stat-cache behavior, references/copy-on-write, and exact native file_get_contents diagnostics exist; phpc run handles current bounded file_get_contents behavior including UTF-8 offset/length reads and selected warning-plus-false recovery";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone1197/local_file_get_contents.php")
        .display()
        .to_string()
}

#[test]
fn file_get_contents_reads_empty_php_input_placeholder() {
    let execution = run_source(
        r#"<?php
$raw = file_get_contents("php://input");
echo $raw === "" ? "empty" : "non-empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "file_get_contents";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("php://input") === "" ? "empty" : "non-empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_reads_current_local_utf8_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
$path = __DIR__ . "/local_read_payload.txt";
$contents = file_get_contents($path);
echo str_contains($contents, "ABSPATH") ? "wp-config" : "missing";
echo "|";
$call = "file_get_contents";
echo $call($path) === $contents ? "repeat" : "different";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "wp-config|repeat");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_preserves_local_binary_bytes_and_byte_offsets() {
    let path = std::env::temp_dir().join(format!(
        "{}-{}-phpc-file-get-contents-binary.bin",
        std::process::id(),
        line!()
    ));
    fs::write(&path, [0x61, 0xbd, 0x63]).expect("temporary binary file can be seeded");
    let source = format!(
        r#"<?php
$path = "{}";
echo bin2hex(file_get_contents($path));
echo "|";
echo bin2hex(file_get_contents($path, false, null, 1, 1));
"#,
        path.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "61bd63|bd");
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_file(path);
}

#[test]
fn file_get_contents_supports_bounded_use_include_path_lookup() {
    let execution = run_source_with_source_file(
        r#"<?php
set_include_path(__DIR__ . "/include_path_lib");
echo file_get_contents("wp_loader.inc", true);
"#,
        "tests/fixtures/milestone1323/file_get_contents_include_path.php".to_string(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-include-path\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_supports_bounded_offset_and_length_reads() {
    let execution = run_source_with_source_file(
        r#"<?php
$path = __DIR__ . "/offset_length_payload.inc";
echo file_get_contents($path, false, null, 3, 5);
echo "|";
echo file_get_contents($path, false, null, -5, 4);
echo "|";
set_include_path(__DIR__);
echo file_get_contents("offset_length_payload.inc", true, null, 11, 4);
"#,
        "tests/fixtures/milestone1413/file_get_contents_offset_length.php".to_string(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "defgh|wxyz|lmno");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_reads_bounded_local_file_urls() {
    let execution = run_source_with_source_file(
        r#"<?php
$path = realpath(__DIR__ . "/file_url_payload.txt");
$url = "file://" . $path;
echo file_get_contents($url, false, null, 7, 6);
"#,
        "tests/fixtures/milestone1631/file_url_wrapper_reads.php".to_string(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "stream");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_percent_decodes_bounded_local_file_urls() {
    let path = std::env::temp_dir().join("phpc-file-url-percent space#payload.txt");
    fs::write(&path, "decoded-file-url").expect("temporary percent-decoding file can be seeded");
    let encoded_path = path
        .to_string_lossy()
        .replace(' ', "%20")
        .replace('#', "%23");
    let source = format!(
        r#"<?php
echo file_get_contents("file://{}", false, null, 8, 4);
"#,
        encoded_path
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "file");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_file(path);
}

#[test]
fn file_get_contents_enforces_bounded_open_basedir_for_local_paths() {
    let root = std::env::temp_dir().join(format!(
        "{}-{}-phpc-file-get-contents-open-basedir",
        std::process::id(),
        line!()
    ));
    let allowed = root.join("allowed");
    let denied = root.join("denied");
    fs::create_dir_all(&allowed).expect("allowed open_basedir fixture directory can be created");
    fs::create_dir_all(&denied).expect("denied open_basedir fixture directory can be created");
    let allowed_file = allowed.join("payload.txt");
    let denied_file = denied.join("secret.txt");
    fs::write(&allowed_file, "allowed-payload").expect("allowed fixture file can be seeded");
    fs::write(&denied_file, "denied-payload").expect("denied fixture file can be seeded");
    let source = format!(
        r#"<?php
function capture_basedir_warning($errno, $errstr) {{
    echo "|warning:" . $errno . ":" . (str_contains($errstr, "open_basedir") ? "basedir" : "other");
    return true;
}}

ini_set("open_basedir", "{}");
set_error_handler("capture_basedir_warning", E_WARNING);
echo file_get_contents("{}");
$blocked = file_get_contents("{}");
echo $blocked === false ? "|blocked" : "|read";
"#,
        allowed.display(),
        allowed_file.display(),
        denied_file.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "allowed-payload|warning:2:basedir|warning:2:other|blocked"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn file_get_contents_applies_offset_and_length_to_php_input_seed() {
    let program = php_compiler::parse(
        r#"<?php
echo file_get_contents("php://input", false, null, 7, 4);
echo "|";
echo file_get_contents("php://input", false, null, -5);
"#,
    )
    .unwrap();
    let execution = php_compiler::interpreter::run_program_with_options(
        &program,
        php_compiler::interpreter::RunOptions {
            request_body: Some("action=save&token=abc".to_string()),
            request_method: Some("POST".to_string()),
            content_type: Some("application/x-www-form-urlencoded".to_string()),
            ..php_compiler::interpreter::RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(execution.stdout, "save|n=abc");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_recovers_missing_files_and_bad_negative_offsets_with_warning_false() {
    let missing = run_source_with_source_file(
        r#"<?php
$value = file_get_contents("tests/fixtures/missing-local-read.txt");
echo $value === false ? "false" : "value";
"#,
        "tests/fixtures/milestone1418/file_get_contents_warning_false.php".to_string(),
    )
    .unwrap();

    assert!(missing.stdout.ends_with("\nfalse"), "{}", missing.stdout);
    assert!(
        missing.stdout.contains(
            "Warning: file_get_contents(tests/fixtures/missing-local-read.txt): Failed to open stream:"
        ),
        "{}",
        missing.stdout
    );
    assert_eq!(missing.stderr, "");
    assert_eq!(missing.exit_code, 0);

    let bad_offset = run_source_with_source_file(
        r#"<?php
$value = file_get_contents("php://input", false, null, -1);
echo $value === false ? "false" : "value";
"#,
        "tests/fixtures/milestone1418/file_get_contents_warning_false.php".to_string(),
    )
    .unwrap();

    assert_eq!(bad_offset.stdout, "false");
    assert_eq!(
        bad_offset.stderr,
        "PHP Warning:  file_get_contents(): Failed to seek to position -1 in the stream in tests/fixtures/milestone1418/file_get_contents_warning_false.php on line 2"
    );
    assert_eq!(bad_offset.exit_code, 0);
}

#[test]
fn file_get_contents_directory_read_emits_notice_and_returns_false() {
    let directory = std::env::temp_dir().join(format!(
        "{}-{}-phpc-file-get-contents-directory",
        std::process::id(),
        line!()
    ));
    fs::create_dir_all(&directory).expect("temporary directory can be created");
    let source = format!(
        r#"<?php
$value = file_get_contents("{}");
echo $value === false ? "false" : "value";
"#,
        directory.display()
    );
    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.contains(
            "Notice: file_get_contents(): Read of 8192 bytes failed with errno=21 Is a directory"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("\nfalse"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn file_get_contents_recoverable_warnings_route_through_bounded_error_handlers() {
    let handled = run_source_with_source_file(
        r#"<?php
function capture_warning($errno, $errstr, $errfile, $errline) {
    echo "handler:" . $errno;
    echo ":" . (str_contains($errstr, "missing-error-handler-read.txt") ? "path" : "missing");
    echo ":" . basename($errfile);
    echo ":" . $errline;
    return true;
}
set_error_handler("capture_warning", E_WARNING);
$value = file_get_contents("tests/fixtures/missing-error-handler-read.txt");
echo $value === false ? "|false" : "|value";
"#,
        "tests/fixtures/milestone1423/file_get_contents_error_handler.php".to_string(),
    )
    .unwrap();

    assert_eq!(
        handled.stdout,
        "handler:2:path:file_get_contents_error_handler.php:10|false"
    );
    assert_eq!(handled.stderr, "");
    assert_eq!(handled.exit_code, 0);

    let passthrough = run_source_with_source_file(
        r#"<?php
function passthrough_warning($errno, $errstr) {
    echo "passthrough:" . $errno . ":" . (str_contains($errstr, "missing-passthrough-read.txt") ? "path" : "missing");
    return false;
}
error_reporting(0);
set_error_handler("passthrough_warning", E_WARNING);
$value = file_get_contents("tests/fixtures/missing-passthrough-read.txt");
echo $value === false ? "|false" : "|value";
restore_error_handler();
$quiet = file_get_contents("tests/fixtures/missing-quiet-read.txt");
echo $quiet === false ? "|quiet-false" : "|quiet-value";
"#,
        "tests/fixtures/milestone1423/file_get_contents_error_handler.php".to_string(),
    )
    .unwrap();

    assert_eq!(passthrough.stdout, "passthrough:2:path|false|quiet-false");
    assert_eq!(passthrough.stderr, "");
    assert_eq!(passthrough.exit_code, 0);

    let array_callable = run_source_with_source_file(
        r#"<?php
class WarningSink {
    public function handle($errno, $errstr, $errfile, $errline) {
        echo "array:" . $errno . ":" . (str_contains($errstr, "missing-array-handler-read.txt") ? "path" : "missing") . ":" . $errline;
        return true;
    }
}
$sink = new WarningSink();
set_error_handler(array($sink, "handle"), E_WARNING);
$value = file_get_contents("tests/fixtures/missing-array-handler-read.txt");
echo $value === false ? "|false" : "|value";
"#,
        "tests/fixtures/milestone1423/file_get_contents_error_handler.php".to_string(),
    )
    .unwrap();

    assert_eq!(array_callable.stdout, "array:2:path:10|false");
    assert_eq!(array_callable.stderr, "");
    assert_eq!(array_callable.exit_code, 0);

    let mask_miss = run_source_with_source_file(
        r#"<?php
function notice_only_warning($errno, $errstr) {
    echo "unexpected-handler";
    return true;
}
set_error_handler("notice_only_warning", E_NOTICE);
$value = file_get_contents("tests/fixtures/missing-mask-read.txt");
echo $value === false ? "false" : "value";
"#,
        "tests/fixtures/milestone1423/file_get_contents_error_handler.php".to_string(),
    )
    .unwrap();

    assert!(
        mask_miss.stdout.ends_with("\nfalse"),
        "{}",
        mask_miss.stdout
    );
    assert!(
        mask_miss.stdout.contains(
            "Warning: file_get_contents(tests/fixtures/missing-mask-read.txt): Failed to open stream:"
        ),
        "{}",
        mask_miss.stdout
    );
    assert_eq!(mask_miss.stderr, "");
    assert_eq!(mask_miss.exit_code, 0);
}

#[test]
fn file_get_contents_warning_uses_lifo_error_handler_stack() {
    let execution = run_source_with_source_file(
        r#"<?php
function first_warning($errno, $errstr) {
    echo "first:" . $errno . ":" . (str_contains($errstr, "missing-first-after-restore.txt") ? "path" : "missing");
    return true;
}
function second_warning($errno, $errstr) {
    echo "second:" . $errno . ":" . (str_contains($errstr, "missing-second-top.txt") ? "path" : "missing");
    return true;
}
set_error_handler("first_warning", E_WARNING);
$previous = set_error_handler("second_warning", E_WARNING);
echo is_string($previous) ? "prev=" . $previous : "prev=other";
$top = file_get_contents("tests/fixtures/missing-second-top.txt");
echo $top === false ? "|top-false" : "|top-value";
restore_error_handler();
$restored = file_get_contents("tests/fixtures/missing-first-after-restore.txt");
echo $restored === false ? "|restored-false" : "|restored-value";
restore_error_handler();
"#,
        "tests/fixtures/milestone1428/file_get_contents_error_handler_stack.php".to_string(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "prev=first_warningsecond:2:path|top-falsefirst:2:path|restored-false"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_rejects_forms_outside_current_subset() {
    let non_string = run_source(
        r#"<?php
try {
    file_get_contents([]);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        non_string.stdout,
        "file_get_contents(): Argument #1 ($filename) must be of type string, array given"
    );

    let stream = run_source("<?php\nfile_get_contents('php://memory');\n").unwrap_err();
    assert_eq!(stream.phase, Phase::Runtime);
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 1);
    assert_eq!(
        stream.message,
        "unsupported call file_get_contents(): only php://input, local file:// URLs, and local file paths are supported in the current stream-wrapper subset"
    );

    let bad_percent =
        run_source("<?php\nfile_get_contents('file:///tmp/bad%ZZpath');\n").unwrap_err();
    assert_eq!(bad_percent.phase, Phase::Runtime);
    assert_eq!(bad_percent.line, 2);
    assert_eq!(bad_percent.column, 1);
    assert_eq!(
        bad_percent.message,
        "unsupported call file_get_contents(): file:// URL path percent escapes must use two hexadecimal digits in the current subset"
    );

    let bad_use_include_path = run_source(
        r#"<?php
try {
    file_get_contents('php://input', []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_use_include_path.stdout,
        "file_get_contents(): Argument #2 ($use_include_path) must be of type bool, array given"
    );

    let bad_offset =
        run_source("<?php\nfile_get_contents('php://input', false, null, '0');\n").unwrap_err();
    assert_eq!(bad_offset.phase, Phase::Runtime);
    assert_eq!(bad_offset.line, 2);
    assert_eq!(bad_offset.column, 1);
    assert_eq!(
        bad_offset.message,
        "unsupported call file_get_contents(): offset argument must be int in the current subset, got string"
    );

    let bad_length = run_source(
        r#"<?php
try {
    file_get_contents('php://input', false, null, 0, -1);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_length.stdout,
        "file_get_contents(): Argument #5 ($length) must be greater than or equal to 0"
    );

    let too_many =
        run_source("<?php\nfile_get_contents('php://input', false, null, 0, 1, 2);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for file_get_contents(): expected 1 to 5 argument(s), got 6"
    );
}

#[test]
fn emit_ir_folds_file_get_contents_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("file_get_contents") ? "1" : "0";
echo is_callable("file_get_contents") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nfile_get_contents('php://input');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FILE_GET_CONTENTS_REJECTION);
}

#[test]
fn emit_ir_rejects_file_get_contents_before_lowering_arguments() {
    let error = emit_ir_source("<?php\nfile_get_contents(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FILE_GET_CONTENTS_REJECTION);
}

#[test]
fn emit_asm_rejects_file_get_contents_before_backend_execution() {
    let error = emit_asm_source("<?php\nfile_get_contents('php://input');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FILE_GET_CONTENTS_REJECTION);
}

#[test]
fn native_file_get_contents_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1203/native_file_get_contents_boundary_emit_ir.cli",
    );
}

#[test]
fn native_file_get_contents_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1203/native_file_get_contents_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1203/native_file_get_contents_boundary.phpc-source");
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
        .expect("native file_get_contents CLI snapshot is readable");
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
