use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::{
    emit_ir_source, native_runtime_scalar_echo_probe_ir,
    native_runtime_scalar_echo_probe_ir_for_target, NativeRuntimeIrTarget,
};

#[test]
fn scalar_echo_probe_ir_matches_committed_snapshot() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent")
        .join("tests/fixtures/milestone636/native_runtime_scalar_echo_probe.ir");
    let expected = std::fs::read_to_string(fixture).expect("read native runtime IR probe fixture");

    assert_eq!(native_runtime_scalar_echo_probe_ir(), expected);
}

#[test]
fn scalar_echo_probe_ir_names_exported_runtime_helpers() {
    let ir = native_runtime_scalar_echo_probe_ir();

    assert!(ir.contains("%phpc.NativeScalarValue = type"), "{ir}");
    assert!(
        ir.contains("%phpc.NativeByteBuffer = type { ptr, i64, i64 }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_scalar_echo_len(%phpc.NativeScalarValue)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_scalar_echo_write(%phpc.NativeScalarValue, ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_scalar_echo_bytes(%phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeByteBuffer @phpc_native_byte_buffer_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer)"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeStringHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeValueHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeDiagnosticHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeArrayHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeObjectHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeResourceHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeReferenceHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_string_len(%phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare ptr @phpc_native_string_bytes(%phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_string_clone_bytes(%phpc.NativeStringHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_string_free(%phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_value_free(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_diagnostic_message_clone_bytes(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeArrayHandle @phpc_native_array_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeObjectHandle @phpc_native_object_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_object_is_null(%phpc.NativeObjectHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeResourceHandle @phpc_native_resource_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_resource_is_null(%phpc.NativeResourceHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeReferenceHandle @phpc_native_reference_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_scalar_echo_owned_bytes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_byte_buffer_from_bytes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_handle_roundtrip()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_handle_to_value_echo()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_container_handle_null_shapes()"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %value)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("not production lowering or linked execution"),
        "{ir}"
    );
}

#[test]
fn scalar_echo_probe_ir_renders_32_bit_usize_helper_signatures() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent")
        .join("tests/fixtures/milestone637/native_runtime_scalar_echo_probe_i32.ir");
    let expected =
        std::fs::read_to_string(fixture).expect("read native runtime i32 IR probe fixture");

    assert_eq!(
        native_runtime_scalar_echo_probe_ir_for_target(NativeRuntimeIrTarget::Pointer32),
        expected
    );

    let ir = native_runtime_scalar_echo_probe_ir_for_target(NativeRuntimeIrTarget::Pointer32);
    assert!(
        ir.contains("declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i32)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_string_handle_roundtrip()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_string_handle_to_value_echo()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_string_to_value_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_container_handle_null_shapes()"),
        "{ir}"
    );
}

#[test]
fn scalar_echo_probe_ir_renders_64_bit_usize_helper_signatures() {
    let ir = native_runtime_scalar_echo_probe_ir_for_target(NativeRuntimeIrTarget::Pointer64);

    assert!(
        ir.contains("declare i64 @phpc_native_scalar_echo_len(%phpc.NativeScalarValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeByteBuffer = type { ptr, i64, i64 }"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_scalar_echo_write(%phpc.NativeScalarValue, ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_scalar_echo_len()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_scalar_echo_owned_bytes()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeByteBuffer @phpc_native_byte_buffer_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_byte_buffer_from_bytes()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_handle_roundtrip()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_handle_to_value_echo()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeArrayHandle @phpc_native_array_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_container_handle_null_shapes()"),
        "{ir}"
    );
}

#[test]
fn normal_print_string_emit_ir_lowers_through_runtime_value_stdout_helper() {
    let ir = emit_ir_source("<?php\n$label = \"runtime helper\";\nprint $label;\n").unwrap();

    assert!(
        ir.contains("%phpc.NativeStringHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeValueHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeDiagnosticHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 14)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_value_free"), "{ir}");
    assert!(
        ir.contains("call void @phpc_native_diagnostic_free"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_string_free"), "{ir}");
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @.str.0)"),
        "{ir}"
    );
}

#[test]
fn normal_echo_string_emit_ir_lowers_through_runtime_value_stdout_helper() {
    let ir = emit_ir_source("<?php\n$label = \"echo helper\";\necho $label;\n").unwrap();

    assert!(
        ir.contains("%phpc.NativeStringHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeValueHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeDiagnosticHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 11)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_value_free"), "{ir}");
    assert!(
        ir.contains("call void @phpc_native_diagnostic_free"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_string_free"), "{ir}");
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @.str.0)"),
        "{ir}"
    );
}

#[test]
fn known_length_string_pointer_emit_ir_lowers_through_runtime_value_stdout_helper() {
    let ir = emit_ir_source(
        "<?php\n$flag = (1 + 2) === 3;\n$label = $flag ? \"left\" : \"stay\";\necho $label;\n",
    )
    .unwrap();

    assert!(
        ir.contains("%phpc.NativeStringHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeValueHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeDiagnosticHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %tmp2, i64 4)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_value_free"), "{ir}");
    assert!(
        ir.contains("call void @phpc_native_diagnostic_free"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_string_free"), "{ir}");
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr %tmp2)"),
        "{ir}"
    );
}

#[test]
fn mixed_length_string_pointer_emit_ir_lowers_selected_length_through_runtime_value_stdout_helper()
{
    let ir = emit_ir_source(
        "<?php\n$flag = (1 + 2) === 3;\n$label = $flag ? \"alpha\" : \"beta\";\necho $label;\n",
    )
    .unwrap();

    assert!(
        ir.contains("select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(ir.contains("select i1 %tmp1, i64 5, i64 4"), "{ir}");
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %tmp2, i64 %tmp3)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr %tmp2)"),
        "{ir}"
    );
}

#[test]
fn normal_print_string_emit_ir_cli_snapshot_uses_runtime_value_stdout_helper() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1585/native_print_string_runtime_helper_lowering.php");
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

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1585/native_print_string_runtime_helper_lowering.cli"),
    )
    .expect("native print runtime helper CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
    assert!(actual.contains("phpc_native_string_from_bytes"), "{actual}");
}

#[test]
fn normal_echo_string_emit_ir_cli_snapshot_uses_runtime_value_stdout_helper() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1591/native_echo_string_runtime_helper_lowering.php");
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

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1591/native_echo_string_runtime_helper_lowering.cli"),
    )
    .expect("native echo runtime helper CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
    assert!(actual.contains("phpc_native_string_from_bytes"), "{actual}");
    assert!(!actual.contains("phpc_native_value_echo_bytes"), "{actual}");
}

#[test]
fn known_length_string_pointer_emit_ir_cli_snapshot_uses_runtime_value_stdout_helper() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1597/native_known_length_string_pointer_runtime_helper.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone1597/native_known_length_string_pointer_runtime_helper.cli",
    ))
    .expect("native known-length string-pointer runtime helper CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
    assert!(
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp2, i64 4)"),
        "{actual}"
    );
    assert!(
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp11, i64 4)"),
        "{actual}"
    );
    assert!(
        actual.contains("phpc_native_value_from_string_with_diagnostic"),
        "{actual}"
    );
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
}

#[test]
fn selected_length_string_pointer_emit_ir_cli_snapshot_uses_runtime_value_stdout_helper() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone1603/native_selected_length_string_pointer_runtime_helper.php",
    );
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone1603/native_selected_length_string_pointer_runtime_helper.cli",
    ))
    .expect("native selected-length string-pointer runtime helper CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
    assert!(
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp2, i64 %tmp3)"),
        "{actual}"
    );
    assert!(
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp12, i64 %tmp13)"),
        "{actual}"
    );
    assert!(
        actual.contains("phpc_native_value_from_string_with_diagnostic"),
        "{actual}"
    );
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
    assert!(
        !actual.contains("@printf(ptr @.fmt_str, ptr %tmp2)"),
        "{actual}"
    );
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
