use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use php_compiler::{codegen::emit_native_executable_c_source, parse};

#[test]
fn native_executable_c_source_routes_direct_strings_and_scalars_through_runtime_helpers() {
    let program = parse(
        "<?php\necho \"native link\\n\";\nprint \"runtime string\";\necho 42;\nprint true;\necho 1.25;\necho false;\necho null;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_native_string_from_bytes"), "{source}");
    assert!(source.contains("phpc_native_value_from_scalar"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_string_with_diagnostic"),
        "{source}"
    );
    assert!(source.contains("phpc_native_value_echo_stdout"), "{source}");
    assert_eq!(
        source
            .matches("phpc_native_value_from_scalar(scalar_")
            .count(),
        5,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_native_value_echo_stdout(value_")
            .count(),
        7,
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\", \"native link"), "{source}");
    assert!(!source.contains("printf(\"%lld\""), "{source}");
    assert!(!source.contains("printf(\"%g\""), "{source}");
    assert!(!source.contains("printf(\"%s\", \"1\")"), "{source}");
}

#[test]
fn native_executable_c_source_reports_owned_diagnostics_through_shared_consumer() {
    let program = parse("<?php\necho \"left\";\n$s = \"AB\";\n$s[0] = \"Z\";\necho $s;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern size_t phpc_native_diagnostic_report"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_report(diagnostic_"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_report(string_offset_write_diagnostic_"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_message_stderr(diagnostic_"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_free(diagnostic_"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_message_stderr(string_offset_write_diagnostic_"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_scalar_runtime_value_echo_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("scalar_runtime_value_echo.php");
    let output_path = native_link_output_path("scalar_runtime_value_echo");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\necho 42;\nprint true;\necho 1.5;\necho false;\necho null;\n",
    )
    .expect("write scalar native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native scalar source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"4211.5");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
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
        "<?php\n$payload = \"A\0B\";\n$repeated = \"A\0BA\0B\";\necho strcasecmp($payload, \"a\0b\");\necho strcmp($payload, \"a\0b\");\necho strncmp($payload, \"A\0C\", 3);\necho strncasecmp($payload, \"a\0c\", \"2\");\necho substr_count($repeated, $payload, 0, 6);\necho substr_count(42042, 42);\necho ord($payload);\necho ord(42042);\necho crc32(\"123456789\");\necho crc32($payload);\necho crc32(null);\n",
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
        11,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_int_diagnostic_")
            .count(),
        11,
        "{source}"
    );
    assert!(
        source.contains(", 0, &string_int_diagnostic_")
            && source.contains(", 1, &string_int_diagnostic_")
            && source.contains(", 2, &string_int_diagnostic_")
            && source.contains(", 3, &string_int_diagnostic_")
            && source.contains(", 4, &string_int_diagnostic_")
            && source.contains(", 5, &string_int_diagnostic_")
            && source.contains(", 6, &string_int_diagnostic_"),
        "byte compare, prefix compare, substring count, ord, and crc32 should share the tagged string-int ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        4,
        "prefix compare lengths and substr_count offset/length should share the native int conversion ABI:\n{source}"
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

const NATIVE_STRING_RESULT_SOURCE: &str = "<?php\n$payload = \"A\0B\";\necho strrev($payload), \"|\";\nprint str_rot13(\"Az-09\");\necho \"|\";\necho bin2hex($payload), \"|\";\necho strtolower(\"MiXeD\"), \"|\";\necho strtoupper(strtolower(\"MiXeD\")), \"|\";\necho ucfirst(\"word\"), \"|\";\necho lcfirst(\"Word\"), \"|\";\necho strrev(42042);\n";

#[test]
fn native_executable_c_source_routes_unary_string_results_through_runtime_contract() {
    let program = parse(NATIVE_STRING_RESULT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_string_result_operation_with_diagnostic(")
            .count(),
        9,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_result_diagnostic_")
            .count(),
        9,
        "{source}"
    );
    for operation_tag in ["4", "5", "13", "48", "49", "53", "54"] {
        assert!(
            source.contains(&format!(", {operation_tag}, &string_result_diagnostic_")),
            "tagged unary string-result operation {operation_tag} should route through the shared ABI:\n{source}"
        );
    }
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_echo_stdout("),
        "string-result handles should be consumed through native value output:\n{source}"
    );
}

const STRING_OFFSET_ISSET_EMPTY_SOURCE: &str = "<?php\n$selected = \"A0\0B\";\n$offset = \"1\";\necho isset($selected[0], $selected[$offset]) ? 1 : 0;\necho \"|\";\necho empty($selected[$offset]) ? 1 : 0;\necho \"|\";\necho isset($selected[99]) ? 1 : 0;\necho \"|\";\necho empty((\"102\")[1]) ? 1 : 0;\necho \"|\";\necho empty(strrev(\"za\")[1]) ? 1 : 0;\n";

const VALUE_OFFSET_PRESENCE_SOURCE: &str = "<?php\n$items = [\"hit\" => \"V\", \"null\" => null, \"empty\" => \"\"];\n$key = \"hit\";\n$text = \"A0\";\necho isset($items[$key]) ? 1 : 0;\necho \"|\";\necho empty($items[\"null\"]) ? 1 : 0;\necho \"|\";\necho isset($items[\"missing\"]) ? 1 : 0;\necho \"|\";\necho empty($items[\"empty\"]) ? 1 : 0;\necho \"|\";\necho isset($text[1]) ? 1 : 0;\necho \"|\";\necho empty($text[0]) ? 1 : 0;\n";

const STRING_OFFSET_READ_SOURCE: &str = concat!(
    "<?php\n",
    "$selected = \"A",
    "\0",
    "B\";\n",
    "$offset = \"1\";\n",
    "echo $selected[0], \"|\";\n",
    "echo $selected[$offset], \"|\";\n",
    "echo strlen($selected[$offset]), \"|\";\n",
    "$a = [];\n",
    "$a[$selected[0]] = $selected[2];\n",
    "echo $a[\"A\"];\n",
);

const STRING_OFFSET_WRITE_SOURCE: &str = concat!(
    "<?php\n",
    "$flag = (1 + 2) === 3;\n",
    "$s = $flag ? \"ABCD\" : \"WXYZ\";\n",
    "$i = \"1\";\n",
    "$rep = $flag ? \"",
    "\0",
    "\" : \"Q\";\n",
    "$s[$i] = $rep;\n",
    "echo $s;\n",
    "$a = [];\n",
    "$a[$s] = $flag ? \"V",
    "\0",
    "\" : \"Z\";\n",
    "echo \"|\", $a[$s];\n",
    "$s[3] = \"!\";\n",
    "echo \"|\", $s;\n",
);

const VALUE_OFFSET_MUTATION_ARRAY_WRITE_SOURCE: &str = "<?php\n$items = [\"seed\" => \"A\"];\n$key = \"dyn\";\n$items[$key] = \"B\";\n$slot = 2;\n$items[$slot] = \"C\";\necho $items[\"seed\"], \"|\", $items[$key], \"|\", $items[$slot], \"|\";\necho isset($items[$key]) ? 1 : 0;\n";

const ARRAY_LVALUE_NESTED_WRITE_SOURCE: &str = "<?php\n$outer = \"outer\";\n$inner = \"inner\";\n$items = [$outer => [$inner => \"old\", \"stay\" => \"S\"], \"root\" => \"R\"];\n$value = \"new\";\n$items[$outer][$inner] = $value;\n$items[$outer][\"added\"] = \"A\" . \"B\";\necho isset($items[$outer][$inner]) ? 1 : 0;\necho \"|\";\necho empty($items[$outer][\"added\"]) ? 1 : 0;\necho \"|\";\necho isset($items[$outer][\"stay\"]) ? 1 : 0;\necho \"|\", $items[\"root\"];\n";

const ARRAY_LVALUE_NESTED_APPEND_SOURCE: &str = "<?php\n$outer = \"outer\";\n$leaf = \"leaf\";\n$items = [$outer => [\"stay\" => \"S\"], \"root\" => \"R\"];\n$value = \"new\";\n$items[$outer][] = $value;\n$items[\"created\"][] = \"C\";\n$items[][$leaf] = \"Z\";\necho isset($items[$outer][0]) ? 1 : 0;\necho \"|\";\necho empty($items[\"created\"][0]) ? 1 : 0;\necho \"|\";\necho isset($items[0][$leaf]) ? 1 : 0;\necho \"|\", $items[\"root\"];\n";

const ARRAY_LVALUE_NESTED_ASSIGNMENT_EXPR_SOURCE: &str = "<?php\n$outer = \"outer\";\n$leaf = \"leaf\";\n$items = [$outer => [\"stay\" => \"S\"]];\necho ($items[$outer][$leaf] = \"A\"), \"|\";\necho ($items[$outer][] = \"B\"), \"|\";\necho ($items[][$leaf] = \"C\"), \"|\";\necho isset($items[$outer][$leaf]) ? 1 : 0;\necho \"|\";\necho empty($items[$outer][0]) ? 1 : 0;\necho \"|\";\necho isset($items[0][$leaf]) ? 1 : 0;\necho \"|\";\necho isset($items[$outer][\"stay\"]) ? 1 : 0;\n";

const ARRAY_LVALUE_NESTED_READ_SOURCE: &str = "<?php\n$outer = \"outer\";\n$inner = \"inner\";\n$leaf = \"leaf\";\n$items = [$outer => [$inner => \"v\"], \"other\" => [$leaf => \"x\"]];\n$out = [];\n$out[] = $items[$outer][$inner];\necho $items[$outer][$inner], \"|\";\nprint $items[\"other\"][$leaf];\necho \"|\", strtoupper($items[$outer][$inner]), \"|\", $out[0];\n";

const ARRAY_LVALUE_COMPOUND_ASSIGNMENT_SOURCE: &str = "<?php\n$key = \"slot\";\n$alt = \"alt\";\n$items = [$key => 2, $alt => 10, \"text\" => \"A\"];\n$out = [];\n$items[$key] += 3;\n$twenty = ($items[$alt] *= 2);\necho $twenty;\n$out[($items[$key] .= \"x\")] = ($items[$alt] -= 5);\necho \"|\", $out[\"5x\"], \"|\", $items[$alt], \"|\", $items[$key];\n";

const ARRAY_LVALUE_INCREMENT_DECREMENT_SOURCE: &str = "<?php\n$key = \"slot\";\n$float = \"float\";\n$items = [$key => 4, $float => 1.5, \"other\" => 9];\n$items[$key]++;\necho ++$items[$key], \"|\", $items[$key]--, \"|\", $items[$key], \"|\";\n$oldFloat = $items[$float]--;\necho $oldFloat, \"|\", $items[$float], \"|\";\n$out = [];\n$out[++$items[$key]] = $items[$key]--;\necho $out[6], \"|\", $items[$key];\n";

const VALUE_OFFSET_MUTATION_ARRAY_APPEND_SOURCE: &str = "<?php\n$items = [\"seed\" => \"A\"];\n$items[] = \"B\";\n$value = \"C\";\n$items[] = $value;\necho $items[\"seed\"], \"|\", $items[0], \"|\", $items[1], \"|\";\necho isset($items[1]) ? 1 : 0;\n";

const VALUE_OFFSET_MUTATION_ARRAY_ASSIGNMENT_EXPR_SOURCE: &str = "<?php\n$items = [\"seed\" => \"A\"];\n$key = \"named\";\n$value = \"C\";\necho ($items[] = \"B\"), \"|\";\necho ($items[] = $value), \"|\";\necho ($items[$key] = \"D\"), \"|\";\necho $items[0], \"|\", $items[1], \"|\", $items[$key], \"|\";\necho isset($items[1]) ? 1 : 0;\n";

const VALUE_OFFSET_ARRAY_READ_SOURCE: &str = "<?php\n$items = [\"first\" => \"q\", 2 => \"B\"];\n$key = \"first\";\n$out = [];\n$out[] = $items[$key];\necho $items[$key], \"|\";\nprint $items[2];\necho \"|\", $out[0], \"|\";\necho strtoupper($items[$key]);\n";

const VALUE_OFFSET_READ_RECOVERY_SOURCE: &str = "<?php\n$items = [\"present\" => \"P\", \"outer\" => [\"scalar\" => 7, \"nullish\" => null]];\n$missing = \"missing\";\necho $items[\"present\"], \"|\";\necho $items[$missing], \"|\";\necho $items[\"outer\"][\"scalar\"][\"leaf\"], \"|\";\n$slot = $items[\"outer\"][\"absent\"];\n$copy = $slot;\nprint $copy;\necho \"|\";\necho isset($items[\"present\"]) ? 1 : 0;\n";

const VALUE_OFFSET_NULL_COALESCE_SOURCE: &str = "<?php\n$items = [\"present\" => \"L\", \"nullish\" => null, 2 => \"N\"];\n$key = \"present\";\n$missing = \"missing\";\n$text = \"abc\";\n$offset = \"1\";\necho ($items[$key] ?? \"fallback\");\necho \"|\";\necho ($items[$missing] ?? \"fallback\");\necho \"|\";\necho ($items[\"nullish\"] ?? \"fallback\");\necho \"|\";\necho ($text[$offset] ?? \"fallback\");\necho \"|\";\necho ($text[9] ?? \"fallback\");\necho \"|\";\necho strtoupper($items[2] ?? \"x\");\n";

const VALUE_OFFSET_NULL_COALESCE_ASSIGN_SOURCE: &str = "<?php\n$items = [\"kept\" => \"K\", \"nullish\" => null, 2 => \"two\"];\n$key = \"missing\";\n$items[$key] ??= \"M\";\n$items[\"nullish\"] ??= \"N\";\n$items[\"kept\"] ??= $items[\"absent\"];\necho $items[$key], \"|\", $items[\"nullish\"], \"|\", $items[\"kept\"], \"|\";\necho ($items[2] ??= \"bad\"), \"|\";\necho ($items[\"expr\"] ??= (string) 7), \"|\";\n$stored = ($items[\"stored\"] ??= $items[$key]);\necho $stored, \"|\", $items[\"stored\"], \"|\";\necho isset($items[\"absent\"]) ? 1 : 0;\n";

const NATIVE_VALUE_VARIABLE_STORAGE_SOURCE: &str = "<?php\n$items = [0 => \"seed\", \"first\" => \"q\"];\n$key = \"first\";\n$slot = $items[$key];\n$copy = $slot;\necho $slot, \"|\", $copy, \"|\";\n$upper = strtoupper($copy);\necho $upper, \"|\";\n$fallback = $items[\"missing\"] ?? \"m\";\necho $fallback, \"|\";\n$cast = (string) 42;\necho $cast, \"|\";\n$items[] = $upper;\necho $items[1];\n";

const VALUE_OFFSET_MUTATION_ARRAY_UNSET_SOURCE: &str = "<?php\n$outer = \"outer\";\n$inner = \"inner\";\n$items = [\"keep\" => \"A\", \"drop\" => \"B\", 2 => \"C\", $outer => [$inner => \"N\", \"stay\" => \"S\"]];\n$key = \"drop\";\nunset($items[$key]);\nunset($items[2]);\nunset($items[99]);\nunset($items[$outer][$inner]);\necho isset($items[\"keep\"]) ? 1 : 0;\necho \"|\";\necho isset($items[$key]) ? 1 : 0;\necho \"|\";\necho empty($items[2]) ? 1 : 0;\necho \"|\";\necho isset($items[$outer][$inner]) ? 1 : 0;\necho \"|\";\n$items[$key] = \"D\";\necho $items[$key];\n";

const VALUE_OFFSET_MUTATION_ARRAY_MULTI_UNSET_SOURCE: &str = "<?php\n$left = [\"keep\" => \"L\", \"drop\" => \"D\", 2 => \"I\"];\n$right = [0 => \"R0\", \"drop\" => \"RD\"];\n$key = \"drop\";\nunset($left[$key], $right[0], $left[2], $right[\"missing\"]);\necho isset($left[\"keep\"]) ? 1 : 0;\necho \"|\";\necho isset($left[$key]) ? 1 : 0;\necho \"|\";\necho empty($right[0]) ? 1 : 0;\necho \"|\";\necho empty($left[2]) ? 1 : 0;\necho \"|\";\necho isset($right[\"drop\"]) ? 1 : 0;\n";

#[test]
fn native_executable_c_source_routes_string_offset_isset_empty_through_bool_boundary() {
    let program = parse(STRING_OFFSET_ISSET_EMPTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count(),
        6,
        "isset/empty offsets should share the runtime value-offset operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_bool_with_diagnostic(")
            .count(),
        6,
        "offset bool results should pass through the typed native bool boundary:\n{source}"
    );
    assert!(
        body.contains(", 1, &value_offset_bool_diagnostic_"),
        "isset offsets should use the shared operation tag:\n{source}"
    );
    assert!(
        body.contains(", 2, &value_offset_bool_diagnostic_"),
        "empty offsets should use the shared operation tag:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "presence paths should not use the string-only offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_and_string_offset_presence_through_value_boundary() {
    let program = parse(VALUE_OFFSET_PRESENCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count()
            >= 6,
        "array and string offset presence should use one value-offset ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_from_array"),
        "array subjects should be materialized through the native value carrier:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_bool_with_diagnostic"),
        "offset presence results should pass through the native bool boundary:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "array/string presence should not dispatch through the string-only offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_offset_reads_through_byte_boundary() {
    let program = parse(STRING_OFFSET_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_clone_bytes"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_report(string_offset_read_diagnostic_"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_string_offset_operation_with_diagnostic(")
            .count()
            >= 5,
        "string-offset reads should share the runtime string-offset operation boundary:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_string_clone_bytes(")
            .count()
            >= 5,
        "offset read values should materialize through the byte clone boundary:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_byte_buffer_free(string_offset_read_buffer")
            .count()
            >= 5,
        "owned string-offset read byte buffers must be cleaned up:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_message_stderr(string_offset_read_diagnostic_"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\""), "{source}");
}

#[test]
fn native_executable_c_source_routes_array_offset_writes_through_value_offset_mutation_boundary() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);"
        ),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        2,
        "direct array offset assignments should share the value-offset mutation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_array_clone(").count(),
        2,
        "array mutation results should rematerialize through the value-array clone boundary:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_from_array(array_"),
        "array subjects should enter the mutation ABI as native values:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_write_diagnostic_"),
        "array offset writes should use the shared write operation tag:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_writes_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_array"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        2,
        "nested array writes should share the lvalue owner/path operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count(),
        2,
        "nested array writes should use the write operation family for every target:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 4,
        "nested write targets should materialize every path key through shared path segments:\n{source}"
    );
    assert!(
        !body.contains("array_offset_write_diagnostic_"),
        "nested writes should not fall back to the direct value-offset write path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_appends_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_PATH_APPEND"),
        "nested append writes should declare the append path segment tag:\n{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        3,
        "nested append assignments should share the lvalue owner/path operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count(),
        3,
        "nested append assignments should use the write operation family:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_APPEND").count(),
        3,
        "each nested append target should materialize exactly one append path segment:\n{source}"
    );
    assert!(
        !body.contains("array_offset_append_value"),
        "nested appends should not fall back to the direct value-offset append path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_assignment_expression_values_through_lvalue_owner_operation(
) {
    let program = parse(ARRAY_LVALUE_NESTED_ASSIGNMENT_EXPR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        3,
        "nested assignment expressions should share the lvalue owner/path operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_NativeArrayLvalueResult array_lvalue_assign_expr_result_")
            .count(),
        3,
        "nested assignment expressions should use the assignment-expression result path:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count(),
        3,
        "nested assignment expressions should use the write operation family:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_APPEND").count(),
        2,
        "nested keyed and append assignment expressions should materialize append path segments only for append forms:\n{source}"
    );
    assert!(
        !body.contains("array_offset_assign_expr_diagnostic_"),
        "nested assignment expressions should not fall back to the direct value-offset assignment-expression path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_reads_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ"),
        "nested array reads should declare the lvalue read operation tag:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count()
            >= 4,
        "nested array reads should use the read operation family for output, print, string-result, and value-mutation consumers:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeArrayLvalueResult array_lvalue_read_result_")
            .count()
            >= 4,
        "nested array reads should share one lvalue read-result path across consumers:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 8,
        "nested read paths should materialize every dynamic and literal key through shared path segments:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "nested reads should not reintroduce the legacy array-read bypass:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_lvalue_compound_assignments_through_read_compute_write()
{
    let program = parse(ARRAY_LVALUE_COMPOUND_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_binary_result"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count()
            >= 4,
        "compound assignments should read current lvalue values through the shared read family:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count()
            >= 4,
        "compound assignments should write computed values through the shared write family:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_binary_result(").count() >= 4,
        "compound assignments should compute through the native value binary ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle array_lvalue_compound_current_")
            && body.contains("phpc_NativeValueHandle native_value_binary_"),
        "compound assignments should own current and computed native value handles:\n{source}"
    );
    assert!(
        !body.contains("assembly mutation lowering rejects"),
        "lowerable array lvalue compound assignments should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_lvalue_increment_decrement_through_update_boundary() {
    let program = parse(ARRAY_LVALUE_INCREMENT_DECREMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE"),
        "increment/decrement should declare the lvalue update family:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT"),
        "increment/decrement should declare the operation tag:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE")
            .count()
            >= 5,
        "statement and expression increment/decrement forms should share the update family:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_ARRAY_LVALUE_INCREMENT")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_DECREMENT")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_POSITION_PRE")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_POSITION_POST"),
        "increment/decrement should carry operation and result-position tags:\n{source}"
    );
    assert!(
        !body.contains(" = phpc_native_value_binary_result("),
        "increment/decrement should not be lowered as an exact +1/-1 binary expression:\n{source}"
    );
    assert!(
        !body.contains("assembly mutation lowering rejects"),
        "lowerable array lvalue increment/decrement should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_appends_through_value_offset_mutation_boundary() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);"
        ),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        2,
        "direct array appends should share the value-offset mutation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_array_clone(").count(),
        2,
        "array append results should rematerialize through the value-array clone boundary:\n{source}"
    );
    assert!(
        body.contains(", 1, &array_offset_append_diagnostic_"),
        "array appends should use the shared append operation tag:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_from_array(array_"),
        "array subjects should enter the append mutation ABI as native values:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_append_value_with_diagnostic("),
        "direct array append assignments should not bypass the value-offset mutation ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_assignment_expression_values_through_value_offset_mutation_boundary(
) {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_ASSIGNMENT_EXPR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);"
        ),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        3,
        "array assignment expressions should share the value-offset mutation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_array_clone(").count(),
        3,
        "array assignment-expression mutations should rematerialize array owners:\n{source}"
    );
    assert!(
        body.contains(", 1, &array_offset_append_assign_expr_diagnostic_"),
        "append assignment expressions should use the shared append operation tag:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_assign_expr_diagnostic_"),
        "keyed assignment expressions should use the shared write operation tag:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_append_value_with_diagnostic("),
        "array assignment expressions should not bypass the value-offset mutation ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_reads_through_value_offset_boundary() {
    let program = parse(VALUE_OFFSET_ARRAY_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count(),
        5,
        "array offset reads should share the value-offset read boundary across output and value consumers:\n{source}"
    );
    assert!(
        body.contains(", 0, &value_offset_read_diagnostic_"),
        "array offset reads should use the shared read operation tag:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "array read results should feed array value mutation consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "array read results should feed native string-result consumers:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "lowerable generated-C array reads should not bypass the shared value-offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_reports_array_read_recovery_through_shared_result_boundaries() {
    let program = parse(VALUE_OFFSET_READ_RECOVERY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_diagnostic_report(value_offset_read_diagnostic_"),
        "direct array-offset reads should report recoverable diagnostics through the value-offset result path:\n{source}"
    );
    assert!(
        body.contains("phpc_native_diagnostic_report(array_lvalue_read_result_"),
        "nested array-lvalue reads should report recoverable diagnostics through the lvalue result path:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count()
            >= 3,
        "direct reads and probes should continue to share the value-offset ABI:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count(),
        2,
        "nested missing/scalar reads should share the lvalue read operation family:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone"),
        "recovered native read values should still compose with direct-variable storage:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_offset_null_coalesce_through_value_boundary() {
    let program = parse(VALUE_OFFSET_NULL_COALESCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count(),
        12,
        "array and string offset null-coalescing reads should share presence and read calls:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_bool_with_diagnostic(")
            .count(),
        6,
        "null-coalescing probes should pass through the native bool diagnostic boundary:\n{source}"
    );
    assert!(
        body.contains(", 1, &value_offset_null_coalesce_diagnostic_"),
        "null-coalescing probes should use the shared isset operation tag:\n{source}"
    );
    assert!(
        body.contains(", 0, &value_offset_null_coalesce_read_diagnostic_"),
        "present null-coalescing offsets should read through the shared read operation tag:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "offset null-coalescing values should feed downstream native value consumers:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "offset null-coalescing should not bypass the shared value-offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_null_coalesce_assign_through_value_boundary() {
    let program = parse(VALUE_OFFSET_NULL_COALESCE_ASSIGN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert!(source.contains("phpc_native_value_array_clone"), "{source}");
    assert!(
        body.matches("array_offset_null_coalesce_assign_present_")
            .count()
            >= 5,
        "array-offset ??= statements and expressions should share one presence-probe path:\n{source}"
    );
    assert!(
        body.contains(", 1, &array_offset_null_coalesce_assign_diagnostic_"),
        "array-offset ??= should use the shared isset operation tag for probes:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_null_coalesce_assign_read_diagnostic_"),
        "array-offset ??= expression values should read present slots through the shared read tag:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_null_coalesce_assign_write_diagnostic_"),
        "array-offset ??= should write missing/null slots through the shared mutation tag:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "array-offset ??= should not reintroduce the legacy direct array read helper:\n{source}"
    );
}

#[test]
fn native_executable_c_source_stores_native_value_results_in_direct_variables() {
    let program = parse(NATIVE_VALUE_VARIABLE_STORAGE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_value_clone"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_clone(").count() >= 3,
        "native-value variable reads should clone handles for variable copies and downstream consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "stored array read and null-coalesce values should use the value-offset boundary:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "stored native values should feed string-result consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_cast_operation_with_diagnostic"),
        "cast results should store through the same native value handle path:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "stored array offset reads should not reintroduce the array-read bypass:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_unsets_through_lvalue_owner_operation() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("typedef struct { uint8_t tag; phpc_NativeValueHandle key; } phpc_NativeArrayPathSegment"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_array"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        4,
        "direct and nested array-offset unsets should share the lvalue owner operation boundary:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 5,
        "direct and nested unset paths should materialize every key through path segments:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_write_diagnostic_"),
        "the adjacent write should stay on the value-offset mutation ABI:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count()
            == 1,
        "unset should leave the value-offset mutation ABI for the lvalue owner path while preserving the follow-up write:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_unset_int")
            && !source.contains("phpc_native_array_unset_string"),
        "array unset should not reintroduce direct int/string unset helpers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_sequences_multi_operand_array_offset_unsets_through_lvalue_owner() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_MULTI_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        4,
        "each unset operand should enter the shared lvalue owner operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET")
            .count(),
        4,
        "multi-operand unset should reuse the shared unset operation family for every operand:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count(),
        4,
        "multi-operand direct unset should materialize every key through path segments:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_unset_int")
            && !source.contains("phpc_native_array_unset_string"),
        "multi-operand unset should not reintroduce direct array unset helpers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_offset_writes_through_value_offset_mutation_boundary() {
    let program = parse(STRING_OFFSET_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_clone_bytes"),
        "{source}"
    );
    assert!(source.contains("phpc_native_byte_buffer_free"), "{source}");
    assert!(
        source
            .matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count()
            >= 2,
        "string-offset writes should share the value-offset mutation boundary:\n{source}"
    );
    assert!(
        source.contains(", 0, &string_offset_write_diagnostic_"),
        "string-offset writes should use the write operation tag:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_string_offset_write_with_diagnostic"),
        "generated-C string-offset writes should not keep the string-only write ABI:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_string_clone_bytes(")
            .count()
            >= 2,
        "write results should become byte buffers through the shared clone boundary:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_byte_buffer_free(string_offset_write_buffer")
            .count()
            >= 2,
        "owned string-offset write byte buffers must be cleaned up:\n{source}"
    );
    assert!(
        source
            .contains("phpc_native_string_from_bytes((const uint8_t *)(string_offset_write_bytes"),
        "dynamic write bytes should be rematerialized by byte length:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)(string_offset_write_bytes"),
        "write result byte lengths should come from the runtime byte buffer:\n{source}"
    );
    assert!(!source.contains("printf(\"%s\""), "{source}");
}

