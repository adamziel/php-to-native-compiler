use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_IS_WRITABLE_REJECTION: &str = "LLVM is_writable lowering rejects direct filesystem writability checks until native writability checks, permission policy, warnings, include_path/open_basedir, stream wrappers, symlink/stat-cache/TOCTOU behavior, non-UTF-8 paths, references/COW, and exact native is_writable diagnostics exist; phpc run handles current bounded is_writable behavior";

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
    let arity = run_source("<?php\necho is_writable();\n").unwrap();
    assert_eq!(arity.exit_code, 255);
    assert!(
        arity
            .stdout
            .contains("Too few arguments to function is_writable(), 0 passed"),
        "{}",
        arity.stdout
    );
    assert_eq!(arity.stderr, "");

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
    assert_eq!(ir_error.message, LLVM_IS_WRITABLE_REJECTION);

    let asm_error = emit_asm_source("<?php\necho is_writable('/tmp');\n").unwrap_err();
    assert_eq!(asm_error.phase, Phase::Codegen);
    assert_eq!(asm_error.line, 2);
    assert_eq!(asm_error.column, 6);
    assert_eq!(asm_error.message, LLVM_IS_WRITABLE_REJECTION);
}

#[test]
fn emit_ir_rejects_is_writable_before_lowering_arguments() {
    let error = emit_ir_source("<?php\necho is_writable(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_IS_WRITABLE_REJECTION);
}

#[test]
fn native_is_writable_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1218/native_is_writable_boundary_emit_ir.cli",
    );
}

#[test]
fn native_is_writable_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1218/native_is_writable_boundary_emit_asm.cli",
    );
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

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a workspace root")
        .to_path_buf()
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let workspace_root = workspace_root();
    let fixture =
        workspace_root.join("tests/fixtures/milestone1218/native_is_writable_boundary.phpc-source");
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
        .expect("native is_writable CLI snapshot is readable");
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
