use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::{emit_ir_source, run_source};

fn temp_md5_path(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("phpc-md5-file-{label}-{nanos}.txt"))
        .display()
        .to_string()
}

#[test]
fn md5_file_hashes_local_file_payloads() {
    let path = temp_md5_path("payload");
    let source = format!(
        r#"<?php
$file = "{}";
file_put_contents($file, "abc");
var_dump(md5_file($file));
echo bin2hex(md5_file($file, true)), "\n";
"#,
        path
    );

    let execution = run_source(&source).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "string(32) \"900150983cd24fb0d6963f7d28e17f72\"\n\
900150983cd24fb0d6963f7d28e17f72\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn md5_file_empty_paths_and_missing_files_match_file_hash_shape() {
    let missing = temp_md5_path("missing");
    let source = format!(
        r#"<?php
try {{
    md5_file("");
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}
var_dump(md5_file("{}"));
"#,
        missing
    );

    let execution = run_source(&source).unwrap();

    assert!(execution
        .stdout
        .starts_with("Path must not be empty\n\nWarning: md5_file("));
    assert!(execution
        .stdout
        .contains("): Failed to open stream: No such file or directory"));
    assert!(execution.stdout.ends_with("bool(false)\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn md5_file_null_paths_are_deprecated_then_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
try {
    md5_file(null);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Deprecated: md5_file(): Passing null to parameter #1 ($filename) of type string is deprecated in Command line code on line 3\n\
Path must not be empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn md5_file_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["md5", "md5_file"] as $name) {
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
fn emit_ir_folds_md5_file_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("md5_file") ? "1" : "0";
echo is_callable("md5_file") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