const FILESYSTEM_PATH_OPERATION_SOURCE: &str = "<?php\n$path = \"pmt/\\0A\";\n$flag = str_contains($path, \"\\0\");\nfile_get_contents($path, $flag);\nrealpath($path);\nfile_exists(42);\nis_writable($path);\nfilesize($path);\nfilemtime($path);\ngetcwd();\nclearstatcache($flag, $path);\nrealpath_cache_get();\nrealpath_cache_size();\necho \"done\\n\";\n";

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
        10,
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
            && source.contains(", 11, &filesystem_path_operation_diagnostic_")
            && source.contains(", 12, &filesystem_path_operation_diagnostic_")
            && source.contains(", 13, &filesystem_path_operation_diagnostic_"),
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
echo "10" < "zeta", "\n";
echo "8foo" > "2", "\n";
echo ".5m" < "5.", "\n";
echo "+foo" < "-word", "\n";
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
        source.contains("phpc_native_string_from_bytes")
            && source.contains("phpc_native_comparison_operand_from_string_and_free"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_operand_from_string_bytes"),
        "generated C should consume string comparison operands through owned string handles, not the raw-byte operand ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operation_from_opcode"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operand_compare_operation_relation_and_free"),
        "runtime-linked native comparison should compute a shared relation result before branch consumption:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_relation_result_decision_or_report_stderr_and_free"),
        "runtime-linked native comparison should convert relation results through the shared decision/reporting ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonOperation"),
        "{source}"
    );
    assert!(source.contains("phpc_NativeComparisonOperand"), "{source}");
    assert!(
        source.contains("phpc_NativeComparisonRelationResult"),
        "comparison results should flow through the shared relation-result ABI before branch truth consumption:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonRelationResult comparison_relation_"),
        "generated C should allocate relation-result temporaries for comparison branch consumers:\n{source}"
    );
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
        "scalar/string comparisons should consume relation results through the relation decision ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_decision_abort_code"),
        "generated C should classify comparison branch aborts through the shared decision abort-code ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_status"),
        "generated C should not duplicate branch-decision status handling outside the abort-code ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_exit_code"),
        "generated C should not duplicate branch-decision exit-code handling outside the abort-code ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_decision_is_true"),
        "generated C should consume branch truth through the runtime branch-decision ABI:\n{source}"
    );
    assert!(
        !source.contains("if (comparison_exit_code_"),
        "generated C should not use exit-code checks as the comparison status classifier:\n{source}"
    );
    assert!(
        !source.contains("phpc_NativeComparisonResult"),
        "generated C should not materialize comparison result objects:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_operand_compare_operation_decision_and_free"),
        "generated C should not bypass the relation-result comparison contract:\n{source}"
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
        "generated C should not compose result branch consumption outside the relation-result boundary:\n{source}"
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

