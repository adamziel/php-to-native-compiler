#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn is_executable_uses_local_owner_execute_bit_and_silent_false_cases() {
    let fixture = TempFsFixture::new("is-executable");
    let file = fixture.path("probe.txt");
    let dir = fixture.path("bin");
    fs::write(&file, "payload").expect("fixture file is written");
    fs::create_dir(&dir).expect("fixture directory is created");

    let source = format!(
        r#"<?php
$file = {file};
$dir = {dir};
chmod($file, 0600);
echo is_executable($file) ? "exec" : "noexec";
chmod($file, 0700);
clearstatcache();
echo "|";
echo is_executable($file) ? "exec" : "noexec";
chmod($dir, 0700);
clearstatcache();
echo "|";
echo is_executable($dir) ? "dir-exec" : "dir-noexec";
echo "|";
var_dump(is_executable($file . "/"));
var_dump(is_executable($file . chr(0) . "tail"));
var_dump(is_executable(false));
"#,
        file = php_string(&file),
        dir = php_string(&dir),
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "noexec|exec|dir-exec|bool(false)\nbool(false)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn disk_space_builtins_return_floats_for_existing_paths_and_validate_nuls() {
    let fixture = TempFsFixture::new("disk-space");
    let file = fixture.path("data.txt");
    fs::write(&file, "payload").expect("fixture file is written");

    let source = format!(
        r#"<?php
$dir = {dir};
$file = {file};
echo is_float(disk_free_space($dir)) ? "free-dir" : "bad";
echo "|";
echo is_float(diskfreespace($file)) ? "free-file" : "bad";
echo "|";
echo is_float(disk_total_space($dir)) ? "total-dir" : "bad";
echo "|";
echo is_float(disk_total_space(b"$dir")) ? "binary-dir" : "bad";
echo "|";
var_dump(disk_total_space($dir . "/missing"));
try {{
    disk_free_space($dir . chr(0));
}} catch (Error $e) {{
    echo $e->getMessage(), "\n";
}}
"#,
        dir = php_string(&fixture.root),
        file = php_string(&file),
    );

    let execution = run_source(&source).unwrap();

    assert!(execution
        .stdout
        .starts_with("free-dir|free-file|total-dir|binary-dir|"));
    assert!(execution
        .stdout
        .contains("Warning: disk_total_space(): No such file or directory in "));
    assert!(
        execution
            .stdout
            .ends_with("bool(false)\ndisk_free_space(): Argument #1 ($directory) must not contain any null bytes\n"),
        "{}",
        execution.stdout
    );
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

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TempFsFixture {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.root, fs::Permissions::from_mode(0o700));
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn php_string(path: &Path) -> String {
    let value = path.to_str().expect("temporary path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
