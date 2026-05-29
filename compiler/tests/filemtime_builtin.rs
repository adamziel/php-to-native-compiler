use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source_with_source_file;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone1313/filemtime_local_metadata.php")
        .display()
        .to_string()
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source_with_source_file(source, fixture_source_file()).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn filemtime_reads_current_local_path_metadata_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
$mtime = filemtime(__FILE__);
echo is_int($mtime) ? "int" : "not-int";
echo "|";
echo $mtime > 0 ? "positive" : "empty";
echo "|";
echo is_int(filemtime(__DIR__)) ? "dir-int" : "dir-false";
echo "|";
echo filemtime(__DIR__ . "/missing-file.php") === false ? "missing-false" : "missing-time";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .starts_with("int|positive|dir-int|\nWarning: filemtime(): stat failed"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("missing-false"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filemtime_is_available_through_string_valued_calls() {
    let execution = run_source_with_source_file(
        r#"<?php
$call = "filemtime";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(__FILE__) === filemtime(__FILE__) ? "repeat" : "different";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|repeat");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filemtime_rejects_forms_outside_current_subset() {
    let arity = run_source_with_source_file(
        r#"<?php
echo filemtime();
"#,
        fixture_source_file(),
    )
    .unwrap();
    assert_eq!(arity.exit_code, 255);
    assert!(
        arity
            .stdout
            .contains("Too few arguments to function filemtime(), 0 passed"),
        "{}",
        arity.stdout
    );

    let scalar = run_source_with_source_file(
        r#"<?php
var_dump(filemtime(42));
"#,
        fixture_source_file(),
    )
    .unwrap();
    assert!(scalar
        .stdout
        .contains("Warning: filemtime(): stat failed for 42"));
    assert!(scalar.stdout.ends_with("bool(false)\n"));
    assert_eq!(scalar.exit_code, 0);

    let stream = runtime_error(
        r#"<?php
echo filemtime("php://input");
"#,
    );
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call filemtime(): stream wrappers are not supported in the current subset"
    );
}

#[test]
fn emit_ir_rejects_filemtime_until_native_filesystem_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo filemtime("wp-config.php");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_filemtime_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("filemtime") ? "1" : "0";
echo is_callable("filemtime") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
