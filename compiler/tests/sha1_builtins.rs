use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::{emit_ir_source, run_source};

fn temp_sha1_path() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("phpc-sha1-{nanos}.txt"))
        .display()
        .to_string()
}

#[test]
fn sha1_matches_core_vectors_and_raw_output() {
    let execution = run_source(
        r#"<?php
var_dump(sha1(""));
var_dump(sha1("a"));
var_dump(sha1("abc"));
var_dump(sha1("message digest"));
echo bin2hex(sha1("abc", true)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\n\
string(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\n\
string(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\n\
string(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\n\
a9993e364706816aba3e25717850c26c9cd0d89d\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sha1_file_hashes_local_file_payloads() {
    let path = temp_sha1_path();
    let source = format!(
        r#"<?php
$file = "{}";
file_put_contents($file, "abc");
var_dump(sha1_file($file));
echo bin2hex(sha1_file($file, true)), "\n";
"#,
        path
    );

    let execution = run_source(&source).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "string(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\n\
a9993e364706816aba3e25717850c26c9cd0d89d\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sha1_file_empty_paths_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
try {
    sha1_file("");
} catch (ValueError $e) {
    echo $e->getMessage(), "|";
}
try {
    sha1_file(null);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Path must not be empty|\n\
Deprecated: sha1_file(): Passing null to parameter #1 ($filename) of type string is deprecated in Command line code on line 8\n\
Path must not be empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sha1_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["sha1", "sha1_file"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/2;11:1/2;");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_sha1_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("sha1") ? "1" : "0";
echo is_callable("sha1_file") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
