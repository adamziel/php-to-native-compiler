use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn is_writable_checks_current_local_filesystem_subset() {
    let temp = TempWritableFixture::new("direct");
    temp.write_file("writable.txt", "writable");
    temp.write_readonly_file("readonly.txt", "readonly");

    let source = format!(
        r#"<?php
echo is_writable({writable}) ? "writable" : "not-writable";
echo "|";
echo is_writable({readonly}) ? "writable" : "not-writable";
echo "|";
echo is_writable({missing}) ? "writable" : "missing";
"#,
        writable = php_string(&temp.path("writable.txt")),
        readonly = php_string(&temp.path("readonly.txt")),
        missing = php_string(&temp.path("missing.txt")),
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "writable|not-writable|missing");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_writable_is_available_through_string_valued_calls() {
    let temp = TempWritableFixture::new("dynamic");
    temp.write_file("target.txt", "target");

    let source = format!(
        r#"<?php
$call = "is_writable";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call({target}) ? "writable" : "not-writable";
"#,
        target = php_string(&temp.path("target.txt")),
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "yes|callable|writable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_writable_rejects_forms_outside_current_subset() {
    let arity = run_source("<?php\necho is_writable();\n").unwrap_err();
    assert_eq!(arity.phase, Phase::Runtime);
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 6);
    assert_eq!(
        arity.message,
        "arity mismatch for is_writable(): expected 1 argument(s), got 0"
    );

    let type_error = run_source("<?php\necho is_writable(42);\n").unwrap_err();
    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 6);
    assert_eq!(
        type_error.message,
        "unsupported call is_writable(): path argument must be string in the current subset, got int"
    );

    let stream = run_source("<?php\necho is_writable('php://memory');\n").unwrap_err();
    assert_eq!(stream.phase, Phase::Runtime);
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call is_writable(): stream wrappers are not supported in the current subset"
    );
}

#[test]
fn native_metadata_recognizes_is_writable_but_direct_calls_stay_unsupported() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("is_writable") ? "1" : "0";
echo is_callable("is_writable") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let ir_error = emit_ir_source("<?php\necho is_writable('/tmp');\n").unwrap_err();
    assert_eq!(ir_error.phase, Phase::Codegen);
    assert_eq!(ir_error.line, 2);
    assert_eq!(ir_error.column, 6);
    assert_eq!(ir_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let asm_error = emit_asm_source("<?php\necho is_writable('/tmp');\n").unwrap_err();
    assert_eq!(asm_error.phase, Phase::Codegen);
    assert_eq!(asm_error.line, 2);
    assert_eq!(asm_error.column, 6);
    assert_eq!(asm_error.message, LLVM_FUNCTION_CALL_REJECTION);
}

struct TempWritableFixture {
    root: PathBuf,
}

impl TempWritableFixture {
    fn new(label: &str) -> Self {
        let unique = format!(
            "phpc-is-writable-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is after Unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        fs::create_dir(&root).expect("temporary is_writable fixture directory is created");
        Self { root }
    }

    fn path(&self, file: &str) -> PathBuf {
        self.root.join(file)
    }

    fn write_file(&self, file: &str, contents: &str) {
        fs::write(self.path(file), contents).expect("temporary writable file is written");
    }

    fn write_readonly_file(&self, file: &str, contents: &str) {
        let path = self.path(file);
        fs::write(&path, contents).expect("temporary readonly file is written");
        let mut permissions = fs::metadata(&path)
            .expect("temporary readonly file has metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).expect("temporary readonly permissions are set");
    }
}

impl Drop for TempWritableFixture {
    fn drop(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let mut permissions = metadata.permissions();
                    permissions.set_readonly(false);
                    let _ = fs::set_permissions(entry.path(), permissions);
                }
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn php_string(path: &Path) -> String {
    let value = path
        .to_str()
        .expect("temporary is_writable path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
