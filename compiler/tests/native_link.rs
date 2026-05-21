use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use php_compiler::{codegen::emit_native_executable_c_source, parse};

#[test]
fn native_executable_c_source_routes_direct_strings_through_runtime_helpers() {
    let program = parse("<?php\necho \"native link\\n\";\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_native_string_from_bytes"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_string_with_diagnostic"),
        "{source}"
    );
    assert!(source.contains("phpc_native_value_echo_stdout"), "{source}");
    assert!(!source.contains("printf(\"%s\", \"native link"), "{source}");
}

#[test]
fn native_executable_c_source_routes_comparison_families_through_runtime_contract() {
    let program = parse(
        r#"<?php
echo 1 == "1", "\n";
echo 1 != "2", "\n";
echo 2 < "10", "\n";
echo 2 <= "2", "\n";
echo "10" > 2, "\n";
echo "alpha" >= "alpha", "\n";
echo 2 === 2, "\n";
echo null == false, "\n";
echo 1 !== "1";
"#,
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("#include <stdbool.h>"), "{source}");
    assert!(source.contains("phpc_native_value_from_scalar"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_materialization_failure_exit_code"),
        "generated C should route comparison operand materialization failures through the runtime ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_compare_branch_and_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonBranchResult"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_result_exit_code"),
        "generated C should route blocked branch decisions through the exit-code accessor:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_result_is_true"),
        "generated C should route branch truth consumption through the truth accessor:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_status"),
        "generated C should not open-code branch status handling:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_value"),
        "generated C should not open-code branch value handling:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_diagnostic_len"),
        "generated C should not need diagnostic-length access after branch ABI reporting:\n{source}"
    );
    assert!(
        !source.contains(".status != 0"),
        "generated C should not inspect branch status fields directly:\n{source}"
    );
    assert!(
        !source.contains(".value != 0"),
        "generated C should not inspect branch value fields directly:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_compare_and_free("),
        "generated C should consume owned comparison values through the branch-returning boundary:\n{source}"
    );
    assert!(
        !source.contains("comparison_string_handle_"),
        "generated C should not allocate comparison-only string handles:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_from_string(comparison_string_handle_"),
        "generated C should materialize comparison strings through the raw byte boundary:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_result_branch_or_report_stderr_and_free"),
        "generated C should not compose result branch consumption outside the owned branch boundary:\n{source}"
    );
    assert!(
        !source.contains("if (comparison_value_handle_"),
        "generated C should not open-code comparison operand handle null checks:\n{source}"
    );
    assert!(
        !source.contains("((1) =="),
        "loose equality should not lower as a C scalar comparison:\n{source}"
    );
}

const DYNAMIC_BINARY_STRING_COMPARISON_SOURCE: &str = r#"<?php
$flag = 1 < 2;
$left = $flag ? "2\x00z" : "10\x00w";
$right = $flag ? "2\x00g" : "10\x00a";
echo ($left > $right) ? 1 : 0, "\n";
echo ($right < $left) ? 1 : 0, "\n";
echo ($left != "2\x00a") ? 1 : 0, "\n";
echo ($left == "2\x00z") ? 1 : 0;
"#;

#[test]
fn native_executable_c_source_tracks_dynamic_string_operand_lengths() {
    let program = parse(DYNAMIC_BINARY_STRING_COMPARISON_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic((const uint8_t *)("),
        "dynamic string comparison operands should materialize through pointer-plus-length value construction:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_compare_branch_and_free"),
        "dynamic string operands should feed the shared owned comparison branch ABI:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)("),
        "tracked dynamic PHP string lengths should avoid C strlen so embedded NUL bytes remain data:\n{source}"
    );
    assert!(
        !source.contains("strcmp("),
        "dynamic string comparisons should stay on the runtime PHP comparison ABI:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_direct_string_runtime_helper_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let fixture =
        workspace_root.join("tests/fixtures/milestone2300/native_link_runtime_helper.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let output_path = native_link_output_path("direct_string_runtime_helper");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            &relative_fixture,
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));
    let expected = strip_fixture_editor_newline(
        fs::read_to_string(
            workspace_root.join("tests/fixtures/milestone2300/native_link_runtime_helper.stdout"),
        )
        .expect("expected native stdout fixture is readable"),
    );

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_dynamic_binary_string_comparison_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("dynamic_binary_string_comparison").with_extension("php");
    fs::write(&temp_php, DYNAMIC_BINARY_STRING_COMPARISON_SOURCE)
        .expect("write temporary dynamic binary string comparison source");
    let output_path = native_link_output_path("dynamic_binary_string_comparison");
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
            panic!("failed to compile dynamic binary string comparison executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run dynamic binary string comparison executable: {error}")
    });

    assert!(
        run.status.success(),
        "dynamic binary string comparison executable failed"
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1\n1\n1\n1");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_runtime_comparison_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("runtime_comparison").with_extension("php");
    fs::write(
        &temp_php,
        r#"<?php
echo 1 == "1", "\n";
echo 1 != "2", "\n";
echo 2 < "10", "\n";
echo 2 <= "2", "\n";
echo "10" > 2, "\n";
echo "alpha" >= "alpha", "\n";
echo 2 === 2, "\n";
echo null == false, "\n";
echo 1 !== "1";
"#,
    )
    .expect("write temporary comparison source");
    let output_path = native_link_output_path("runtime_comparison");
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
        .unwrap_or_else(|error| panic!("failed to compile native comparison executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native comparison executable: {error}"));

    assert!(run.status.success(), "native comparison executable failed");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "1\n1\n1\n1\n1\n1\n1\n1\n1"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_uses_runtime_comparison_results_as_branch_conditions() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("runtime_comparison_branch").with_extension("php");
    fs::write(
        &temp_php,
        r#"<?php
echo ("10" > 2) ? 1 : 0, "\n";
echo (1 != "2") ? 1 : 0, "\n";
echo (2 < "10") ? 1 : 0, "\n";
echo (2 <= "2") ? 1 : 0, "\n";
echo ("alpha" >= "alpha") ? 1 : 0, "\n";
echo (null == false) ? 1 : 0, "\n";
echo (1 !== "1") ? 1 : 0, "\n";
echo (2 === 2) ? 1 : 0;
"#,
    )
    .expect("write temporary comparison branch source");
    let output_path = native_link_output_path("runtime_comparison_branch");
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
            panic!("failed to compile native comparison branch executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native comparison branch executable: {error}")
    });

    assert!(
        run.status.success(),
        "native comparison branch executable failed"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "1\n1\n1\n1\n1\n1\n1\n1"
    );
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

fn strip_fixture_editor_newline(mut value: String) -> String {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    value
}
