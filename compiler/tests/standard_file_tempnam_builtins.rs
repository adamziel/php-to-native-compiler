#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn tempnam_and_sys_get_temp_dir_cover_standard_temp_file_prerequisites() {
    let fixture = TempFsFixture::new("tempnam");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
echo is_string(sys_get_temp_dir()) ? "tempdir\n" : "bad-tempdir\n";
$a = tempnam($root, "pref_");
$b = tempnam($root, "/ignored/component");
$c = tempnam("", "fallback_");
$d = tempnam($root, 123);
var_dump(file_exists($a));
var_dump(file_exists($b));
var_dump(file_exists($c));
var_dump(file_exists($d));
echo dirname($a) === $root ? "dir-a\n" : "bad-dir-a\n";
echo str_starts_with(basename($a), "pref_") ? "prefix-a\n" : "bad-prefix-a\n";
echo str_starts_with(basename($b), "component") ? "prefix-b\n" : "bad-prefix-b\n";
echo str_starts_with(basename($d), "123") ? "prefix-d\n" : "bad-prefix-d\n";
echo dirname($c) === sys_get_temp_dir() ? "fallback\n" : "bad-fallback\n";
echo "perms:";
printf("%o\n", fileperms($a));
unlink($a);
unlink($b);
unlink($c);
unlink($d);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert_eq!(
        execution.stdout,
        concat!(
            "tempdir\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "dir-a\n",
            "prefix-a\n",
            "prefix-b\n",
            "prefix-d\n",
            "fallback\n",
            "perms:100600\n",
        )
    );
}

#[test]
fn tempnam_argument_errors_are_catchable_php_errors() {
    let fixture = TempFsFixture::new("tempnam-errors");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
foreach ([["dir", "\0"], ["prefix", "\0"], ["dir", []], ["prefix", []]] as $case) {{
    try {{
        if ($case[0] === "dir") {{
            tempnam($case[1], "pref_");
        }} else {{
            tempnam($root, $case[1]);
        }}
    }} catch (Error $e) {{
        echo $e->getMessage(), "\n";
    }}
}}
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert_eq!(
        execution.stdout,
        concat!(
            "tempnam(): Argument #1 ($directory) must not contain any null bytes\n",
            "tempnam(): Argument #2 ($prefix) must not contain any null bytes\n",
            "tempnam(): Argument #1 ($directory) must be of type string, array given\n",
            "tempnam(): Argument #2 ($prefix) must be of type string, array given\n",
        )
    );
}

#[test]
fn tempnam_reports_system_temp_fallback_before_open_basedir_denial() {
    let source = r#"<?php
ini_set("open_basedir", ".");
var_dump(tempnam("missing-tempnam-directory", "prefix_"));
"#;

    let execution = run_source(source).unwrap();

    let notice = execution
        .stdout
        .find("Notice: tempnam(): file created in the system's temporary directory")
        .expect("tempnam fallback notice is emitted");
    let warning = execution
        .stdout
        .find("Warning: tempnam(): open_basedir restriction in effect.")
        .expect("tempnam open_basedir warning is emitted");
    assert!(
        notice < warning,
        "fallback notice must precede open_basedir denial:\n{}",
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
