use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[cfg(unix)]
#[test]
fn is_link_checks_current_local_filesystem_subset() {
    let temp = TempLinkFixture::new("direct");
    temp.write_file("target.txt", "target");
    temp.symlink_file("target.txt", "target-link.txt");

    let source = format!(
        r#"<?php
echo is_link({link}) ? "link" : "not-link";
echo "|";
echo is_link({target}) ? "link" : "not-link";
echo "|";
echo is_link({missing}) ? "link" : "missing";
"#,
        link = php_string(&temp.path("target-link.txt")),
        target = php_string(&temp.path("target.txt")),
        missing = php_string(&temp.path("missing.txt")),
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "link|not-link|missing");
    assert_eq!(execution.exit_code, 0);
}

#[cfg(unix)]
#[test]
fn is_link_is_available_through_string_valued_calls() {
    let temp = TempLinkFixture::new("dynamic");
    temp.write_file("target.txt", "target");
    temp.symlink_file("target.txt", "target-link.txt");

    let source = format!(
        r#"<?php
$call = "is_link";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call({link}) ? "link" : "not-link";
"#,
        link = php_string(&temp.path("target-link.txt")),
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "yes|callable|link");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_link_rejects_forms_outside_current_subset() {
    let arity = run_source("<?php\necho is_link();\n").unwrap();
    assert_eq!(arity.exit_code, 255);
    assert!(
        arity
            .stdout
            .contains("Too few arguments to function is_link(), 0 passed"),
        "{}",
        arity.stdout
    );

    let scalar = run_source("<?php\nvar_dump(is_link(42)); var_dump(is_link(false));\n").unwrap();
    assert_eq!(scalar.stdout, "bool(false)\nbool(false)\n");
    assert_eq!(scalar.exit_code, 0);

    let stream = run_source("<?php\necho is_link('php://memory');\n").unwrap_err();
    assert_eq!(stream.phase, Phase::Runtime);
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 6);
    assert_eq!(
        stream.message,
        "unsupported call is_link(): stream wrappers are not supported in the current subset"
    );
}

#[test]
fn native_metadata_recognizes_is_link_but_direct_calls_stay_unsupported() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("is_link") ? "1" : "0";
echo is_callable("is_link") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let ir_error = emit_ir_source("<?php\necho is_link('/tmp');\n").unwrap_err();
    assert_eq!(ir_error.phase, Phase::Codegen);
    assert_eq!(ir_error.line, 2);
    assert_eq!(ir_error.column, 6);
    assert_eq!(ir_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let asm_error = emit_asm_source("<?php\necho is_link('/tmp');\n").unwrap_err();
    assert_eq!(asm_error.phase, Phase::Codegen);
    assert_eq!(asm_error.line, 2);
    assert_eq!(asm_error.column, 6);
    assert_eq!(asm_error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[cfg(unix)]
struct TempLinkFixture {
    root: PathBuf,
}

#[cfg(unix)]
impl TempLinkFixture {
    fn new(label: &str) -> Self {
        let unique = format!(
            "phpc-is-link-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is after Unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        fs::create_dir(&root).expect("temporary is_link fixture directory is created");
        Self { root }
    }

    fn path(&self, file: &str) -> PathBuf {
        self.root.join(file)
    }

    fn write_file(&self, file: &str, contents: &str) {
        fs::write(self.path(file), contents).expect("temporary target file is written");
    }

    fn symlink_file(&self, target: &str, link: &str) {
        std::os::unix::fs::symlink(target, self.path(link)).expect("temporary symlink is created");
    }
}

#[cfg(unix)]
impl Drop for TempLinkFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn php_string(path: &Path) -> String {
    let value = path
        .to_str()
        .expect("temporary is_link path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
