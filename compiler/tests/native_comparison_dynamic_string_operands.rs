use std::fs;
use std::path::PathBuf;
use std::process::Command;

use php_compiler::{codegen::emit_native_executable_c_source, parse};

const DYNAMIC_STRING_COMPARISON_SOURCE: &str = r#"<?php
$left = (1 < 2) ? "10" : "20";
$right = (1 < 2) ? "02" : "12";
echo ($left > 2) ? 1 : 0, "\n";
echo (2 < $left) ? 1 : 0, "\n";
echo ($left !== 10) ? 1 : 0, "\n";
echo ($right == 2) ? 1 : 0;
"#;

const MIXED_LENGTH_DYNAMIC_STRING_COMPARISON_SOURCE: &str = r#"<?php
$left = (1 < 2) ? "10" : "100";
$right = (1 < 2) ? "2" : "20";
echo ($left > $right) ? 1 : 0, "\n";
echo ($right < $left) ? 1 : 0, "\n";
echo ($left !== 10) ? 1 : 0, "\n";
echo ($right == 2) ? 1 : 0;
"#;

#[test]
fn native_executable_c_source_materializes_known_length_string_expr_comparisons() {
    let program = parse(DYNAMIC_STRING_COMPARISON_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic((const uint8_t *)((("),
        "dynamic string comparison operands should materialize through the native value byte-string ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_comparison_with_diagnostic")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_GT")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_LT")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_EQ"),
        "dynamic string comparison operands should feed the shared native value comparison ABI:\n{source}"
    );
    assert!(
        source.contains("\"10\") : (\"20\"))), 2")
            && source.contains("\"02\") : (\"12\"))), 2"),
        "known-length dynamic string comparison operands should carry explicit byte lengths:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)("),
        "known-length string expression comparison operands should use tracked byte lengths instead of C string length:\n{source}"
    );
    assert!(
        !source.contains("strcmp("),
        "native executable comparisons should not fall back to C string comparison:\n{source}"
    );
}

#[test]
fn native_executable_c_source_materializes_mixed_length_string_expr_comparisons() {
    let program = parse(MIXED_LENGTH_DYNAMIC_STRING_COMPARISON_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        !source.contains("strlen((const char *)("),
        "mixed-length string expression comparison operands should use tracked byte lengths instead of C string length:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic((const uint8_t *)((("),
        "mixed-length dynamic string comparison operands should materialize through the native value byte-string ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_comparison_with_diagnostic")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_GT")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_LT")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_EQ"),
        "mixed-length dynamic string comparison operands should feed the shared native value comparison ABI:\n{source}"
    );
    assert!(
        source.contains("? (2) : (3)") && source.contains("? (1) : (2)"),
        "mixed-length dynamic string comparison operands should carry conditional byte lengths:\n{source}"
    );
    assert!(
        !source.contains("strcmp("),
        "native executable comparisons should not fall back to C string comparison:\n{source}"
    );
}

#[test]
fn emit_exe_runs_known_length_string_expr_comparisons_through_runtime() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("dynamic_string_comparison").with_extension("php");
    fs::write(&temp_php, DYNAMIC_STRING_COMPARISON_SOURCE)
        .expect("write temporary dynamic string comparison source");
    let output_path = native_link_output_path("dynamic_string_comparison");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile dynamic string comparison executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run dynamic string comparison executable: {error}")
    });

    assert!(run.status.success(), "native comparison executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1\n1\n1\n1");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_runs_mixed_length_string_expr_comparisons_through_runtime() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("mixed_length_string_comparison").with_extension("php");
    fs::write(&temp_php, MIXED_LENGTH_DYNAMIC_STRING_COMPARISON_SOURCE)
        .expect("write temporary mixed-length dynamic string comparison source");
    let output_path = native_link_output_path("mixed_length_string_comparison");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile mixed-length dynamic string comparison executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run mixed-length dynamic string comparison executable: {error}")
    });

    assert!(
        run.status.success(),
        "mixed-length native comparison executable failed"
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1\n1\n1\n1");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

fn has_cc() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}

fn native_link_output_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("phpc-native-link-{name}-{}", std::process::id()));
    path
}
