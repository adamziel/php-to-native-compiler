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
fn native_executable_c_source_routes_strlen_through_string_conversion_result() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\necho strlen(42);\necho strlen(false);\necho strlen(null);\necho strlen($payload);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeStringConversionResult"),
        "{source}"
    );
    assert!(source.contains("phpc_NativeByteBuffer"), "{source}");
    assert!(source.contains("phpc_native_value_from_scalar"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_to_string_bytes(")
            .count(),
        4,
        "{source}"
    );
    assert_eq!(
        source
            .matches("  phpc_native_string_conversion_result_free(")
            .count(),
        4,
        "{source}"
    );
    assert!(
        source.contains(".bytes.len"),
        "strlen should use runtime conversion byte lengths:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)"),
        "generated C should not use C strlen for PHP strlen operands:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_predicates_through_runtime_contract() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\necho str_starts_with($payload, \"A\0\");\necho str_ends_with($payload, \"\0B\");\necho str_contains(42, \"2\");\necho str_contains($payload, \"C\");\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_predicate_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_string_predicate_with_diagnostic(")
            .count(),
        4,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_predicate_diagnostic_")
            .count(),
        4,
        "{source}"
    );
    assert!(
        source.contains("static const uint8_t phpc_native_value_bytes_"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar"),
        "scalar operands should be admitted through the native value boundary:\n{source}"
    );
    assert!(
        !source.contains("strncmp(")
            && !source.contains("strstr(")
            && !source.contains("strlen((const char *)"),
        "string predicates should not use C string APIs for PHP byte semantics:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_int_builtins_through_runtime_contract() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\n$repeated = \"A\0BA\0B\";\necho strcasecmp($payload, \"a\0b\");\necho substr_count($repeated, $payload, 0, 6);\necho substr_count(42042, 42);\necho ord($payload);\necho ord(42042);\necho crc32(\"123456789\");\necho crc32($payload);\necho crc32(null);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_int_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_string_int_operation_with_diagnostic(")
            .count(),
        8,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_int_diagnostic_")
            .count(),
        8,
        "{source}"
    );
    assert!(
        source.contains(", 0, &string_int_diagnostic_")
            && source.contains(", 1, &string_int_diagnostic_")
            && source.contains(", 5, &string_int_diagnostic_")
            && source.contains(", 6, &string_int_diagnostic_"),
        "case compare, substring count, ord, and crc32 should share the tagged string-int ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        2,
        "substr_count offset and length should share the native int conversion ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)"),
        "string-int builtins should use PHP value-to-string byte conversion:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_distance_builtins_through_runtime_contract() {
    let program = parse(
        "<?php\n$left = \"kitten\";\n$right = \"sitting\";\n$insert = 1;\n$replace = 2;\n$delete = 1;\necho levenshtein($left, $right);\necho levenshtein(\"A\0B\", \"A\0C\", $insert, $replace, $delete);\necho similar_text(42042, 42);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_distance_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_string_distance_operation_with_diagnostic(")
            .count(),
        3,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_distance_diagnostic_")
            .count(),
        3,
        "{source}"
    );
    assert!(
        source.contains(", 0, &string_distance_diagnostic_")
            && source.contains(", 1, &string_distance_diagnostic_"),
        "levenshtein and similar_text should share the tagged string-distance ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        3,
        "levenshtein costs should share the native int conversion ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)"),
        "string-distance builtins should use PHP value-to-string byte conversion:\n{source}"
    );
}

const FILESYSTEM_PATH_OPERATION_SOURCE: &str = "<?php\n$path = \"pmt/\\0A\";\n$flag = str_contains($path, \"\\0\");\nfile_get_contents($path, $flag);\nrealpath($path);\nfile_exists(42);\nis_writable($path);\nfilesize($path);\nfilemtime($path);\ngetcwd();\nclearstatcache($flag, $path);\necho \"done\\n\";\n";