#[test]
fn native_executable_c_source_rematerializes_comparison_decisions_as_operands() {
    let program = parse(
        r#"<?php
$payload = "2";
echo (($payload > 1) == true), "\n";
echo (((1 < 2) == (2 > 1)) ? 1 : 0), "\n";
echo ((null == false) != ("10" < 2));
"#,
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeComparisonOperand phpc_native_comparison_branch_decision_result_operand"),
        "generated C should declare the branch-decision-to-operand ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operand_compare_operation_relation_and_free"),
        "outer comparisons should consume the shared comparison relation ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_relation_result_decision_or_report_stderr_and_free"),
        "outer comparisons should convert relation results through the shared decision/reporting ABI:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_comparison_branch_decision_result_operand(")
            .count()
            >= 4,
        "nested loose comparison operands should rematerialize branch decisions through the shared operand ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_branch_decision_is_true"),
        "generated C should still consume final branch truth through the runtime ABI:\n{source}"
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
            && source.contains("phpc_native_comparison_branch_decision_abort_code")
            && source.contains("phpc_native_comparison_branch_decision_is_true"),
        "array comparison results should use the common branch-decision abort/truth ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_status"),
        "array comparison guards should not duplicate branch-decision status handling outside the abort-code ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_exit_code"),
        "array comparison guards should not duplicate branch-decision exit-code handling outside the abort-code ABI:\n{source}"
    );
    assert!(
        !source.contains("if (comparison_exit_code_"),
        "array comparison guards should not use exit-code checks as the status classifier:\n{source}"
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
        source.contains("phpc_native_string_from_bytes((const uint8_t *)(")
            && source.contains("phpc_native_comparison_operand_from_string_and_free"),
        "dynamic string comparison operands should materialize through length-aware owned string handles:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_operand_from_string_bytes"),
        "dynamic string comparison operands should not bypass the string-handle comparison operand ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_comparison_operation_from_opcode")
            && source
                .contains("phpc_native_comparison_operand_compare_operation_relation_and_free")
            && source.contains(
                "phpc_native_comparison_relation_result_decision_or_report_stderr_and_free"
            ),
        "dynamic string operands should feed the shared comparison relation-result ABI:\n{source}"
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
        "<?php\n$payload = \"A\0B\";\n$repeated = \"A\0BA\0B\";\necho strcasecmp($payload, \"a\0b\");\necho \"\\n\";\necho strcmp($payload, \"a\0b\");\necho \"\\n\";\necho strncmp($payload, \"A\0C\", 3);\necho \"\\n\";\necho strncasecmp($payload, \"a\0c\", \"2\");\necho \"\\n\";\necho substr_count($repeated, $payload, 0, 6);\necho \"\\n\";\necho substr_count(42042, 42);\necho \"\\n\";\necho ord($payload);\necho \"\\n\";\necho ord(42042);\necho \"\\n\";\necho crc32(\"123456789\");\necho \"\\n\";\necho crc32($payload);\necho \"\\n\";\necho crc32(null);\necho \"\\n\";\n",
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
        "0\n-1\n-1\n0\n2\n2\n65\n52\n3421780262\n382410329\n0\n"
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
fn emit_exe_links_and_runs_unary_string_result_operation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_result_operation");
    let source_path = native_link_output_path("string_result_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, NATIVE_STRING_RESULT_SOURCE)
        .expect("native string-result source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-result source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"B\0A|Nm-09|410042|mixed|MIXED|Word|word|24024");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_isset_empty_bool_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_isset_empty_bool_boundary");
    let source_path = native_link_output_path("string_offset_isset_empty_bool_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, STRING_OFFSET_ISSET_EMPTY_SOURCE)
        .expect("native string-offset bool source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset bool source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"1|1|0|1|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_and_string_offset_presence_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_string_offset_presence_value_boundary");
    let source_path =
        native_link_output_path("array_string_offset_presence_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_PRESENCE_SOURCE)
        .expect("native value-offset presence source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value-offset presence source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"1|1|0|1|1|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_write_value_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_write_value_mutation");
    let source_path = native_link_output_path("array_offset_write_value_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_WRITE_SOURCE)
        .expect("native array offset mutation source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset mutation source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"A|B|C|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_write_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_write");
    let source_path = native_link_output_path("nested_array_lvalue_write_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_WRITE_SOURCE)
        .expect("native nested array lvalue write source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue write source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"1|0|1|R");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_append_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_append");
    let source_path = native_link_output_path("nested_array_lvalue_append_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_APPEND_SOURCE)
        .expect("native nested array lvalue append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue append source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"1|0|1|R");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_assignment_expression_value_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_assignment_expr_value");
    let source_path = native_link_output_path("nested_array_assignment_expr_value_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_ASSIGNMENT_EXPR_SOURCE)
        .expect("native nested array assignment-expression source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array assignment-expression source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"A|B|C|1|0|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_read_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_read");
    let source_path = native_link_output_path("nested_array_lvalue_read_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_READ_SOURCE)
        .expect("native nested array lvalue read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue read source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"v|x|V|v");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_append_value_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_append_value_mutation");
    let source_path = native_link_output_path("array_offset_append_value_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_APPEND_SOURCE)
        .expect("native array offset append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset append source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"A|B|C|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_assignment_expression_value_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_assignment_expr_value_mutation");
    let source_path = native_link_output_path("array_assignment_expr_value_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        VALUE_OFFSET_MUTATION_ARRAY_ASSIGNMENT_EXPR_SOURCE,
    )
    .expect("native array assignment-expression source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array assignment-expression source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"B|C|D|B|C|D|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_read_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_read_value_boundary");
    let source_path = native_link_output_path("array_offset_read_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_ARRAY_READ_SOURCE)
        .expect("native array offset read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset read source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"q|B|q|Q");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_lvalue_compound_assignment_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_lvalue_compound_assignment");
    let source_path = native_link_output_path("array_lvalue_compound_assignment_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_COMPOUND_ASSIGNMENT_SOURCE)
        .expect("native array lvalue compound-assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array lvalue compound source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"20|15|15|5x");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_lvalue_increment_decrement_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_lvalue_increment_decrement");
    let source_path = native_link_output_path("array_lvalue_increment_decrement_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_INCREMENT_DECREMENT_SOURCE)
        .expect("native array lvalue increment/decrement source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array lvalue increment/decrement source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"6|6|5|1.5|0.5|6|5");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_read_recovery_result_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_read_recovery_result");
    let source_path = native_link_output_path("array_read_recovery_result_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_READ_RECOVERY_SOURCE)
        .expect("native array read recovery source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array read recovery source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"P||||1");
    let stderr = String::from_utf8_lossy(&run.stderr);
    for expected in [
        "undefined array key \"missing\"",
        "Warning: Trying to access array offset on value of type int",
        "undefined array key \"absent\"",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in stderr {stderr:?}"
        );
    }

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_offset_null_coalesce_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("offset_null_coalesce_value_boundary");
    let source_path = native_link_output_path("offset_null_coalesce_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_NULL_COALESCE_SOURCE)
        .expect("native offset null-coalesce source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native offset null-coalesce source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"L|fallback|fallback|b|fallback|N");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_null_coalesce_assign_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_null_coalesce_assign_value_boundary");
    let source_path =
        native_link_output_path("array_offset_null_coalesce_assign_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_NULL_COALESCE_ASSIGN_SOURCE)
        .expect("native offset null-coalesce assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native offset null-coalesce assignment source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"M|N|K|two|7|M|M|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_native_value_variable_storage_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("native_value_variable_storage");
    let source_path = native_link_output_path("native_value_variable_storage_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, NATIVE_VALUE_VARIABLE_STORAGE_SOURCE)
        .expect("native value variable storage source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value variable storage source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"q|q|Q|m|42|Q");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_unset_lvalue_owner_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_unset_lvalue_owner");
    let source_path = native_link_output_path("array_offset_unset_lvalue_owner_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_UNSET_SOURCE)
        .expect("native array offset unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset unset source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"1|0|1|0|D");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_multi_operand_array_offset_unset_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("multi_operand_array_offset_unset");
    let source_path = native_link_output_path("multi_operand_array_offset_unset_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_MULTI_UNSET_SOURCE)
        .expect("native multi-operand array unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native multi-operand array unset source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"1|0|1|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_read_byte_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_read_byte_boundary");
    let source_path = native_link_output_path("string_offset_read_byte_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, STRING_OFFSET_READ_SOURCE)
        .expect("native string-offset read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset read source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"A|\0|1|B");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_write_byte_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_write_byte_boundary");
    let source_path = native_link_output_path("string_offset_write_byte_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, STRING_OFFSET_WRITE_SOURCE)
        .expect("native string-offset write source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset write source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"A\0CD|V\0|A\0C!");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_write_warning_continuation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_write_warning_continuation");
    let source_path =
        native_link_output_path("string_offset_write_warning_continuation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$flag = (1 + 2) === 3;\n$s = $flag ? \"ABC\" : \"WXY\";\n$rep = $flag ? \"XY\" : \"Z\";\n$s[1] = $rep;\n$a = [];\n$a[$s] = \"hit\";\necho $s, \"|\", strlen($s), \"|\", $a[$s];\n",
    )
    .expect("native string-offset warning source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset warning source path is valid UTF-8"),
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
    assert_eq!(run.stdout, b"AXC|3|hit");
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "Only the first byte will be assigned to the string offset"
    );

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
        "realpath_cache_get() awaits the shared filesystem realpath-cache ABI",
        "realpath_cache_size() awaits the shared filesystem realpath-cache ABI",
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
echo "10" < "zeta", "\n";
echo "8foo" > "2", "\n";
echo ".5m" < "5.", "\n";
echo "+foo" < "-word", "\n";
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
        "1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1"
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

#[test]
fn emit_exe_runs_nested_runtime_comparison_decision_operands() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("nested_runtime_comparison_operands").with_extension("php");
    fs::write(
        &temp_php,
        r#"<?php
$payload = "2";
echo (($payload > 1) == true), "\n";
echo (((1 < 2) == (2 > 1)) ? 1 : 0), "\n";
echo ((null == false) != ("10" < 2));
"#,
    )
    .expect("write temporary nested comparison operand source");
    let output_path = native_link_output_path("nested_runtime_comparison_operands");
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
            panic!("failed to compile nested comparison operand executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run nested comparison operand executable: {error}")
    });

    assert!(
        run.status.success(),
        "nested comparison operand executable failed"
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1\n1\n1");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

const GENERALIZED_ARRAY_KEY_SOURCE: &str = "<?php\n$slot = \"slot\";\n$two = 2;\n$numeric = \"3\";\n$nil = null;\n$binary = \"A\0B\";\n$a = [$slot => \"text\", $two => \"two\", $numeric => \"three\", $nil => \"null-key\", $binary => \"bin\0ary\", false => \"false-key\", true => \"true-key\", 4.0 => \"float-key\"];\necho $a[$slot], \"\\n\";\necho $a[2], \"\\n\";\necho $a[\"3\"], \"\\n\";\necho $a[$nil], \"\\n\";\necho $a[$binary], \"\\n\";\necho $a[false], \"\\n\";\necho $a[true], \"\\n\";\necho $a[4.0], \"\\n\";\n$a[$slot] = \"updated\";\n$a[$two] = \"two-updated\";\necho $a[\"slot\"], \"\\n\";\necho $a[2], \"\\n\";\n";

const NATIVE_ARRAY_APPEND_SOURCE: &str = "<?php\n$a = [1, \"two\", (string)(2 + 1), null];\necho $a[0], \"|\", $a[1], \"|\", $a[2], \"|\", $a[3];\n";

const NATIVE_VALUE_OPERATION_ARRAY_SOURCE: &str = "<?php\n$a = [];\n$a[\"s\" . \"lot\"] = (2 + 3) * (5 - 1);\n$a[(1 << 2) + 1] = \"fi\" . \"ve\";\n$a[\"neg\"] = -(\"6\" - 2);\necho $a[\"slot\"], \"|\", $a[5], \"|\", $a[\"neg\"];\n";

const NATIVE_VALUE_BITWISE_SOURCE: &str = "<?php\n$a = [];\n$a[\"and\"] = \"B\" & \"A\";\n$a[\"or\"] = \"A\" | \"\0\";\n$a[\"xor\"] = \"A\" ^ \"\0\";\n$a[\"not\"] = ~5;\n$a[\"left\"] = 8 << \"1\";\n$a[\"right\"] = 8 >> 1;\necho $a[\"and\"], \"|\", $a[\"or\"], \"|\", $a[\"xor\"], \"|\", $a[\"not\"], \"|\", $a[\"left\"], \"|\", $a[\"right\"];\n";

const NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE: &str = "<?php\n$a = [];\n$a[(int)\"5\"] = (string)((2 + 3) > 4);\n$a[(int)(3 <= 2)] = get_debug_type((string)123);\n$a[(float)\"3\"] = gettype((float)\"3.5\");\necho $a[5], \"|\", $a[0], \"|\", $a[3];\n";

const NATIVE_VALUE_TYPE_PREDICATE_SOURCE: &str = "<?php\necho is_int(\"6\" + 1), \"|\";\necho is_float(\"7\" / 2), \"|\";\necho is_string((string)(2 + 3)), \"|\";\necho is_bool((2 + 3) > 4), \"|\";\necho is_array((array)\"x\"), \"|\";\necho is_scalar(gettype((float)\"3.5\")), \"|\";\necho is_numeric((string)(2 + 3)), \"|\";\necho is_countable((array)null), \"|\";\necho is_iterable((array)\"x\"), \"|\";\necho is_null((array)null), \"|\";\necho is_object((array)\"x\");\n";

const NATIVE_VALUE_CAST_ECHO_SOURCE: &str = "<?php\necho (int)\"5.9\", \"|\";\necho (float)\"3.5\", \"|\";\necho (string)(2 + 3), \"|\";\necho (bool)\"0\", \"|\";\necho gettype((string)123);\n";

const NATIVE_VALUE_OPERATION_ECHO_SOURCE: &str = "<?php\n$left = \"6\";\n$right = 2;\necho -$left, \"|\";\necho $left + $right, \"|\";\necho $left / $right, \"|\";\necho \"A\" . \"\0B\", \"|\";\necho \"B\" & \"A\", \"|\";\necho 8 << \"1\";\n";

const NATIVE_VALUE_OPERATION_PRINT_SOURCE: &str = "<?php\n$left = \"6\";\n$right = 2;\n$a = [];\n$a[\"sum\"] = $left + $right;\nprint -$left;\nprint \"|\";\nprint $left + $right;\nprint \"|\";\nprint $left / $right;\nprint \"|\";\nprint \"A\" . \"\0B\";\nprint \"|\";\nprint gettype((string)123);\nprint \"|\";\nprint $a[\"sum\"];\n";

const NATIVE_VALUE_CAST_BUILTIN_SOURCE: &str = "<?php\n$a = [];\n$a[strval(5)] = floatval(\"3.5\");\n$a[\"truth\"] = doubleval(\"2.5\");\necho strval(\"A\"), \"|\", boolval(\"0\"), \"|\", floatval(\" -12.8 \"), \"|\", doubleval(\"2.5\"), \"|\", $a[\"5\"], \"|\", $a[\"truth\"];\n";

const NATIVE_ARRAY_VALUE_OPERAND_SOURCE: &str = "<?php\n$a = [];\n$a[\"nested\"] = [1, 2];\necho (int)[1], \"|\", (int)[], \"|\", (float)[0], \"|\", boolval([0]), \"|\", gettype([1]);\n";

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
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic"
        ),
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
        source.contains("PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_binary_result(")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_bitwise_operation_with_diagnostic("),
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
fn native_executable_c_source_routes_bitwise_values_through_shared_value_boundary() {
    let program = parse(NATIVE_VALUE_BITWISE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic"
        ),
        "{source}"
    );
    for op in [
        "PHPC_NATIVE_VALUE_BITWISE_AND",
        "PHPC_NATIVE_VALUE_BITWISE_OR",
        "PHPC_NATIVE_VALUE_BITWISE_XOR",
        "PHPC_NATIVE_VALUE_BITWISE_NOT",
        "PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT",
        "PHPC_NATIVE_VALUE_BITWISE_SHIFT_RIGHT",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }
    assert_eq!(
        source
            .matches(" = phpc_native_value_bitwise_operation_with_diagnostic(")
            .count(),
        6,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_binary_result("),
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_unary_result("),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && source.contains("phpc_native_array_read_key_with_diagnostic"),
        "bitwise values should compose through array write/read boundaries:\n{source}"
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
fn emit_exe_links_and_runs_native_bitwise_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_bitwise_value_boundary").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_BITWISE_SOURCE)
        .expect("write native bitwise value boundary fixture");
    let output_path = native_link_output_path("native_bitwise_value_boundary");
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
    assert_eq!(run.stdout, b"@|A|A|-6|16|4");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
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
fn native_executable_c_source_routes_compare_cast_and_type_name_results_through_shared_abi() {
    let program = parse(NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_compare_result",
        "extern phpc_NativeValueHandle phpc_native_value_cast_operation_with_diagnostic",
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
        source
            .matches(" = phpc_native_value_cast_operation_with_diagnostic(")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_cast_result("),
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
fn native_executable_c_source_routes_type_predicates_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_TYPE_PREDICATE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern bool phpc_native_value_type_predicate(phpc_NativeValueHandle value, uint8_t predicate);"
        ),
        "{source}"
    );

    for tag in [
        "PHPC_NATIVE_VALUE_TYPE_IS_NULL",
        "PHPC_NATIVE_VALUE_TYPE_IS_BOOL",
        "PHPC_NATIVE_VALUE_TYPE_IS_INT",
        "PHPC_NATIVE_VALUE_TYPE_IS_FLOAT",
        "PHPC_NATIVE_VALUE_TYPE_IS_STRING",
        "PHPC_NATIVE_VALUE_TYPE_IS_ARRAY",
        "PHPC_NATIVE_VALUE_TYPE_IS_SCALAR",
        "PHPC_NATIVE_VALUE_TYPE_IS_NUMERIC",
        "PHPC_NATIVE_VALUE_TYPE_IS_COUNTABLE",
        "PHPC_NATIVE_VALUE_TYPE_IS_ITERABLE",
        "PHPC_NATIVE_VALUE_TYPE_IS_OBJECT",
    ] {
        assert!(source.contains(tag), "{tag}\n\n{source}");
    }

    assert_eq!(
        source
            .matches(" = phpc_native_value_type_predicate(")
            .count(),
        11,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_compare_result(")
            && source.contains(" = phpc_native_value_cast_operation_with_diagnostic(")
            && source.contains(" = phpc_native_value_type_name_result("),
        "type predicates should consume existing value-result operation, comparison, cast, and type-name materialization:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_type_predicate_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_type_predicate_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_TYPE_PREDICATE_SOURCE)
        .expect("write native type-predicate value-result fixture");
    let output_path = native_link_output_path("native_type_predicate_value_result");
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
    assert_eq!(run.stdout, b"1|1|1|1|1|1|1|1|1||");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn native_executable_c_source_routes_cast_echoes_through_value_cast_operation_abi() {
    let program = parse(NATIVE_VALUE_CAST_ECHO_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_cast_operation_with_diagnostic"
        ),
        "{source}"
    );
    assert!(
        source
            .contains("extern phpc_NativeValueOperationResult phpc_native_value_type_name_result"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_cast_operation_with_diagnostic(")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_cast_result("),
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_type_name_result("),
        "{source}"
    );
    assert!(
        source.matches("phpc_native_value_echo_stdout(").count() >= 5,
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_VALUE_CAST_STRING")
            && source.contains("PHPC_NATIVE_VALUE_CAST_INT")
            && source.contains("PHPC_NATIVE_VALUE_CAST_BOOL")
            && source.contains("PHPC_NATIVE_VALUE_CAST_FLOAT")
            && source.contains("PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_operation_result_free"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_operation_echoes_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_OPERATION_ECHO_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_unary_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_binary_result",
        "extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic",
    ] {
        assert!(source.contains(declaration), "{declaration}\n\n{source}");
    }

    for op in [
        "PHPC_NATIVE_VALUE_UNARY_NEGATE",
        "PHPC_NATIVE_VALUE_BINARY_ADD",
        "PHPC_NATIVE_VALUE_BINARY_DIV",
        "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        "PHPC_NATIVE_VALUE_BITWISE_AND",
        "PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }

    assert_eq!(
        source.matches(" = phpc_native_value_unary_result(").count(),
        1,
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_binary_result(")
            .count(),
        3,
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_bitwise_operation_with_diagnostic(")
            .count(),
        2,
        "{source}"
    );
    assert!(
        source.matches("phpc_native_value_echo_stdout(").count() >= 6,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_operation_result_free"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_print_values_through_value_result_and_array_read_abi() {
    let program = parse(NATIVE_VALUE_OPERATION_PRINT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_unary_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_binary_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_type_name_result",
        "extern phpc_NativeValueHandle phpc_native_value_cast_operation_with_diagnostic",
        "extern phpc_NativeValueHandle phpc_native_array_read_key_with_diagnostic",
    ] {
        assert!(source.contains(declaration), "{declaration}\n\n{source}");
    }

    for op in [
        "PHPC_NATIVE_VALUE_UNARY_NEGATE",
        "PHPC_NATIVE_VALUE_BINARY_ADD",
        "PHPC_NATIVE_VALUE_BINARY_DIV",
        "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        "PHPC_NATIVE_VALUE_CAST_STRING",
        "PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }

    assert!(
        source.contains(" = phpc_native_value_unary_result(")
            && source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_type_name_result(")
            && source.contains(" = phpc_native_value_cast_operation_with_diagnostic(")
            && source.contains(" = phpc_native_array_read_key_with_diagnostic("),
        "print should use the existing runtime value-result and array-read boundaries:\n{source}"
    );
    assert!(
        source.matches("phpc_native_value_echo_stdout(").count() >= 8,
        "print output should flow through the value stdout ABI for direct and materialized values:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_scalar_cast_builtins_through_value_cast_contract() {
    let program = parse(NATIVE_VALUE_CAST_BUILTIN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_cast_operation_with_diagnostic"
        ),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_cast_operation_with_diagnostic(")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_cast_result("),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_VALUE_CAST_STRING")
            && source.contains("PHPC_NATIVE_VALUE_CAST_BOOL")
            && source.contains("PHPC_NATIVE_VALUE_CAST_FLOAT"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key")
            && source.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && source.contains("phpc_native_value_echo_stdout("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_handles_through_value_operand_boundary() {
    let program = parse(NATIVE_ARRAY_VALUE_OPERAND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_from_array(phpc_NativeArrayHandle array);"
        ),
        "{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_from_array(").count() >= 6,
        "array handles used as array values, casts, cast builtins, and type-name operands should share one value materialization boundary:\n{source}"
    );
    let immediate_array_value_frees = source
        .lines()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|lines| {
            let Some(array_handle) = lines[0]
                .trim()
                .split(" = phpc_native_value_from_array(")
                .nth(1)
                .and_then(|suffix| suffix.strip_suffix(");"))
            else {
                return false;
            };
            let expected = format!("phpc_native_array_free({array_handle});");
            lines[1].trim() == expected.as_str()
        })
        .count();
    assert!(
        immediate_array_value_frees >= 6,
        "temporary array literals cloned into value handles should release the source array immediately:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_cast_operation_with_diagnostic(")
            .count()
            >= 4,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_type_name_result("),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_insert_key_value_with_diagnostic(")
            && source.contains("phpc_native_array_append_value_with_diagnostic("),
        "nested array values should compose with keyed insert and append boundaries:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_cast_echo_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_cast_echo_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_CAST_ECHO_SOURCE)
        .expect("write native cast echo value-result fixture");
    let output_path = native_link_output_path("native_cast_echo_value_result");
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
    assert_eq!(run.stdout, b"5|3.5|5||string");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_operation_echo_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_operation_echo_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_OPERATION_ECHO_SOURCE)
        .expect("write native operation echo value-result fixture");
    let output_path = native_link_output_path("native_operation_echo_value_result");
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
    assert_eq!(run.stdout, b"-6|8|3|A\0B|@|16");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_operation_print_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_operation_print_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_OPERATION_PRINT_SOURCE)
        .expect("write native operation print value-result fixture");
    let output_path = native_link_output_path("native_operation_print_value_result");
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
    assert_eq!(run.stdout, b"-6|8|3|A\0B|string|8");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_array_value_operand_boundary_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_array_value_operand_boundary").with_extension("php");
    fs::write(&temp_php, NATIVE_ARRAY_VALUE_OPERAND_SOURCE)
        .expect("write native array value operand fixture");
    let output_path = native_link_output_path("native_array_value_operand_boundary");
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
    assert_eq!(run.stdout, b"1|0|1|1|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_scalar_cast_builtin_boundary_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_scalar_cast_builtin_boundary").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_CAST_BUILTIN_SOURCE)
        .expect("write native scalar-cast builtin fixture");
    let output_path = native_link_output_path("native_scalar_cast_builtin_boundary");
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
    assert_eq!(run.stdout, b"A||-12.8|2.5|3.5|2.5");
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

fn main_body(source: &str) -> &str {
    source
        .split_once("int main(void)")
        .map(|(_, body)| body)
        .unwrap_or(source)
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
