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

#[test]
fn chmod_and_readability_predicates_cover_phpt_permission_variations() {
    let fixture = TempFsFixture::new("permission-predicates");
    let file = fixture.path("probe.txt");
    let dir = fixture.path("subdir");
    fs::write(&file, "payload").expect("fixture file is written");
    fs::create_dir(&dir).expect("fixture directory is created");

    let source = format!(
        r#"<?php
$old = getcwd();
$root = {root};
$file = {file};
$badPath = {bad_path};
chdir($root);
var_dump(chmod($file, 0002));
clearstatcache();
var_dump(is_writable($file));
var_dump(is_writeable($file));
var_dump(chmod($file, 0200));
clearstatcache();
var_dump(is_writable($file));
var_dump(is_readable(0));
var_dump(is_readable(1234));
var_dump(is_readable(-2.34555));
var_dump(is_readable(true));
var_dump(is_readable(false));
var_dump(is_readable(" "));
var_dump(chmod($file, 0777));
var_dump(chmod($badPath, 0755));
clearstatcache();
printf("%o\n", fileperms($file) & 0777);
var_dump(chmod($root . "/missing.txt", 0777));
chmod($file, 0600);
chdir($old);
"#,
        root = php_string(&fixture.root),
        file = php_string(&file),
        bad_path = php_string(&dir.join("missing/../../probe.txt")),
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.starts_with(concat!(
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(true)\n",
            "\n",
            "Warning: chmod(): No such file or directory in ",
        )),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("bool(false)\n777\n\nWarning: chmod(): No such file or directory in "),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("bool(false)\n"),
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