#[test]
fn native_executable_c_source_routes_filesystem_path_builtins_through_shared_blocker() {
    let program = parse(FILESYSTEM_PATH_OPERATION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_filesystem_path_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_filesystem_path_operation_with_diagnostic(")
            .count(),
        8,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_predicate_with_diagnostic"),
        "filesystem optional flags should compose with the existing truthy value producer:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "filesystem path operands should enter the same native value boundary for scalar and string families:\n{source}"
    );
    assert!(
        source.contains(", 0, &filesystem_path_operation_diagnostic_")
            && source.contains(", 1, &filesystem_path_operation_diagnostic_")
            && source.contains(", 2, &filesystem_path_operation_diagnostic_")
            && source.contains(", 6, &filesystem_path_operation_diagnostic_")
            && source.contains(", 8, &filesystem_path_operation_diagnostic_")
            && source.contains(", 9, &filesystem_path_operation_diagnostic_")
            && source.contains(", 10, &filesystem_path_operation_diagnostic_")
            && source.contains(", 11, &filesystem_path_operation_diagnostic_"),
        "filesystem path builtins should share one operation-tagged ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_integer_arguments_through_value_conversion() {
    let program = parse(
        "<?php\n$offset = \"0\";\n$length = 4.0;\n$insert = true;\n$replace = \"1\";\n$delete = 1.0;\necho substr_count(\"aaaa\", \"aa\", $offset, $length);\necho levenshtein(\"kitten\", \"sitting\", $insert, $replace, $delete);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_to_int64_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        5,
        "substr_count offset/length and levenshtein costs should share the same int conversion ABI:\n{source}"
    );
    assert!(
        source.contains(", 0, &int_conversion_diagnostic_")
            && source.contains(", 1, &int_conversion_diagnostic_")
            && source.contains(", 2, &int_conversion_diagnostic_"),
        "string offset, string length, and string distance cost roles should use operation tags:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_int_operation_with_diagnostic")
            && source.contains("phpc_native_value_string_distance_operation_with_diagnostic"),
        "converted int arguments should compose with both string-int and string-distance consumers:\n{source}"
    );
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
    assert!(
        source.contains("phpc_native_comparison_operand_from_scalar"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operand_from_string_bytes"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operation_from_opcode"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operand_compare_operation_decision_and_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonOperation"),
        "{source}"
    );
    assert!(source.contains("phpc_NativeComparisonOperand"), "{source}");
    assert!(
        !source.contains("phpc_NativeComparisonBranchResult"),
        "scalar/string comparison branches should not expose intermediate branch-result storage:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonBranchDecision"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_from_result"),
        "scalar/string comparisons should consume operands through the direct decision ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_decision_exit_code"),
        "generated C should consume branch exits through the runtime branch-decision ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_decision_is_true"),
        "generated C should consume branch truth through the runtime branch-decision ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_NativeComparisonResult"),
        "generated C should consume comparison operands directly through the decision ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_operand_compare_operation_branch_and_free"),
        "scalar/string comparisons should not materialize an intermediate branch result:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_materialization_failure_exit_code"),
        "generated C should not open-code comparison operand materialization failure checks:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_compare_and_free"),
        "generated C should consume operands through the comparison operand boundary:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_compare_branch_and_free"),
        "generated C should consume operands through the comparison operand boundary:\n{source}"
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
        !source.contains("phpc_native_comparison_branch_result_exit_code"),
        "generated C should not consume branch exits through raw branch-result accessors:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_is_true"),
        "generated C should not consume branch truth through raw branch-result accessors:\n{source}"
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
        !source.contains("comparison_diagnostic_handle_"),
        "generated C should not carry comparison operand diagnostics outside the operand ABI:\n{source}"
    );
    assert!(
        !source.contains("((1) =="),
        "loose equality should not lower as a C scalar comparison:\n{source}"
    );
}

const ARRAY_HANDLE_COMPARISON_SOURCE: &str = "<?php\n$left = [1, \"two\" => 2];\n$right = [1, \"two\" => 2];\necho ($left === $right), \"\\n\";\necho ([1, \"two\" => 2] !== [1, \"two\" => 3]), \"\\n\";\necho ([1] == [1]), \"\\n\";\necho ([2] > [1]), \"\\n\";\n";

