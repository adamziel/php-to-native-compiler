use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source_with_source_file;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone1187/is_file.php")
        .display()
        .to_string()
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source_with_source_file(source, fixture_source_file()).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn is_file_checks_current_local_filesystem_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
echo is_file(__FILE__) ? "file" : "missing";
echo "|";
echo is_file(__DIR__) ? "dir" : "not-file";
echo "|";
echo is_file(__DIR__ . "/missing-file.php") ? "file" : "missing";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "file|not-file|missing");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_file_is_available_through_string_valued_calls() {
    let execution = run_source_with_source_file(
        r#"<?php
$call = "is_file";
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
fn is_file_treats_trailing_slash_regular_file_path_as_not_file() {
    let fixture = TempFsFixture::new("is-file-trailing-slash");
    let file = fixture.root.join("regular.tmp");
    fs::write(&file, "payload").expect("regular fixture file is written");
    let trailing_file = format!("{}/", file.to_str().expect("temporary path is valid UTF-8"));

    let source = format!(
        r#"<?php
var_dump(is_file({file}));
var_dump(is_file({trailing_file}));
"#,
        file = php_string(&file),
        trailing_file = php_string(&trailing_file),
    );

    let execution = run_source_with_source_file(&source, fixture_source_file()).unwrap();

    assert_eq!(execution.stdout, "bool(true)\nbool(false)\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_file_rejects_forms_outside_current_subset() {
    let arity = run_source_with_source_file(
        r#"<?php
echo is_file();
"#,
        fixture_source_file(),
    );
    let arity = arity.unwrap();
    assert_eq!(arity.exit_code, 255);
    assert!(
        arity
            .stdout
            .contains("Too few arguments to function is_file(), 0 passed"),
        "{}",
        arity.stdout
    );
    assert_eq!(arity.stderr, "");

    let type_error = runtime_error(
        r#"<?php
echo is_file(42);
"#,
    );
    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 6);
    assert_eq!(
        type_error.message,
        "unsupported call is_file(): path argument must be string in the current subset, got int"
    );

    let stream = runtime_error(
        r#"<?php
echo is_file("php://memory");
"#,
    );
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call is_file(): stream wrappers are not supported in the current subset"
    );
}

struct TempFsFixture {
    root: PathBuf,
}

impl TempFsFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is after Unix epoch")
                .as_nanos()
        ));
        fs::create_dir(&root).expect("temporary filesystem fixture root is created");
        Self { root }
    }
}

impl Drop for TempFsFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn php_string(value: impl AsRef<Path>) -> String {
    let value = value
        .as_ref()
        .to_str()
        .expect("temporary path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

#[test]
fn emit_ir_rejects_is_file_until_native_filesystem_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo is_file("wp-config.php") ? "yes" : "no";
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_is_file_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("is_file") ? "1" : "0";
echo is_callable("is_file") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
