use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source_with_source_file;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone734/file_exists.php")
        .display()
        .to_string()
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source_with_source_file(source, fixture_source_file()).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn file_exists_checks_current_local_filesystem_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
echo file_exists(__FILE__) ? "file" : "missing";
echo "|";
echo file_exists(__DIR__) ? "dir" : "missing";
echo "|";
echo file_exists(__DIR__ . "/missing-file.php") ? "exists" : "missing";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "file|dir|missing");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_exists_is_available_through_string_valued_calls() {
    let execution = run_source_with_source_file(
        r#"<?php
$call = "file_exists";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(__FILE__) ? "file" : "missing";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|file");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_exists_with_synthetic_relative_source_paths_uses_existing_repo_relative_resolution() {
    let execution = run_source_with_source_file(
        r#"<?php
echo file_exists(__FILE__) ? "file" : "missing";
echo "|";
echo file_exists(__DIR__) ? "dir" : "missing";
"#,
        "tests/fixtures/milestone734/file_exists.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "file|dir");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_exists_rejects_forms_outside_current_subset() {
    let arity = run_source_with_source_file(
        r#"<?php
echo file_exists();
"#,
        fixture_source_file(),
    )
    .unwrap();
    assert_eq!(arity.stderr, "");
    assert_eq!(arity.exit_code, 255);
    assert!(arity
        .stdout
        .contains("Too few arguments to function file_exists()"));
    assert!(arity.stdout.contains("exactly 1 expected"));

    let type_error = runtime_error(
        r#"<?php
echo file_exists(42);
"#,
    );
    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 6);
    assert_eq!(
        type_error.message,
        "unsupported call file_exists(): path argument must be string in the current subset, got int"
    );

    let stream = runtime_error(
        r#"<?php
echo file_exists("php://memory");
"#,
    );
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call file_exists(): stream wrappers are not supported in the current subset"
    );
}

#[test]
fn emit_ir_rejects_file_exists_until_native_filesystem_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo file_exists("wp-content/db.php") ? "yes" : "no";
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_file_exists_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("file_exists") ? "1" : "0";
echo is_callable("file_exists") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