#[test]
fn native_executable_c_source_routes_array_handle_comparisons_through_runtime_branch() {
    let program = parse(ARRAY_HANDLE_COMPARISON_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_NativeArrayHandle"), "{source}");
    assert!(
        source
            .contains("extern phpc_NativeComparisonBranchResult phpc_native_array_compare_branch"),
        "generated C should declare the shared array comparison branch ABI:\n{source}"
    );
    assert_eq!(
        source.matches(" = phpc_native_array_compare_branch(").count(),
        4,
        "strict, loose-equality, and ordering array comparisons should share the array branch ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonBranchDecision")
            && source.contains("phpc_native_comparison_branch_decision_from_result")
            && source.contains("phpc_native_comparison_branch_decision_exit_code")
            && source.contains("phpc_native_comparison_branch_decision_is_true"),
        "array comparison results should use the common branch-decision ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_exit_code"),
        "array comparison results should not use raw branch-result exit accessors:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_is_true"),
        "array comparison results should not use raw branch-result truth accessors:\n{source}"
    );
    assert!(
        !source.contains(" = phpc_native_comparison_operand_compare_operation_branch_and_free("),
        "array handles should not pass through scalar/string comparison operands:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_compare_branch_and_free("),
        "generated C should keep array handle ownership with the existing cleanup list:\n{source}"
    );
    assert!(
        source.contains("phpc_native_array_free(array_"),
        "array comparison should preserve existing generated-C array cleanup:\n{source}"
    );
}

const ARRAY_HANDLE_STRICT_COMPARISON_SOURCE: &str = "<?php\n$left = [1, \"two\" => 2];\n$right = [1, \"two\" => 2];\n$different = [1, \"two\" => 3];\necho ($left === $right), \"\\n\";\necho ([1, \"two\" => 2] !== $different), \"\\n\";\n";

#[test]
fn emit_exe_links_and_runs_array_handle_strict_comparison_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("array_handle_strict_comparison").with_extension("php");
    fs::write(&temp_php, ARRAY_HANDLE_STRICT_COMPARISON_SOURCE)
        .expect("write native array-handle comparison fixture");
    let output_path = native_link_output_path("array_handle_strict_comparison");
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
            panic!("failed to compile native array comparison executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native array comparison executable: {error}")
    });

    assert!(
        run.status.success(),
        "native array comparison executable failed"
    );
    assert_eq!(run.stdout, b"1\n1\n");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
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
        source.contains("phpc_native_comparison_operand_from_string_bytes((const uint8_t *)("),
        "dynamic string comparison operands should materialize through pointer-plus-length value construction:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operation_from_opcode")
            && source
                .contains("phpc_native_comparison_operand_compare_operation_decision_and_free"),
        "dynamic string operands should feed the shared comparison operand-decision ABI:\n{source}"
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
fn emit_exe_links_and_runs_strlen_conversion_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("strlen_conversion");
    let source_path = native_link_output_path("strlen_conversion_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$payload = \"A\0B\";\necho strlen(42);\necho strlen(false);\necho strlen(null);\necho strlen($payload);\necho \"\\n\";\n",
    )
    .expect("native strlen conversion source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native strlen source path is valid UTF-8"),
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

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "2003\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_predicate_conversion_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_predicate_conversion");
    let source_path = native_link_output_path("string_predicate_conversion_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$payload = \"A\0B\";\necho (str_starts_with($payload, \"A\0\") ? 1 : 0);\necho (str_ends_with($payload, \"\0B\") ? 1 : 0);\necho (str_contains(42, \"2\") ? 1 : 0);\necho (str_contains($payload, \"\") ? 1 : 0);\necho (str_contains($payload, \"C\") ? 1 : 0);\necho \"\\n\";\n",
    )
    .expect("native string predicate source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string predicate source path is valid UTF-8"),
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

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "11110\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_int_operation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_int_operation");
    let source_path = native_link_output_path("string_int_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$payload = \"A\0B\";\n$repeated = \"A\0BA\0B\";\necho strcasecmp($payload, \"a\0b\");\necho \"\\n\";\necho substr_count($repeated, $payload, 0, 6);\necho \"\\n\";\necho substr_count(42042, 42);\necho \"\\n\";\necho ord($payload);\necho \"\\n\";\necho ord(42042);\necho \"\\n\";\necho crc32(\"123456789\");\necho \"\\n\";\necho crc32($payload);\necho \"\\n\";\necho crc32(null);\necho \"\\n\";\n",
    )
    .expect("native string-int source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-int source path is valid UTF-8"),
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

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "0\n2\n2\n65\n52\n3421780262\n382410329\n0\n"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_distance_operation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_distance_operation");
    let source_path = native_link_output_path("string_distance_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$left = \"kitten\";\n$right = \"sitting\";\n$insert = 1;\n$replace = 2;\n$delete = 1;\necho levenshtein($left, $right);\necho \"\\n\";\necho levenshtein(\"A\0B\", \"A\0C\", $insert, $replace, $delete);\necho \"\\n\";\necho similar_text(42042, 42);\necho \"\\n\";\n",
    )
    .expect("native string-distance source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-distance source path is valid UTF-8"),
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

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "3\n2\n2\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_reports_shared_filesystem_path_blocker_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("filesystem_path_operation");
    let source_path = native_link_output_path("filesystem_path_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, FILESYSTEM_PATH_OPERATION_SOURCE)
        .expect("native filesystem path source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native filesystem path source path is valid UTF-8"),
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

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "done\n");
    let stderr = String::from_utf8_lossy(&run.stderr);
    for expected in [
        "file_get_contents() awaits the shared filesystem stream ABI",
        "realpath() awaits the shared filesystem canonicalization ABI",
        "file_exists() awaits the shared filesystem stat ABI",
        "is_writable() awaits the shared filesystem stat ABI",
        "filesize() awaits the shared filesystem stat-value ABI",
        "filemtime() awaits the shared filesystem stat-value ABI",
        "getcwd() awaits the shared process current-directory ABI",
        "clearstatcache() awaits the shared filesystem stat-cache ABI",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in {stderr:?}"
        );
    }

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_integer_argument_conversion_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_integer_argument_conversion");
    let source_path = native_link_output_path("string_integer_argument_conversion_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$offset = \"0\";\n$length = 4.0;\n$insert = true;\n$replace = \"1\";\n$delete = 1.0;\necho substr_count(\"aaaa\", \"aa\", $offset, $length);\necho \"\\n\";\necho levenshtein(\"kitten\", \"sitting\", $insert, $replace, $delete);\necho \"\\n\";\n",
    )
    .expect("native int conversion source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native int conversion source path is valid UTF-8"),
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

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "2\n3\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
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

