use std::fs;
use std::path::PathBuf;
use std::process::Command;

use php_compiler::{codegen::emit_native_executable_c_source, parse};

const DYNAMIC_STRING_COMPARISON_SOURCE: &str = r#"<?php
echo ((extension_loaded("Json") ? "10" : "20") > 2), "\n";
echo (2 < (extension_loaded("Json") ? "10" : "20")), "\n";
echo ((extension_loaded("Json") ? "10" : "20") !== 10), "\n";
echo ((extension_loaded("Json") ? "02" : "12") == 2);
"#;

const MIXED_LENGTH_DYNAMIC_STRING_COMPARISON_SOURCE: &str = r#"<?php
echo ((extension_loaded("Json") ? "10" : "100") > (extension_loaded("Json") ? "2" : "20")), "\n";
echo ((extension_loaded("Json") ? "2" : "20") < (extension_loaded("Json") ? "10" : "100")), "\n";
echo ((extension_loaded("Json") ? "10" : "100") !== 10), "\n";
echo ((extension_loaded("Json") ? "2" : "20") == 2);
"#;

#[test]
fn native_executable_c_source_materializes_known_length_string_expr_comparisons() {
    let program = parse(DYNAMIC_STRING_COMPARISON_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic((const uint8_t *)("),
        "dynamic string comparison operands should materialize string bytes through the native value comparison ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_compare_result")
            && source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "dynamic string comparison operands should feed the runtime value comparison and diagnostic output ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_operand_compare_operation_branch_and_free")
            && !source.contains("phpc_native_comparison_branch_decision_from_result"),
        "dynamic string comparison output should not materialize intermediate branch results:\n{source}"
    );
    assert!(
        !source.contains("comparison_string_handle_"),
        "dynamic string comparison operands should not allocate comparison-only string handles:\n{source}"
    );
    assert!(
        !source.contains("comparison_diagnostic_handle_"),
        "dynamic string comparison operands should keep diagnostics inside the operand ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_compare_branch_and_free"),
        "dynamic string comparison operands should not use the older branch comparison ABI:\n{source}"
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
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic((const uint8_t *)("),
        "mixed-length dynamic string comparison operands should materialize string bytes through the native value comparison ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_compare_result")
            && source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "mixed-length dynamic string comparison operands should feed the runtime value comparison and diagnostic output ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_operand_compare_operation_branch_and_free")
            && !source.contains("phpc_native_comparison_branch_decision_from_result"),
        "mixed-length dynamic string comparison output should not materialize intermediate branch results:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_compare_branch_and_free"),
        "mixed-length dynamic string comparison operands should not use the older branch comparison ABI:\n{source}"
    );
    assert!(
        !source.contains("comparison_string_handle_"),
        "mixed-length dynamic string comparison operands should not allocate comparison-only string handles:\n{source}"
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
