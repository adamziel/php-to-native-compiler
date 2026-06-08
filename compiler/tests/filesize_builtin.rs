use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source_with_source_file;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone1308/filesize_local_metadata.php")
        .display()
        .to_string()
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source_with_source_file(source, fixture_source_file()).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn filesize_reads_current_local_file_metadata_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
$size = filesize(__FILE__);
echo is_int($size) ? "int" : "not-int";
echo "|";
echo $size > 0 ? "positive" : "empty";
echo "|";
echo filesize(__DIR__) === false ? "dir-false" : "dir-size";
echo "|";
echo filesize(__DIR__ . "/missing-file.php") === false ? "missing-false" : "missing-size";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert!(
        execution.stdout.starts_with("int|positive|dir-size|"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filesize(): stat failed for "),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.ends_with("missing-false"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filesize_is_available_through_string_valued_calls() {
    let execution = run_source_with_source_file(
        r#"<?php
$call = "filesize";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(__FILE__) === filesize(__FILE__) ? "repeat" : "different";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|repeat");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filesize_rejects_forms_outside_current_subset() {
    let arity = run_source_with_source_file(
        r#"<?php
echo filesize();
"#,
        fixture_source_file(),
    )
    .unwrap();
    assert_eq!(arity.stderr, "");
    assert_eq!(arity.exit_code, 255);
    assert!(arity
        .stdout
        .contains("Too few arguments to function filesize()"));
    assert!(arity.stdout.contains("exactly 1 expected"));

    let type_warning = run_source_with_source_file(
        r#"<?php
echo filesize(42);
"#,
        fixture_source_file(),
    )
    .unwrap();
    assert_eq!(type_warning.stderr, "");
    assert_eq!(type_warning.exit_code, 0);
    assert!(type_warning
        .stdout
        .contains("Warning: filesize(): stat failed for 42"));

    let stream = runtime_error(
        r#"<?php
echo filesize("php://input");
"#,
    );
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call filesize(): stream wrappers are not supported in the current subset"
    );
}

#[test]
fn emit_ir_rejects_filesize_until_native_filesystem_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo filesize("wp-config.php");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_filesize_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("filesize") ? "1" : "0";
echo is_callable("filesize") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