const GENERALIZED_ARRAY_KEY_SOURCE: &str = "<?php\n$slot = \"slot\";\n$two = 2;\n$numeric = \"3\";\n$nil = null;\n$binary = \"A\0B\";\n$a = [$slot => \"text\", $two => \"two\", $numeric => \"three\", $nil => \"null-key\", $binary => \"bin\0ary\", false => \"false-key\", true => \"true-key\", 4.0 => \"float-key\"];\necho $a[$slot], \"\\n\";\necho $a[2], \"\\n\";\necho $a[\"3\"], \"\\n\";\necho $a[$nil], \"\\n\";\necho $a[$binary], \"\\n\";\necho $a[false], \"\\n\";\necho $a[true], \"\\n\";\necho $a[4.0], \"\\n\";\n$a[$slot] = \"updated\";\n$a[$two] = \"two-updated\";\necho $a[\"slot\"], \"\\n\";\necho $a[2], \"\\n\";\n";

const NATIVE_ARRAY_APPEND_SOURCE: &str = "<?php\n$a = [1, \"two\", (string)(2 + 1), null];\necho $a[0], \"|\", $a[1], \"|\", $a[2], \"|\", $a[3];\n";

const NATIVE_VALUE_OPERATION_ARRAY_SOURCE: &str = "<?php\n$a = [];\n$a[\"s\" . \"lot\"] = (2 + 3) * (5 - 1);\n$a[(1 << 2) + 1] = \"fi\" . \"ve\";\n$a[\"neg\"] = -(\"6\" - 2);\necho $a[\"slot\"], \"|\", $a[5], \"|\", $a[\"neg\"];\n";

const NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE: &str = "<?php\n$a = [];\n$a[(int)\"5\"] = (string)((2 + 3) > 4);\n$a[(int)(3 <= 2)] = get_debug_type((string)123);\n$a[(float)\"3\"] = gettype((float)\"3.5\");\necho $a[5], \"|\", $a[0], \"|\", $a[3];\n";

