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
fn native_executable_c_source_routes_string_key_array_access_through_runtime_helpers() {
    let program =
        parse("<?php\n$a = [\"slot\" => 10];\n$a[\"slot\"] = 20;\necho $a[\"slot\"];\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_generalized_array_key_runtime_path(&source);
    assert!(
        source.contains("phpc_native_value_echo_stdout(array_read"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_write_string_scalar"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_read_string"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%lld\", 20"), "{source}");
}

#[test]
fn native_executable_c_source_routes_string_key_string_values_through_runtime_helpers() {
    let program =
        parse("<?php\n$a = [\"slot\" => \"old\"];\n$a[\"slot\"] = \"new\";\necho $a[\"slot\"];\n")
            .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_string_with_diagnostic"),
        "{source}"
    );
    assert_generalized_array_key_runtime_path(&source);
    assert!(
        source.contains("phpc_native_value_echo_stdout(array_read"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_write_string_value"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_read_string"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\", \"new"), "{source}");
}

#[test]
fn native_executable_c_source_routes_array_materialization_error_exits_through_program_cleanup() {
    let program = parse(
        "<?php\n$label = \"sym\";\n$a = [\"slot\" => \"old\", 7, \"tail\"];\n$b = [0 => true, \"next\" => \"value\"];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let main = main_body(&source);
    let error_exits = main
        .lines()
        .filter(|line| line.contains("return 1;"))
        .collect::<Vec<_>>();

    assert!(
        error_exits.len() >= 5,
        "expected append, keyed scalar, and keyed string error exits:\n{source}"
    );
    for exit in &error_exits {
        assert!(
            exit.contains("phpc_native_array_free("),
            "array materialization error exit must free live native arrays:\n{exit}\n\n{source}"
        );
    }
    assert!(
        error_exits
            .iter()
            .any(|exit| exit.matches("phpc_native_array_free(").count() >= 2),
        "later array materialization errors should clean all live native arrays:\n{source}"
    );
    assert!(
        error_exits
            .iter()
            .any(|exit| exit.contains("phpc_native_symbol_table_free(phpc_symbols);")),
        "program cleanup should include the shared symbol table when one is live:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_key_bool_values_through_runtime_helpers() {
    let program =
        parse("<?php\n$a = [\"slot\" => false];\n$a[\"slot\"] = true;\necho $a[\"slot\"];\n")
            .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_native_bool(false)"), "{source}");
    assert!(source.contains("phpc_native_bool(true)"), "{source}");
    assert_generalized_array_key_runtime_path(&source);
    assert!(
        source.contains("phpc_native_value_echo_stdout(array_read"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_write_string_scalar"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_read_string"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\", \"1"), "{source}");
}

#[test]
fn native_executable_c_source_routes_integer_key_bool_values_through_runtime_helpers() {
    let program = parse("<?php\n$a = [0 => false];\n$a[0] = true;\necho $a[0];\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_native_bool(false)"), "{source}");
    assert!(source.contains("phpc_native_bool(true)"), "{source}");
    assert_generalized_array_key_runtime_path(&source);
    assert!(
        source.contains("phpc_native_value_echo_stdout(array_read"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_write_int_scalar"),
        "{source}"
    );
    assert!(!source.contains("phpc_native_array_read_int"), "{source}");
    assert!(!source.contains("printf(\"%s\", \"1"), "{source}");
}

#[test]
fn native_executable_c_source_routes_string_key_null_values_through_runtime_helpers() {
    let program =
        parse("<?php\necho \"A\";\n$a = [\"slot\" => null];\necho $a[\"slot\"];\necho \"B\";\n")
            .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_native_null()"), "{source}");
    assert_generalized_array_key_runtime_path(&source);
    assert!(
        source.contains("phpc_native_value_echo_stdout(array_read"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_write_string_scalar"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_array_read_string"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\", \"AB"), "{source}");
}

#[test]
fn native_executable_c_source_routes_string_variables_through_symbol_table_helpers() {
    let program = parse("<?php\n$label = \"sym\";\necho $label;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_write_read_order(&source);
    assert!(source.contains("phpc_native_value_echo_stdout"), "{source}");
    assert!(source.contains("label"), "{source}");
    assert!(!source.contains("printf(\"%s\", \"sym"), "{source}");
}

#[test]
fn native_executable_c_source_routes_int_variables_through_symbol_table_helpers() {
    let program = parse("<?php\n$n = 42;\necho $n;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_write_read_order(&source);
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(42))"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_echo_stdout"),
        "{source}"
    );
    assert!(source.contains("n"), "{source}");
    assert!(
        !main_body(&source).contains("printf(\"%lld\", 42);"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_bool_variables_through_symbol_table_helpers() {
    let program = parse("<?php\n$flag = true;\necho $flag;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_write_read_order(&source);
    assert!(
        source.contains("phpc_native_linked_value_from_bool(1)"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_echo_stdout"),
        "{source}"
    );
    assert!(source.contains("flag"), "{source}");
    assert!(
        !main_body(&source).contains("printf(\"%s\", \"1\");"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_variable_copies_through_symbol_table_helpers() {
    let program = parse("<?php\n$a = \"sym\";\n$b = $a;\necho $b;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_copy_readback(&source, "a", "b");
    assert!(source.contains("phpc_native_value_echo_stdout"), "{source}");
    assert!(
        !main_body(&source).contains("printf(\"%s\", \"sym\");"),
        "{source}"
    );
    assert_eq!(
        main_body(&source).matches("phpc_native_string_from_bytes").count(),
        1,
        "variable copy should read the source symbol instead of rematerializing the string literal:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_int_variable_copies_through_symbol_table_helpers() {
    let program = parse("<?php\n$a = 42;\n$b = $a;\necho $b;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_copy_readback(&source, "a", "b");
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(42))"),
        "{source}"
    );
    assert_eq!(
        main_body(&source)
            .matches("phpc_native_linked_value_from_int((long long)(42))")
            .count(),
        1,
        "variable copy should read the source symbol instead of rematerializing the integer literal:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_int_variable_copy_chains_through_symbol_table_helpers() {
    let program = parse("<?php\n$a = 42;\n$b = $a;\n$c = $b;\necho $c;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_copy_chain_readback(&source, &["a", "b", "c"]);
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(42))"),
        "{source}"
    );
    assert_eq!(
        main_body(&source)
            .matches("phpc_native_linked_value_from_int((long long)(42))")
            .count(),
        1,
        "copy chain should read linked source symbols instead of rematerializing the integer literal:\n{source}"
    );
    assert!(
        !main_body(&source).contains("printf(\"%lld\", 42);"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_bool_variable_copies_through_symbol_table_helpers() {
    let program = parse("<?php\n$a = true;\n$b = $a;\necho $b;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_copy_readback(&source, "a", "b");
    assert!(
        source.contains("phpc_native_linked_value_from_bool(1)"),
        "{source}"
    );
    assert_eq!(
        main_body(&source)
            .matches("phpc_native_linked_value_from_bool(1)")
            .count(),
        1,
        "variable copy should read the source symbol instead of rematerializing the boolean literal:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_scalar_overwrites_through_symbol_table_helpers() {
    let program = parse("<?php\n$n = 7;\n$n = 42;\necho $n;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_overwrite_readback(&source, "n");
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(7))"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(42))"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("printf(\"%lld\", 7);"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("printf(\"%lld\", 42);"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_variable_unset_and_isset_through_symbol_table_helpers() {
    let program =
        parse("<?php\n$n = 42;\necho isset($n) ? 1 : 0;\nunset($n);\necho isset($n) ? 1 : 0;\n")
            .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_unset_isset_routing(&source, "n");
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(42))"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("if (1) { printf(\"%s\", \"1\"); }"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("if (0) { printf(\"%s\", \"1\"); }"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("printf(\"%lld\", 0);"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_multi_variable_unset_through_symbol_table_helpers() {
    let program = parse(
        "<?php\n$a = 11;\n$b = 22;\nunset($a, $b);\necho isset($a) ? 1 : 0;\necho isset($b) ? 1 : 0;\n$a = 33;\necho $a;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_multi_unset_isset_reassign_routing(&source, &["a", "b"], "a");
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(11))"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(22))"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(33))"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("if (0) { printf(\"%s\", \"1\"); }"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("printf(\"%lld\", 33);"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_multi_argument_isset_through_symbol_table_helpers() {
    let program = parse(
        "<?php\n$a = 11;\n$b = 22;\necho isset($a, $b) ? 1 : 0;\nunset($b);\necho isset($a, $b) ? 1 : 0;\n$b = 33;\necho isset($a, $b) ? 1 : 0;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert_symbol_table_multi_argument_isset_reassign_routing(&source, &["a", "b"], "b");
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(11))"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(22))"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_linked_value_from_int((long long)(33))"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("if (1) { printf(\"%s\", \"1\"); }"),
        "{source}"
    );
    assert!(
        !main_body(&source).contains("if (0) { printf(\"%s\", \"1\"); }"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_debug_output_builtins_through_value_boundary() {
    let program = parse(
        "<?php\n$items = [\"key\" => \"A\0B\", 7, true, null];\nvar_dump($items, \"tail\");\nprint_r($items);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_debug_output_with_diagnostic("
        ),
        "{source}"
    );
    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_value_from_array_clone("),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_debug_output_with_diagnostic(")
            .count(),
        3,
        "{source}"
    );
    assert!(
        body.contains(", 0, false, &debug_output_diagnostic_")
            && body.contains(", 1, false, &debug_output_diagnostic_"),
        "{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle debug_value_")
            && body.contains("phpc_native_value_from_array_clone(array_"),
        "{source}"
    );
    assert!(!body.contains("printf("), "{source}");
}

#[test]
fn native_executable_c_source_declares_scalar_value_materialization_for_debug_output() {
    let program = parse("<?php\nvar_dump(null, false, true, 42, 2.5);\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_from_scalar(phpc_NativeScalarValue value);"
        ),
        "{source}"
    );
    for expected in [
        "phpc_native_value_from_scalar(phpc_native_null())",
        "phpc_native_value_from_scalar(phpc_native_bool(false))",
        "phpc_native_value_from_scalar(phpc_native_bool(true))",
        "phpc_native_value_from_scalar(phpc_native_int((int64_t)(42)))",
        "phpc_native_value_from_scalar(phpc_native_float((double)(2.5)))",
    ] {
        assert!(body.contains(expected), "{expected}\n\n{source}");
    }
    assert_eq!(
        body.matches(" = phpc_native_value_debug_output_with_diagnostic(")
            .count(),
        5,
        "{source}"
    );
    assert!(!body.contains("printf("), "{source}");
}

#[test]
fn emit_exe_links_and_runs_debug_output_runtime_helper_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let output_path = native_link_output_path("debug_output_runtime_helper");
    let mut input_path = output_path.clone();
    input_path.set_extension("php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);

    fs::write(
        &input_path,
        "<?php\n$items = [\"key\" => \"A\0B\", 7, true, null];\nvar_dump($items);\nprint_r($items);\n",
    )
    .expect("write native debug-output fixture");
    let relative_fixture = input_path
        .strip_prefix(workspace_root)
        .unwrap_or(&input_path)
        .to_str()
        .expect("native debug-output fixture path is valid UTF-8")
        .to_string();

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

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));
    let expected = b"array(4) {\n  [\"key\"]=>\n  string(3) \"A\0B\"\n  [0]=>\n  int(7)\n  [1]=>\n  bool(true)\n  [2]=>\n  NULL\n}\nArray\n(\n    [key] => A\0B\n    [0] => 7\n    [1] => 1\n    [2] => \n)\n";

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);
}

#[test]
fn emit_exe_links_and_runs_scalar_debug_output_runtime_helper_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let output_path = native_link_output_path("scalar_debug_output_runtime_helper");
    let mut input_path = output_path.clone();
    input_path.set_extension("php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);

    fs::write(
        &input_path,
        "<?php\nvar_dump(null, false, true, 42, 2.5);\n",
    )
    .expect("write scalar native debug-output fixture");
    let relative_fixture = input_path
        .strip_prefix(workspace_root)
        .unwrap_or(&input_path)
        .to_str()
        .expect("native debug-output fixture path is valid UTF-8")
        .to_string();

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

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));
    let expected = b"NULL\nbool(false)\nbool(true)\nint(42)\nfloat(2.5)\n";

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);
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
fn emit_exe_links_and_runs_generalized_array_key_materialization_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let output_path = native_link_output_path("generalized_array_key_materialization");
    let mut input_path = output_path.clone();
    input_path.set_extension("php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);

    fs::write(
        &input_path,
        "<?php\n$slot = \"slot\";\n$two = 2;\n$numeric = \"3\";\n$binary = \"A\0B\";\n$a = [$slot => \"text\", $two => \"two\", $numeric => \"three\", null => \"null-key\", $binary => \"binary\"];\necho $a[$slot], \"\\n\";\necho $a[2], \"\\n\";\necho $a[\"3\"], \"\\n\";\necho $a[null], \"\\n\";\necho $a[$binary], \"\\n\";\n$a[$slot] = \"updated\";\n$a[$two] = \"two-updated\";\necho $a[\"slot\"], \"\\n\";\necho $a[2], \"\\n\";\n",
    )
    .expect("write generalized native array-key fixture");
    let relative_fixture = input_path
        .strip_prefix(workspace_root)
        .unwrap_or(&input_path)
        .to_str()
        .expect("native array-key fixture path is valid UTF-8")
        .to_string();

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

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));
    let expected = b"text\ntwo\nthree\nnull-key\nbinary\nupdated\ntwo-updated\n";

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, expected);
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&input_path);
}

fn assert_generalized_array_key_runtime_path(source: &str) {
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
}

fn assert_symbol_table_write_read_order(source: &str) {
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(
        source.contains("phpc_native_symbol_table_write"),
        "{source}"
    );
    assert!(source.contains("phpc_native_symbol_table_read"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");

    let write = source
        .find("phpc_native_symbol_table_write(phpc_symbols")
        .expect("source contains main-body symbol-table write");
    let read = source
        .find("phpc_native_symbol_table_read(phpc_symbols")
        .expect("source contains main-body symbol-table read");
    assert!(
        write < read,
        "variable assignment should be written before readback:\n{source}"
    );
}

fn assert_symbol_table_copy_readback(source: &str, source_name: &str, target_name: &str) {
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    assert!(source.contains(&format!("/* {source_name} */")), "{source}");
    assert!(source.contains(&format!("/* {target_name} */")), "{source}");

    let body = main_body(source);
    assert_eq!(
        body.matches("phpc_native_symbol_table_write(phpc_symbols")
            .count(),
        2,
        "source assignment and target assignment should both write symbols:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_read(phpc_symbols")
            .count(),
        2,
        "target assignment and echo should both read symbols:\n{source}"
    );

    let first_write = body
        .find("phpc_native_symbol_table_write(phpc_symbols")
        .expect("source assignment writes a symbol");
    let copy_read = body
        .find("phpc_NativeLinkedValue value_")
        .expect("variable copy reads source symbol into a linked value");
    let after_copy_read = &body[copy_read..];
    let copy_write = copy_read
        + after_copy_read
            .find("phpc_native_symbol_table_write(phpc_symbols")
            .expect("variable copy writes target symbol");
    let echo_read = body
        .rfind("phpc_native_symbol_table_read(phpc_symbols")
        .expect("echo reads target symbol");

    assert!(
        first_write < copy_read && copy_read < copy_write && copy_write < echo_read,
        "expected source write, source read, target write, target read order:\n{source}"
    );
}

fn assert_symbol_table_copy_chain_readback(source: &str, names: &[&str]) {
    assert!(names.len() >= 2);
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    for name in names {
        assert!(source.contains(&format!("/* {name} */")), "{source}");
    }

    let body = main_body(source);
    assert_eq!(
        body.matches("phpc_native_symbol_table_write(phpc_symbols")
            .count(),
        names.len(),
        "initial assignment and each chained copy should write symbols:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_read(phpc_symbols")
            .count(),
        names.len(),
        "each chained copy and echo should read linked symbols:\n{source}"
    );

    let mut cursor = 0;
    let first_write = body[cursor..]
        .find("phpc_native_symbol_table_write(phpc_symbols")
        .expect("initial assignment writes a symbol");
    cursor += first_write + "phpc_native_symbol_table_write(phpc_symbols".len();

    for _ in 1..names.len() {
        let copy_read = body[cursor..]
            .find("phpc_NativeLinkedValue value_")
            .expect("copy reads source symbol into a linked value");
        cursor += copy_read + "phpc_NativeLinkedValue value_".len();

        let copy_write = body[cursor..]
            .find("phpc_native_symbol_table_write(phpc_symbols")
            .expect("copy writes target symbol");
        cursor += copy_write + "phpc_native_symbol_table_write(phpc_symbols".len();
    }

    let echo_read = body[cursor..]
        .find("phpc_native_symbol_table_read(phpc_symbols")
        .expect("echo reads final target symbol");
    cursor += echo_read + "phpc_native_symbol_table_read(phpc_symbols".len();
    assert!(
        cursor <= body.len(),
        "expected source write, chained copy read/write pairs, then final readback:\n{source}"
    );
}

fn assert_symbol_table_overwrite_readback(source: &str, name: &str) {
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    assert!(source.contains(&format!("/* {name} */")), "{source}");

    let body = main_body(source);
    assert_eq!(
        body.matches("phpc_native_symbol_table_write(phpc_symbols")
            .count(),
        2,
        "initial assignment and overwrite should both write the symbol:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_read(phpc_symbols")
            .count(),
        1,
        "echo should read the overwritten symbol from the table:\n{source}"
    );

    let first_write = body
        .find("phpc_native_symbol_table_write(phpc_symbols")
        .expect("initial assignment writes a symbol");
    let second_write = body
        .rfind("phpc_native_symbol_table_write(phpc_symbols")
        .expect("overwrite writes the same symbol");
    let read = body
        .find("phpc_native_symbol_table_read(phpc_symbols")
        .expect("echo reads the symbol");

    assert!(
        first_write < second_write && second_write < read,
        "expected initial write, overwrite, then readback order:\n{source}"
    );
}

fn assert_symbol_table_unset_isset_routing(source: &str, name: &str) {
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    assert!(
        source.contains("phpc_native_symbol_table_isset"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_unset"),
        "{source}"
    );
    assert!(source.contains(&format!("/* {name} */")), "{source}");

    let body = main_body(source);
    assert_eq!(
        body.matches("phpc_native_symbol_table_write(phpc_symbols")
            .count(),
        1,
        "initial assignment should write one symbol:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_isset(phpc_symbols")
            .count(),
        2,
        "isset before and after unset should both call the helper:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_unset(phpc_symbols")
            .count(),
        1,
        "unset should call the helper once:\n{source}"
    );

    let write = body
        .find("phpc_native_symbol_table_write(phpc_symbols")
        .expect("assignment writes a symbol");
    let first_isset = body
        .find("phpc_native_symbol_table_isset(phpc_symbols")
        .expect("first isset calls the helper");
    let unset = body
        .find("phpc_native_symbol_table_unset(phpc_symbols")
        .expect("unset calls the helper");
    let second_isset = body
        .rfind("phpc_native_symbol_table_isset(phpc_symbols")
        .expect("second isset calls the helper");

    assert!(
        write < first_isset && first_isset < unset && unset < second_isset,
        "expected write, isset, unset, then isset order:\n{source}"
    );
}

fn assert_symbol_table_multi_unset_isset_reassign_routing(
    source: &str,
    unset_names: &[&str],
    reassigned_name: &str,
) {
    assert_eq!(unset_names.len(), 2);
    assert!(unset_names.contains(&reassigned_name));
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    assert!(
        source.contains("phpc_native_symbol_table_isset"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_unset"),
        "{source}"
    );
    assert!(source.contains("phpc_native_symbol_table_read"), "{source}");
    for name in unset_names {
        assert!(source.contains(&format!("/* {name} */")), "{source}");
    }

    let body = main_body(source);
    assert_eq!(
        body.matches("phpc_native_symbol_table_write(phpc_symbols")
            .count(),
        3,
        "two initial assignments and one reassignment should write symbols:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_unset(phpc_symbols")
            .count(),
        2,
        "multi-variable unset should call the helper for both locals:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_isset(phpc_symbols")
            .count(),
        2,
        "isset after multi-variable unset should call the helper for both locals:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_read(phpc_symbols")
            .count(),
        1,
        "echo after reassignment should read the restored symbol from the table:\n{source}"
    );

    let write = "phpc_native_symbol_table_write(phpc_symbols";
    let unset = "phpc_native_symbol_table_unset(phpc_symbols";
    let isset = "phpc_native_symbol_table_isset(phpc_symbols";
    let read = "phpc_native_symbol_table_read(phpc_symbols";

    let mut cursor = 0;
    cursor = find_after(
        body,
        cursor,
        write,
        "first local assignment writes a symbol",
    ) + write.len();
    cursor = find_after(
        body,
        cursor,
        write,
        "second local assignment writes a symbol",
    ) + write.len();
    cursor = find_after(body, cursor, unset, "first local unset calls the helper") + unset.len();
    cursor = find_after(body, cursor, unset, "second local unset calls the helper") + unset.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "first post-unset isset calls the helper",
    ) + isset.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "second post-unset isset calls the helper",
    ) + isset.len();
    cursor = find_after(
        body,
        cursor,
        write,
        "reassignment after unset writes the restored symbol",
    ) + write.len();
    let _ = find_after(body, cursor, read, "echo reads the reassigned symbol");
}

fn assert_symbol_table_multi_argument_isset_reassign_routing(
    source: &str,
    names: &[&str],
    reassigned_name: &str,
) {
    assert_eq!(names.len(), 2);
    assert!(names.contains(&reassigned_name));
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    assert!(
        source.contains("phpc_native_symbol_table_isset"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_unset"),
        "{source}"
    );
    for name in names {
        assert!(source.contains(&format!("/* {name} */")), "{source}");
    }

    let body = main_body(source);
    assert_eq!(
        body.matches("phpc_native_symbol_table_write(phpc_symbols")
            .count(),
        3,
        "two initial assignments and one reassignment should write symbols:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_unset(phpc_symbols")
            .count(),
        1,
        "missing-operand unset should call the helper once:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_isset(phpc_symbols")
            .count(),
        6,
        "three multi-argument isset expressions should call the helper for both locals:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_native_symbol_table_read(phpc_symbols")
            .count(),
        0,
        "multi-argument isset should not read scalar values from the table:\n{source}"
    );

    let write = "phpc_native_symbol_table_write(phpc_symbols";
    let unset = "phpc_native_symbol_table_unset(phpc_symbols";
    let isset = "phpc_native_symbol_table_isset(phpc_symbols";

    let mut cursor = 0;
    cursor = find_after(
        body,
        cursor,
        write,
        "first local assignment writes a symbol",
    ) + write.len();
    cursor = find_after(
        body,
        cursor,
        write,
        "second local assignment writes a symbol",
    ) + write.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "first all-set isset operand uses helper",
    ) + isset.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "second all-set isset operand uses helper",
    ) + isset.len();
    cursor = find_after(
        body,
        cursor,
        unset,
        "missing operand unset calls the helper",
    ) + unset.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "first missing-case isset operand uses helper",
    ) + isset.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "second missing-case isset operand uses helper",
    ) + isset.len();
    cursor = find_after(
        body,
        cursor,
        write,
        "reassignment after unset writes the restored symbol",
    ) + write.len();
    cursor = find_after(
        body,
        cursor,
        isset,
        "first restored isset operand uses helper",
    ) + isset.len();
    let _ = find_after(
        body,
        cursor,
        isset,
        "second restored isset operand uses helper",
    );
}

fn main_body(source: &str) -> &str {
    source
        .split_once("int main(void) {")
        .map(|(_, body)| body)
        .expect("generated C source contains main body")
}

fn find_after(source: &str, cursor: usize, needle: &str, label: &str) -> usize {
    cursor
        + source[cursor..]
            .find(needle)
            .unwrap_or_else(|| panic!("{label}:\n{source}"))
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
