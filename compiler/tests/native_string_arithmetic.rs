use std::path::Path;
use std::process::Command;

use php_compiler::{emit_asm_source, emit_ir_source, run_source};

#[test]
fn phpc_run_still_handles_numeric_string_arithmetic() {
    let execution = run_source(
        r#"<?php
echo "2" + 3, "\n";
echo 8 - "2.5", "\n";
echo "3e1" * 2, "\n";
echo 9 / "3", "\n";
echo "8" % 3;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "5\n5.5\n60\n3\n2");
}

#[test]
fn emit_ir_lowers_numeric_string_operands_in_static_primitive_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
echo "2" + 3, "\n";
echo 8 - "2.5", "\n";
echo "3e1" * 2;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("call %phpc.NativeScalarValue @phpc_native_int(i64 5)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeScalarValue @phpc_native_float(double 5.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeScalarValue @phpc_native_float(double 60.0)"),
        "{ir}"
    );
    assert!(uses_native_diagnostic_result_output(&ir, 5), "{ir}");
    assert!(!ir.contains("call i32 (ptr, ...) @printf"), "{ir}");

    for (source, tag) in [
        ("<?php\necho \"8\" % 3;\n", 4),
        ("<?php\necho 9 / \"3\";\n", 3),
    ] {
        let ir = emit_ir_source(source).unwrap();
        assert!(
            ir.contains("call %phpc.NativeValueOperationResult @phpc_native_value_binary_result"),
            "{source}: {ir}"
        );
        assert!(
            ir.contains(&format!("i8 {tag}")),
            "expected native value-operation tag {tag} for {source}:\n{ir}"
        );
    }
}

fn uses_native_diagnostic_result_output(ir: &str, minimum_sinks: usize) -> bool {
    ir.contains("%phpc.NativeDiagnosticResult = type { ptr }")
        && ir.contains("@phpc_native_diagnostic_result_from_value")
        && ir
            .matches("@phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
            .count()
            >= minimum_sinks
}

#[test]
fn emit_asm_routes_non_numeric_string_arithmetic_to_runtime_error_path() {
    let asm = emit_asm_source("<?php\necho \"two\" + 3;\n").unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn native_string_arithmetic_emit_ir_cli_routes_value_result_operands() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone164/native_string_arithmetic.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    assert!(
        output.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(
        ir.contains("%phpc.NativeValueOperationResult = type")
            && ir.contains("@phpc_native_value_binary_result"),
        "{ir}"
    );
    assert!(
        ir.contains("i8 3") && ir.contains("i8 4"),
        "string division and modulo should use shared value-operation tags:\n{ir}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "unexpected CLI stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
