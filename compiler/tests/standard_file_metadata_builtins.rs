#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn file_metadata_builtins_cover_local_owner_inode_group_and_type_queries() {
    let fixture = TempFsFixture::new("metadata");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$file = $root . "/file.txt";
$dir = $root . "/dir";
file_put_contents($file, "payload");
mkdir($dir);
echo gettype(fileinode($file)) . ":";
echo gettype(fileowner($file)) . ":";
echo gettype(filegroup($file)) . ":";
echo gettype(fileatime($file)) . ":";
echo gettype(filectime($file)) . ":";
echo filetype($file) . ":" . filetype($dir) . ":";
echo filegroup($root . "/missing") === false ? "missing" : "bad";
unlink($file);
rmdir($dir);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.starts_with(
            "integer:integer:integer:integer:integer:file:dir:\nWarning: filegroup(): stat failed"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("missing"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_metadata_builtins_emit_php_shaped_scalar_path_warnings() {
    let source = r#"<?php
var_dump(filegroup("/no/such/file/dir"));
var_dump(fileinode(100));
var_dump(fileowner("string"));
var_dump(fileperms(" "));
var_dump(filetype("missing-type"));
var_dump(stat(false));
var_dump(stat(" "));
var_dump(lstat(22));
var_dump(filegroup("bad" . chr(0) . "path"));
var_dump(fileatime("/no/such/file/or/dir"));
var_dump(filemtime("missing-mtime"));
var_dump(filectime(1234));
var_dump(touch(false));
var_dump(touch(""));
"#;

    let execution = run_source(source).unwrap();

    assert!(
        execution
            .stdout
            .contains("Warning: filegroup(): stat failed for /no/such/file/dir"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: fileinode(): stat failed for 100"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: fileowner(): stat failed for string"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: fileperms(): stat failed for  "),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filetype(): Lstat failed for missing-type"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: stat(): stat failed for  "),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: lstat(): Lstat failed for 22"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filegroup(): Filename contains null byte"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: fileatime(): stat failed for /no/such/file/or/dir"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filemtime(): stat failed for missing-mtime"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filectime(): stat failed for 1234"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stdout.matches("bool(false)").count(), 14);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_metadata_predicates_enforce_bounded_open_basedir_before_local_metadata_reads() {
    let fixture = TempFsFixture::new("open-basedir-metadata");
    let allowed = fixture.root.join("allowed");
    let denied = fixture.root.join("denied");
    fs::create_dir(&allowed).expect("allowed open_basedir directory is created");
    fs::create_dir(&denied).expect("denied open_basedir directory is created");
    let allowed_file = allowed.join("visible.txt");
    let denied_file = denied.join("secret.txt");
    fs::write(&allowed_file, "visible").expect("allowed fixture file is written");
    fs::write(&denied_file, "secret").expect("denied fixture file is written");

    let source = format!(
        r#"<?php
function capture_basedir_warning($errno, $errstr) {{
    echo str_contains($errstr, "open_basedir restriction in effect") ? "W:basedir\n" : "W:other\n";
    return true;
}}

ini_set("open_basedir", {allowed});
set_error_handler("capture_basedir_warning", E_WARNING);
echo file_exists({allowed_file}) ? "allowed\n" : "bad\n";
var_dump(file_exists({denied_file}));
var_dump(is_file({denied_file}));
var_dump(is_dir({denied_dir}));
var_dump(is_readable({denied_file}));
var_dump(is_writable({denied_file}));
var_dump(is_link({denied_file}));
var_dump(filesize({denied_file}));
var_dump(stat({denied_file}));
var_dump(realpath({denied_file}));
"#,
        allowed = php_string(&allowed),
        allowed_file = php_string(&allowed_file),
        denied_file = php_string(&denied_file),
        denied_dir = php_string(&denied),
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.starts_with("allowed\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stdout.matches("W:basedir\n").count(), 9);
    assert_eq!(execution.stdout.matches("bool(false)\n").count(), 9);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

struct TempFsFixture {
    root: PathBuf,
}

impl TempFsFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-standard-file-{label}-{}-{}",
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

fn php_string(path: &Path) -> String {
    let value = path.to_str().expect("temporary path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
