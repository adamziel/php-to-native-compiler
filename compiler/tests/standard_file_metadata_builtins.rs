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
fn file_metadata_builtins_reject_trailing_slash_regular_file_paths() {
    let fixture = TempFsFixture::new("metadata-trailing-slash");
    let file = fixture.root.join("file.txt");
    fs::write(&file, "payload").expect("fixture file is created");
    let file = php_string(&file);
    let source = format!(
        r#"<?php
$file = {file};
var_dump(fileowner($file . "/"));
var_dump(filegroup($file . "/"));
var_dump(fileinode($file . "/"));
var_dump(fileperms($file . "/"));
"#,
        file = file
    );

    let execution = run_source(&source).unwrap();

    for function in ["fileowner", "filegroup", "fileinode", "fileperms"] {
        assert!(
            execution
                .stdout
                .contains(&format!("Warning: {function}(): stat failed for ")),
            "{}",
            execution.stdout
        );
    }
    assert_eq!(execution.stdout.matches("bool(false)").count(), 4);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn open_basedir_denials_use_php_display_warning_shape() {
    let fixture = TempFsFixture::new("open-basedir-display");
    fs::create_dir(fixture.root.join("allowed")).expect("allowed directory is created");
    fs::create_dir(fixture.root.join("denied")).expect("denied directory is created");
    fs::write(fixture.root.join("denied/file.txt"), "payload")
        .expect("denied fixture file is created");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$allowed = $root . "/allowed";
$denied = "../denied/file.txt";
chdir($allowed);
ini_set("open_basedir", ".");
var_dump(filetype($denied));
var_dump(lstat($denied));
var_dump(fileatime($denied));
var_dump(touch($denied));
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution
            .stdout
            .matches(
                "open_basedir restriction in effect. File(../denied/file.txt) is not within the allowed path(s): (.)"
            )
            .count(),
        4,
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stdout.matches("bool(false)").count(), 4);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_metadata_open_basedir_rejections_use_php_shaped_warning_text() {
    let fixture = TempFsFixture::new("metadata-basedir");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$allowed = $root . "/allowed";
$denied = $root . "/denied";
mkdir($allowed);
mkdir($denied);
file_put_contents($allowed . "/ok.txt", "ok");
file_put_contents($denied . "/bad.txt", "bad");
ini_set("open_basedir", $allowed);
var_dump(fileowner($denied . "/bad.txt"));
var_dump(filegroup($denied . "/bad.txt"));
var_dump(fileinode($denied . "/bad.txt"));
var_dump(fileatime($denied . "/bad.txt"));
var_dump(filectime($denied . "/bad.txt"));
var_dump(is_executable($denied . "/bad.txt"));
ini_set("open_basedir", "");
unlink($allowed . "/ok.txt");
unlink($denied . "/bad.txt");
rmdir($allowed);
rmdir($denied);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    for function in [
        "fileowner",
        "filegroup",
        "fileinode",
        "fileatime",
        "filectime",
        "is_executable",
    ] {
        assert!(
            execution.stdout.contains(&format!(
                "Warning: {function}(): open_basedir restriction in effect. File("
            )),
            "{}",
            execution.stdout
        );
        assert!(
            execution
                .stdout
                .contains(") is not within the allowed path(s): ("),
            "{}",
            execution.stdout
        );
    }
    assert_eq!(execution.stdout.matches("bool(false)").count(), 6);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chown_and_chgrp_missing_files_warn_return_false_and_continue() {
    let fixture = TempFsFixture::new("ownership");
    let missing = php_string(&fixture.root.join("missing.txt"));
    let source = format!(
        r#"<?php
echo function_exists("chown") ? "chown" : "missing";
echo ":" . (function_exists("chgrp") ? "chgrp" : "missing");
var_dump(chown({missing}, 0));
var_dump(chgrp({missing}, 0));
echo "alive";
"#,
        missing = missing
    );

    let execution = run_source(&source).unwrap();

    assert!(execution
        .stdout
        .starts_with("chown:chgrp\nWarning: chown(): No such file or directory"));
    assert!(
        execution
            .stdout
            .contains("Warning: chgrp(): No such file or directory"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.ends_with("bool(false)\nalive"));
    assert_eq!(execution.stdout.matches("bool(false)").count(), 2);
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