#[test]
fn native_executable_c_source_routes_array_key_and_value_expressions_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_OPERATION_ARRAY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeValueOperationResult"),
        "{source}"
    );
    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_binary_result"),
        "{source}"
    );
    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_unary_result"),
        "{source}"
    );
    for op in [
        "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        "PHPC_NATIVE_VALUE_BINARY_ADD",
        "PHPC_NATIVE_VALUE_BINARY_MUL",
        "PHPC_NATIVE_VALUE_BINARY_SUB",
        "PHPC_NATIVE_VALUE_BINARY_SHIFT_LEFT",
        "PHPC_NATIVE_VALUE_UNARY_NEGATE",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }
    assert!(
        source
            .matches(" = phpc_native_value_binary_result(")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_unary_result("),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_operation_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key")
            && source.contains("phpc_native_array_insert_key_value_with_diagnostic"),
        "value operation results should feed the existing array key/value boundaries:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_appends_through_diagnostic_boundary() {
    let program = parse(NATIVE_ARRAY_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern bool phpc_native_array_append_value_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeValueHandle value, phpc_NativeDiagnosticHandle *diagnostic);"
        ),
        "{source}"
    );
    assert!(
        source
            .matches("phpc_native_array_append_value_with_diagnostic(")
            .count()
            >= 5,
        "declaration plus every appended value should use the diagnostic append ABI:\n{source}"
    );
    assert!(
        source.contains("array_append_diagnostic_")
            && source.contains("phpc_native_diagnostic_message_stderr(array_append_diagnostic_"),
        "append diagnostics should be reported through the shared diagnostic boundary:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_array_append_diagnostic_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_array_append_diagnostic").with_extension("php");
    fs::write(&temp_php, NATIVE_ARRAY_APPEND_SOURCE)
        .expect("write native array append diagnostic fixture");
    let output_path = native_link_output_path("native_array_append_diagnostic");
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
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|two|3|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_value_result_array_key_and_value_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_value_result_array_key_value").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_OPERATION_ARRAY_SOURCE)
        .expect("write native value-result array key/value fixture");
    let output_path = native_link_output_path("native_value_result_array_key_value");
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
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"20|five|-4");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn native_executable_c_source_routes_compare_cast_and_type_name_results_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_compare_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_cast_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_type_name_result",
    ] {
        assert!(source.contains(declaration), "{declaration}\n\n{source}");
    }

    for op in [
        "PHPC_NATIVE_VALUE_COMPARISON_GT",
        "PHPC_NATIVE_VALUE_COMPARISON_LE",
        "PHPC_NATIVE_VALUE_CAST_STRING",
        "PHPC_NATIVE_VALUE_CAST_INT",
        "PHPC_NATIVE_VALUE_CAST_FLOAT",
        "PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE",
        "PHPC_NATIVE_VALUE_TYPE_NAME_DEBUG",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }

    assert!(
        source
            .matches(" = phpc_native_value_compare_result(")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_cast_result(").count() >= 6,
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_type_name_result(")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key")
            && source.contains("phpc_native_array_insert_key_value_with_diagnostic"),
        "compare/cast/type-name results should feed the existing array key/value boundaries:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_compare_cast_type_name_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_compare_cast_type_name_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE)
        .expect("write native compare/cast/type-name value-result fixture");
    let output_path = native_link_output_path("native_compare_cast_type_name_result");
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
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|string|double");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn native_executable_c_source_routes_array_keys_through_runtime_materialization() {
    let program = parse(GENERALIZED_ARRAY_KEY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeArrayKeyMaterializationResult"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_insert_key_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_read_key_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_key_materialization_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "array keys should enter the same native value materialization boundary for scalar and string families:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_read_int("),
        "indexed reads should not bypass generalized key materialization:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_generalized_array_key_materialization_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("generalized_array_key_materialization").with_extension("php");
    fs::write(&temp_php, GENERALIZED_ARRAY_KEY_SOURCE)
        .expect("write generalized native array-key fixture");
    let output_path = native_link_output_path("generalized_array_key_materialization");
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
            panic!("failed to compile generalized array-key executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run generalized array-key executable: {error}"));

    assert!(run.status.success(), "native array-key executable failed");
    assert_eq!(
        run.stdout,
        b"text\ntwo\nthree\nnull-key\nbin\0ary\nfalse-key\ntrue-key\nfloat-key\nupdated\ntwo-updated\n"
    );
    assert_eq!(run.stderr, b"");

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
