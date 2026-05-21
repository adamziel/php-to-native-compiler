use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::{
    codegen::emit_native_executable_c_source, emit_ir_source, error::Phase,
    native_runtime_scalar_echo_probe_ir, native_runtime_scalar_echo_probe_ir_for_target, parse,
    NativeRuntimeIrTarget,
};

const LLVM_REQUEST_SUPERGLOBAL_REJECTION: &str = "LLVM request-superglobal lowering rejects $_SERVER, $_COOKIE, $_GET, $_POST, $_REQUEST, $_FILES, and $_SESSION until native request-state storage, SAPI population, variables_order policy, upload metadata, session storage, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded request superglobal behavior";
const ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION: &str = "assembly request-superglobal lowering rejects $_SERVER, $_COOKIE, $_GET, $_POST, $_REQUEST, $_FILES, and $_SESSION until native request-state storage, SAPI population, variables_order policy, upload metadata, session storage, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded request superglobal behavior";

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
fn native_request_superglobal_builtin_consumers_share_request_blocker() {
    for source in [
        "<?php\necho array_values($_GET);\n",
        "<?php\necho array_key_exists($_GET['preview'], []);\n",
        "<?php\necho array_key_exists('preview', $_GET);\n",
        "<?php\nheader($_SERVER['SCRIPT_NAME']);\n",
        "<?php\necho constant($_POST['action']);\n",
        "<?php\necho file_get_contents($_REQUEST['template']);\n",
        "<?php\necho get_class($_GET['class']);\n",
        "<?php\necho stream_get_contents($_FILES['upload']);\n",
        "<?php\nexit($_COOKIE['wordpress_test_cookie']);\n",
    ] {
        let ir_error = emit_ir_source(source).unwrap_err();
        assert_eq!(ir_error.phase, Phase::Codegen, "{source}");
        assert_eq!(
            ir_error.message, LLVM_REQUEST_SUPERGLOBAL_REJECTION,
            "{source}"
        );

        let program = parse(source).unwrap();
        let c_error = emit_native_executable_c_source(&program).unwrap_err();
        assert_eq!(c_error.phase, Phase::Codegen, "{source}");
        assert_eq!(
            c_error.message, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION,
            "{source}"
        );
    }
}

#[test]
fn native_request_superglobal_member_and_statement_consumers_share_request_blocker() {
    for source in [
        "<?php\n$obj->method($_POST['action']);\n",
        "<?php\n$_GET['class']::boot();\n",
        "<?php\n$obj->{$_POST['action']} = 'x';\n",
        "<?php\n$_GET['class']::$prop = 1;\n",
        "<?php\nreturn $_GET['preview'];\n",
        "<?php\nthrow $_POST['action'];\n",
        "<?php\nglobal $_COOKIE;\n",
        "<?php\ninclude $_REQUEST['template'];\n",
    ] {
        let ir_error = emit_ir_source(source).unwrap_err();
        assert_eq!(ir_error.phase, Phase::Codegen, "{source}");
        assert_eq!(
            ir_error.message, LLVM_REQUEST_SUPERGLOBAL_REJECTION,
            "{source}"
        );

        let program = parse(source).unwrap();
        let c_error = emit_native_executable_c_source(&program).unwrap_err();
        assert_eq!(c_error.phase, Phase::Codegen, "{source}");
        assert_eq!(
            c_error.message, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION,
            "{source}"
        );
    }
}

#[test]
fn generated_ir_blocks_debug_output_builtins_at_shared_value_boundary() {
    for source in [
        "<?php\n$payload = \"A\\0B\";\nvar_dump($payload, 42);\n",
        "<?php\n$payload = \"A\\0B\";\nprint_r($payload, false);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen, "{source}");
        assert!(
            error
                .message
                .contains("LLVM debug-output builtin lowering rejects var_dump() and print_r()"),
            "{source}: {}",
            error.message
        );
    }
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
        ir.contains(
            "%phpc.NativeStringConversionResult = type { %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }"
        ),
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
        ir.contains("%phpc.NativeArrayKeySnapshotHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeArrayEntrySnapshotHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeArrayKeyMetadata = type { i8, [7 x i8], i64, i64 }"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "%phpc.NativeArrayKeyMaterializationResult = type { i8, [7 x i8], i64, %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeClassMetadataHandle = type { ptr }"),
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
        ir.contains("%phpc.NativeRequestStateHandle = type { ptr }"),
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
        ir.contains("declare i1 @phpc_native_string_truthy(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_string_free(%phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)"
        ),
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
            "declare %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_clone(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_value_truthy(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_string_len(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_string_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_string_clone_bytes(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_string_clone_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)"
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
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_echo_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_serialize_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_var_dump_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_value_print_r_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)"
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
            "declare %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringConversionResult @phpc_native_value_text_bytes(%phpc.NativeValueHandle, i8)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_value_text_membership_with_diagnostic(%phpc.NativeValueHandle, i8, ptr, ptr, i64, i1, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_string_array_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i64, i8, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult)"
        ),
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
        ir.contains(
            "declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
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
        ir.contains("declare %phpc.NativeArrayHandle @phpc_native_array_empty()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_array_len(%phpc.NativeArrayHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_array_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i8, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle, %phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_append_value(%phpc.NativeArrayHandle, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_write_int_scalar(%phpc.NativeArrayHandle, i64, %phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_write_int_value(%phpc.NativeArrayHandle, i64, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_write_string_scalar(%phpc.NativeArrayHandle, %phpc.NativeStringHandle, %phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_write_string_value(%phpc.NativeArrayHandle, %phpc.NativeStringHandle, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_read_int(%phpc.NativeArrayHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_read_string(%phpc.NativeArrayHandle, %phpc.NativeStringHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeArrayKeySnapshotHandle @phpc_native_array_key_snapshot(%phpc.NativeArrayHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_key_snapshot_is_null(%phpc.NativeArrayKeySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_array_key_snapshot_len(%phpc.NativeArrayKeySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeArrayKeyMetadata @phpc_native_array_key_snapshot_key_at(%phpc.NativeArrayKeySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringHandle @phpc_native_array_key_snapshot_string_clone_at(%phpc.NativeArrayKeySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare void @phpc_native_array_key_snapshot_free(%phpc.NativeArrayKeySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeArrayKeyMaterializationResult @phpc_native_value_to_array_key(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_value_array_key_exists_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i8, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringConversionResult @phpc_native_array_key_materialization_text_bytes(%phpc.NativeArrayKeyMaterializationResult, i8)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_key_materialization_to_value_with_diagnostic(%phpc.NativeArrayKeyMaterializationResult, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare void @phpc_native_array_key_materialization_result_free(%phpc.NativeArrayKeyMaterializationResult)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeArrayEntrySnapshotHandle @phpc_native_array_entry_snapshot(%phpc.NativeArrayHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_entry_snapshot_is_null(%phpc.NativeArrayEntrySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_array_entry_snapshot_len(%phpc.NativeArrayEntrySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeArrayKeyMetadata @phpc_native_array_entry_snapshot_key_at(%phpc.NativeArrayEntrySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringHandle @phpc_native_array_entry_snapshot_string_clone_at(%phpc.NativeArrayEntrySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_key_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_key_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_value_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare void @phpc_native_array_entry_snapshot_free(%phpc.NativeArrayEntrySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_array_free(%phpc.NativeArrayHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_null()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_class_metadata_is_null(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_name(ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_string(%phpc.NativeStringHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_class_metadata_name_len(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_class_metadata_name_clone_bytes(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare void @phpc_native_class_metadata_free(%phpc.NativeClassMetadataHandle)"
        ),
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
        ir.contains(
            "declare %phpc.NativeObjectHandle @phpc_native_object_alloc(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_object_class_name_len(%phpc.NativeObjectHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeByteBuffer @phpc_native_object_class_name_clone_bytes(%phpc.NativeObjectHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeClassMetadataHandle @phpc_native_object_class_metadata_clone(%phpc.NativeObjectHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_object_free(%phpc.NativeObjectHandle)"),
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
        ir.contains("declare i1 @phpc_native_reference_is_empty(%phpc.NativeReferenceHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeReferenceHandle @phpc_native_reference_from_scalar(%phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeReferenceHandle @phpc_native_reference_from_value(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeReferenceHandle @phpc_native_reference_clone(%phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_reference_write_scalar(%phpc.NativeReferenceHandle, %phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_reference_write_value(%phpc.NativeReferenceHandle, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_reference_write_reference(%phpc.NativeReferenceHandle, %phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_reference_free(%phpc.NativeReferenceHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateHandle @phpc_native_request_state_null()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_request_state_is_null(%phpc.NativeRequestStateHandle)"
        ),
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
        ir.contains("define i64 @phpc_probe_value_string_byte_diagnostics()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_formatter_diagnostics()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_string_truthy_boundaries()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_native_value_truthy_clone()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic_branch()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_to_string_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_to_text_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_text_membership()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_string_array_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_array_len_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_key_to_value_materialization()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_array_key_exists()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_array_query_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeStringConversionResult @phpc_native_value_text_bytes"),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_value_text_membership_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_key_text_view_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeArrayKeyMaterializationResult @phpc_native_value_to_array_key"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringConversionResult @phpc_native_array_key_materialization_text_bytes"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_array_key_materialization_to_value_with_diagnostic"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_container_handle_null_shapes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_handle_empty_len()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_handle_append_read()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_key_snapshot_order()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_entry_snapshot_value_routes()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_key_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, i64 0)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_value_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, i64 1)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_object_class_metadata_alloc_name()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_reference_cell_roundtrip()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle %reference)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call i1 @phpc_native_reference_is_empty(%phpc.NativeReferenceHandle %reference)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeReferenceHandle @phpc_native_reference_clone(%phpc.NativeReferenceHandle %value_reference)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call i1 @phpc_native_reference_write_reference(%phpc.NativeReferenceHandle %reference, %phpc.NativeReferenceHandle %cloned_reference)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_reference_string_conversion_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %value_reference)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_request_state_handle_null_shape()"),
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
        ir.contains(
            "call i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("br i1 %value_failed, label %report_diagnostic, label %echo_value"),
        "{ir}"
    );
    assert!(
        ir.contains("phi i64 [ %diagnostic_len, %report_diagnostic ], [ %written, %echo_value ]"),
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
        ir.contains("declare i1 @phpc_native_string_truthy(ptr, i32)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_string_truthy_boundaries()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_native_value_truthy_clone()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_string_to_value_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_string_to_value_diagnostic_branch()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_to_string_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_to_text_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i32 @phpc_native_value_string_len(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_value_string_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_string_byte_diagnostics()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_formatter_diagnostics()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_string_array_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_value_array_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 @phpc_native_value_array_len_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr, i32)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_array_key_to_value_materialization()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_array_key_exists()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_array_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_value_array_query_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_array_key_text_view_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("phi i32 [ %diagnostic_len, %report_diagnostic ], [ %written, %echo_value ]"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_container_handle_null_shapes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_array_handle_empty_len()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_array_handle_append_read()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_array_entry_snapshot_len(%phpc.NativeArrayEntrySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeArrayKeyMetadata @phpc_native_array_entry_snapshot_key_at(%phpc.NativeArrayEntrySnapshotHandle, i32)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_array_entry_snapshot_value_routes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_reference_string_conversion_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_name(ptr, i32)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i32 @phpc_native_class_metadata_name_len(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_object_class_metadata_alloc_name()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_request_state_handle_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i32, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i32)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_symbol_table_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_symbol_table_write_read()"),
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
        ir.contains("declare i1 @phpc_native_string_truthy(ptr, i64)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_string_truthy_boundaries()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_native_value_truthy_clone()"),
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
            "declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic_branch()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_to_string_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_to_text_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_value_string_len(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_string_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_string_byte_diagnostics()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_formatter_diagnostics()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_string_array_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_array_len_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_key_to_value_materialization()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_array_key_exists()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_array_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_array_query_operation()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_key_text_view_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeArrayHandle @phpc_native_array_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeArrayHandle @phpc_native_array_empty()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i64 @phpc_native_array_len(%phpc.NativeArrayHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_array_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle, %phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_write_int_scalar(%phpc.NativeArrayHandle, i64, %phpc.NativeScalarValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_array_write_string_value(%phpc.NativeArrayHandle, %phpc.NativeStringHandle, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_read_int(%phpc.NativeArrayHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_array_read_string(%phpc.NativeArrayHandle, %phpc.NativeStringHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_container_handle_null_shapes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_handle_empty_len()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_handle_append_read()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_array_entry_snapshot_len(%phpc.NativeArrayEntrySnapshotHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_array_entry_snapshot_value_routes()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_reference_string_conversion_diagnostic()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_name(ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_class_metadata_name_len(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeObjectHandle @phpc_native_object_alloc(%phpc.NativeClassMetadataHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_object_class_metadata_alloc_name()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateHandle @phpc_native_request_state_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_request_state_handle_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeSymbolTableHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_is_null(%phpc.NativeSymbolTableHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_symbol_table_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_symbol_table_write_read()"),
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
        ir.contains("%phpc.NativeSymbolTableHandle = type { ptr }"),
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
        ir.contains("declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_write"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_symbol_table_read"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "%phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 14)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %tmp0)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_write"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_symbol_table_read"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_value_free"), "{ir}");
    assert!(ir.contains("call void @phpc_native_string_free"), "{ir}");
    assert!(
        ir.contains("call void @phpc_native_symbol_table_free"),
        "{ir}"
    );
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
        ir.contains("%phpc.NativeSymbolTableHandle = type { ptr }"),
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
        ir.contains("declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 11)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %tmp0)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_write"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_symbol_table_read"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_value_free"), "{ir}");
    assert!(ir.contains("call void @phpc_native_string_free"), "{ir}");
    assert!(
        ir.contains("call void @phpc_native_symbol_table_free"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @.str.0)"),
        "{ir}"
    );
}

#[test]
fn simple_string_variable_emit_ir_matches_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$label = \"sym\";\necho $label;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [4 x i8] c"sym\00"
@.str.1 = private unnamed_addr constant [6 x i8] c"label\00"
@.str.2 = private unnamed_addr constant [6 x i8] c"label\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 3)
  %tmp1 = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %tmp0)
  %tmp2 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 5, %phpc.NativeValueHandle %tmp1)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp1)
  call void @phpc_native_string_free(%phpc.NativeStringHandle %tmp0)
  %tmp3 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.2, i64 5)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp3)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp3)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
    assert!(
        !ir.contains("call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp1)"),
        "{ir}"
    );
}

#[test]
fn simple_integer_variable_emit_ir_matches_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$n = 42;\necho $n;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [2 x i8] c"n\00"
@.str.1 = private unnamed_addr constant [2 x i8] c"n\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0
  %tmp1 = insertvalue %phpc.NativeScalarValue %tmp0, i64 42, 3
  %tmp2 = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %tmp1)
  %tmp3 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.0, i64 1, %phpc.NativeValueHandle %tmp2)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp2)
  %tmp4 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 1)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp4)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp4)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
}

#[test]
fn simple_boolean_variable_emit_ir_matches_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$flag = true;\necho $flag;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [5 x i8] c"flag\00"
@.str.1 = private unnamed_addr constant [5 x i8] c"flag\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 1, 0
  %tmp1 = insertvalue %phpc.NativeScalarValue %tmp0, i8 1, 1
  %tmp2 = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %tmp1)
  %tmp3 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.0, i64 4, %phpc.NativeValueHandle %tmp2)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp2)
  %tmp4 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 4)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp4)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp4)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
}

#[test]
fn string_variable_copy_emit_ir_reads_and_writes_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$a = \"sym\";\n$b = $a;\necho $b;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [4 x i8] c"sym\00"
@.str.1 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.2 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.3 = private unnamed_addr constant [2 x i8] c"b\00"
@.str.4 = private unnamed_addr constant [2 x i8] c"b\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 3)
  %tmp1 = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %tmp0)
  %tmp2 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 1, %phpc.NativeValueHandle %tmp1)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp1)
  call void @phpc_native_string_free(%phpc.NativeStringHandle %tmp0)
  %tmp3 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.2, i64 1)
  %tmp4 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.3, i64 1, %phpc.NativeValueHandle %tmp3)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp3)
  %tmp5 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.4, i64 1)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp5)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp5)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
}

#[test]
fn scalar_string_reassignment_emit_ir_overwrites_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$a = \"old\";\n$a = \"new\";\necho $a;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [4 x i8] c"old\00"
@.str.1 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.2 = private unnamed_addr constant [4 x i8] c"new\00"
@.str.3 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.4 = private unnamed_addr constant [2 x i8] c"a\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.0, i64 3)
  %tmp1 = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %tmp0)
  %tmp2 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 1, %phpc.NativeValueHandle %tmp1)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp1)
  call void @phpc_native_string_free(%phpc.NativeStringHandle %tmp0)
  %tmp3 = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.2, i64 3)
  %tmp4 = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %tmp3)
  %tmp5 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.3, i64 1, %phpc.NativeValueHandle %tmp4)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp4)
  call void @phpc_native_string_free(%phpc.NativeStringHandle %tmp3)
  %tmp6 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.4, i64 1)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp6)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp6)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        2
    );
    assert!(
        !ir.contains("call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp4)"),
        "{ir}"
    );
}

fn assert_symbol_table_ir_declares_liveness_helpers(ir: &str) {
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_write"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_symbol_table_read"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_isset"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_empty"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_unset"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_symbol_table_free"),
        "{ir}"
    );
}

#[test]
fn chained_variable_copy_emit_ir_reads_written_symbol_slots_through_third_variable() {
    let ir = emit_ir_source("<?php\n$a = \"sym\";\n$b = $a;\n$c = $b;\necho $c;\n").unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        3,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        3,
        "{ir}"
    );

    let read_a = ir
        .find("call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.2, i64 1)")
        .expect("copy from $a reads the written $a slot");
    let write_b = ir
        .find("call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.3, i64 1")
        .expect("copy to $b writes the read value into the $b slot");
    let read_b = ir
        .find("call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.4, i64 1)")
        .expect("copy from $b reads the previously written $b slot");
    let write_c = ir
        .find("call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.5, i64 1")
        .expect("copy to $c writes the read value into the $c slot");
    let read_c = ir
        .find("call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.6, i64 1)")
        .expect("echo reads the final $c slot");
    let echo_c = ir
        .find("call i64 @phpc_native_value_echo_stdout")
        .expect("echo prints the native value read from $c");

    assert!(read_a < write_b, "{ir}");
    assert!(write_b < read_b, "{ir}");
    assert!(read_b < write_c, "{ir}");
    assert!(write_c < read_c, "{ir}");
    assert!(read_c < echo_c, "{ir}");
}

#[test]
fn direct_variable_read_emit_ir_uses_symbol_table_across_print_copy_echo_and_missing() {
    let ir = emit_ir_source(
        "<?php\n$source = \"S\";\nprint $source;\n$copy = $source;\necho $copy;\necho $missing;\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i64 @phpc_native_value_echo_stdout")
            .count(),
        3,
        "{ir}"
    );
    for value in ["S", "source", "copy", "missing"] {
        assert!(ir.contains(&format!("c\"{value}\\00\"")), "{ir}");
    }
}

#[test]
fn native_expression_value_flows_through_assignment_print_liveness_and_conditions() {
    let ir = emit_ir_source(
        "<?php\n$a = \"0\";\n$b = $a;\nprint $b;\necho isset($b) ? \"I\" : \"M\";\necho empty($b) ? \"E\" : \"N\";\necho $b ? \"T\" : \"F\";\necho !$missing ? \"Z\" : \"Q\";\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_isset")
            .count(),
        1,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_empty")
            .count(),
        3,
        "{ir}"
    );
    assert!(
        ir.contains("xor i1") && ir.contains("true"),
        "native direct-local condition should invert empty() into truthiness: {ir}"
    );

    let writes: Vec<_> = ir
        .match_indices("call i1 @phpc_native_symbol_table_write")
        .map(|(index, _)| index)
        .collect();
    let reads: Vec<_> = ir
        .match_indices("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
        .map(|(index, _)| index)
        .collect();
    let isset = ir
        .find("call i1 @phpc_native_symbol_table_isset")
        .expect("isset($b) uses the native expression carrier");
    let empties: Vec<_> = ir
        .match_indices("call i1 @phpc_native_symbol_table_empty")
        .map(|(index, _)| index)
        .collect();

    assert!(writes[0] < reads[0], "{ir}");
    assert!(reads[0] < writes[1], "{ir}");
    assert!(writes[1] < reads[1], "{ir}");
    assert!(reads[1] < isset, "{ir}");
    assert!(isset < empties[0], "{ir}");
    assert!(empties[0] < empties[1], "{ir}");
    assert!(empties[1] < empties[2], "{ir}");
}

#[test]
fn direct_local_assignments_materialize_literal_slot_and_missing_rhs_families() {
    let ir = emit_ir_source(
        "<?php\n$string = \"S\";\n$int = 7;\n$null = null;\n$copy = $string;\n$missing_copy = $missing;\necho $copy;\necho $missing_copy;\necho $null;\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        5,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        5,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_value_from_scalar")
            .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string"),
        "{ir}"
    );
    for value in [
        "S",
        "string",
        "int",
        "null",
        "copy",
        "missing_copy",
        "missing",
    ] {
        assert!(ir.contains(&format!("c\"{value}\\00\"")), "{ir}");
    }

    let writes: Vec<_> = ir
        .match_indices("call i1 @phpc_native_symbol_table_write")
        .map(|(index, _)| index)
        .collect();
    let reads: Vec<_> = ir
        .match_indices("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
        .map(|(index, _)| index)
        .collect();

    assert!(writes[2] < reads[0], "{ir}");
    assert!(reads[0] < writes[3], "{ir}");
    assert!(writes[3] < reads[1], "{ir}");
    assert!(reads[1] < writes[4], "{ir}");
    assert!(writes[4] < reads[2], "{ir}");
}

#[test]
fn native_value_handle_result_materializes_owned_string_scalar_and_symbol_sources() {
    let ir = emit_ir_source(
        "<?php\n$text = \"T\";\n$number = 9;\n$flag = true;\n$copy = $text;\n$missing_copy = $missing;\nprint $copy;\necho $number;\necho $flag ? \"Y\" : \"N\";\n",
    )
    .unwrap();

    assert!(
        ir.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle"
        ),
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_value_from_scalar")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_empty")
            .count(),
        1,
        "{ir}"
    );

    let scalar = ir
        .find("call %phpc.NativeValueHandle @phpc_native_value_from_scalar")
        .expect("scalar expressions materialize through the native value result");
    let read = ir
        .find("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
        .expect("direct locals materialize through the native value result");
    let cleanups: Vec<_> = ir
        .match_indices("call void @phpc_native_value_free")
        .map(|(index, _)| index)
        .collect();

    assert!(cleanups.iter().any(|cleanup| scalar < *cleanup), "{ir}");
    assert!(cleanups.iter().any(|cleanup| read < *cleanup), "{ir}");
}

#[test]
fn native_value_handle_result_keeps_request_array_object_and_resource_blockers() {
    let cases = [
        ("<?php\n$copy = $_SERVER;\n", "request-superglobal"),
        ("<?php\n$copy = [];\n", "array lowering rejects arrays"),
        (
            "<?php\n$copy = $box->name;\n",
            "object-property lowering rejects instance property",
        ),
        (
            "<?php\n$copy = fopen(\"php://memory\", \"w\");\n",
            "stream-resource lowering rejects fopen()",
        ),
    ];

    for (source, expected) in cases {
        let error = emit_ir_source(source).unwrap_err();
        assert!(
            error.message.contains(expected),
            "expected `{expected}` in `{}`",
            error.message
        );
    }
}

#[test]
fn multi_variable_isset_emit_ir_combines_each_direct_local_liveness_check() {
    let ir = emit_ir_source(
        "<?php\n$first = 1;\n$second = 2;\necho isset($first, $second) ? \"Y\" : \"N\";\nunset($second);\necho isset($first, $second) ? \"Y\" : \"N\";\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_isset")
            .count(),
        4,
        "{ir}"
    );
    assert_eq!(ir.matches(" = and i1 ").count(), 2, "{ir}");
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_unset")
            .count(),
        1,
        "{ir}"
    );
}

#[test]
fn read_after_unset_variable_copy_emit_ir_transfers_missing_null_independently() {
    let ir = emit_ir_source(
        "<?php\n$a = \"old\";\nunset($a);\n$b = $a;\n$a = \"new\";\necho $b;\necho $a;\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        3,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        3,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_unset")
            .count(),
        1,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i64 @phpc_native_value_echo_stdout")
            .count(),
        2,
        "{ir}"
    );
    for value in ["old", "new", "a", "b"] {
        assert!(ir.contains(&format!("c\"{value}\\00\"")), "{ir}");
    }
}

#[test]
fn integer_variable_copy_emit_ir_reads_and_writes_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$a = 42;\n$b = $a;\necho $b;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.1 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.2 = private unnamed_addr constant [2 x i8] c"b\00"
@.str.3 = private unnamed_addr constant [2 x i8] c"b\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0
  %tmp1 = insertvalue %phpc.NativeScalarValue %tmp0, i64 42, 3
  %tmp2 = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %tmp1)
  %tmp3 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.0, i64 1, %phpc.NativeValueHandle %tmp2)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp2)
  %tmp4 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 1)
  %tmp5 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.2, i64 1, %phpc.NativeValueHandle %tmp4)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp4)
  %tmp6 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.3, i64 1)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp6)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp6)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
}

#[test]
fn boolean_variable_copy_emit_ir_reads_and_writes_symbol_table_snapshot() {
    let ir = emit_ir_source("<?php\n$a = true;\n$b = $a;\necho $b;\n").unwrap();
    let _expected = r#"; generated by phpc milestone 1
%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }
%phpc.NativeStringHandle = type { ptr }
%phpc.NativeValueHandle = type { ptr }
%phpc.NativeDiagnosticHandle = type { ptr }
%phpc.NativeSymbolTableHandle = type { ptr }
declare i32 @printf(ptr, ...)
declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, i64)
declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)
declare i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)
declare void @phpc_native_value_free(%phpc.NativeValueHandle)
declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)
declare void @phpc_native_string_free(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)
declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)
declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)
declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, i64)
declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)

@.fmt_int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.fmt_float = private unnamed_addr constant [3 x i8] c"%g\00"
@.fmt_str = private unnamed_addr constant [3 x i8] c"%s\00"
@.str.0 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.1 = private unnamed_addr constant [2 x i8] c"a\00"
@.str.2 = private unnamed_addr constant [2 x i8] c"b\00"
@.str.3 = private unnamed_addr constant [2 x i8] c"b\00"

define i32 @main() {
entry:
  %phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()
  %tmp0 = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 1, 0
  %tmp1 = insertvalue %phpc.NativeScalarValue %tmp0, i8 1, 1
  %tmp2 = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %tmp1)
  %tmp3 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.0, i64 1, %phpc.NativeValueHandle %tmp2)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp2)
  %tmp4 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.1, i64 1)
  %tmp5 = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.2, i64 1, %phpc.NativeValueHandle %tmp4)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp4)
  %tmp6 = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %phpc.symbols, ptr @.str.3, i64 1)
  call i64 @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %tmp6)
  call void @phpc_native_value_free(%phpc.NativeValueHandle %tmp6)
  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)
  ret i32 0
}
"#;

    assert_symbol_table_ir_declares_liveness_helpers(&ir);
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
        ir.contains("%phpc.NativeSymbolTableHandle = type { ptr }"),
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
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_scalar"),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_empty"),
        "{ir}"
    );
    assert!(ir.contains("xor i1"), "{ir}");
    assert!(ir.contains("br i1"), "{ir}");
    assert!(ir.contains(" = phi %phpc.NativeValueHandle "), "{ir}");
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.2, i64 4)"
        ) && ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.3, i64 4)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_value_free"), "{ir}");
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_write"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_symbol_table_read"),
        "{ir}"
    );
    assert!(ir.contains("call void @phpc_native_string_free"), "{ir}");
    assert!(
        ir.contains("call void @phpc_native_symbol_table_free"),
        "{ir}"
    );
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
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_scalar"),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_empty"),
        "{ir}"
    );
    assert!(ir.contains("xor i1"), "{ir}");
    assert!(ir.contains("br i1"), "{ir}");
    assert!(ir.contains(" = phi %phpc.NativeValueHandle "), "{ir}");
    assert!(
        ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.2, i64 5)"
        ) && ir.contains(
            "call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr @.str.3, i64 4)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_write"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_symbol_table_read"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr %tmp2)"),
        "{ir}"
    );
}

#[test]
fn direct_string_literal_emit_ir_stays_on_value_stdout_without_symbol_table() {
    let ir = emit_ir_source("<?php\necho \"direct helper\";\n").unwrap();

    assert!(ir.contains("phpc_native_value_echo_stdout"), "{ir}");
    assert!(ir.contains("phpc_native_string_from_bytes"), "{ir}");
    assert!(
        ir.contains("phpc_native_value_from_string(%phpc.NativeStringHandle"),
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic"),
        "{ir}"
    );
    assert!(!ir.contains("phpc_native_symbol_table_write"), "{ir}");
    assert!(!ir.contains("phpc_native_symbol_table_read"), "{ir}");
    assert!(!ir.contains("%phpc.NativeSymbolTableHandle"), "{ir}");
}

#[test]
fn native_value_handle_result_feeds_stdout_for_literal_pointer_and_symbol_sources() {
    let ir = emit_ir_source(
        "<?php\necho \"literal\";\n$flag = true;\n$label = $flag ? \"left\" : \"right\";\nprint $label;\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_value_from_string")
            .count(),
        3,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        1,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i64 @phpc_native_value_echo_stdout")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call void @phpc_native_value_free").count(),
        4,
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_symbol_table_empty"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr %tmp"),
        "{ir}"
    );

    let literal_value = ir
        .find("call %phpc.NativeValueHandle @phpc_native_value_from_string")
        .expect("literal stdout materializes through NativeValueHandleResult");
    let literal_stdout = ir
        .find("call i64 @phpc_native_value_echo_stdout")
        .expect("literal stdout consumes the materialized handle");
    let label_read = ir
        .rfind("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
        .expect("print $label reads a native value handle");
    let label_stdout = ir
        .rfind("call i64 @phpc_native_value_echo_stdout")
        .expect("print $label consumes the read handle");

    assert!(literal_value < literal_stdout, "{ir}");
    assert!(label_read < label_stdout, "{ir}");
}

#[test]
fn native_value_handle_result_reaches_return_and_argument_frame_boundaries() {
    let return_error = emit_ir_source("<?php\n$value = \"R\";\nreturn $value;\n").unwrap_err();
    assert!(
        return_error
            .message
            .contains("user-function lowering rejects function declarations and return statements"),
        "{}",
        return_error.message
    );

    let call_error =
        emit_ir_source("<?php\n$value = \"A\";\nunknown_user_function($value, \"B\", 3);\n")
            .unwrap_err();
    assert!(
        call_error
            .message
            .contains("function-call lowering rejects function calls"),
        "{}",
        call_error.message
    );
    assert!(
        !call_error.message.contains("array lowering rejects arrays"),
        "{}",
        call_error.message
    );
}

#[test]
fn native_value_handle_result_centralizes_return_and_argument_blockers() {
    let cases = [
        ("<?php\nreturn $_SERVER;\n", "request-superglobal"),
        ("<?php\nreturn [];\n", "array lowering rejects arrays"),
        (
            "<?php\nunknown_user_function($box->name);\n",
            "object-property lowering rejects instance property",
        ),
        (
            "<?php\nunknown_user_function(fopen(\"php://memory\", \"w\"));\n",
            "stream-resource lowering rejects fopen()",
        ),
        (
            "<?php\nunknown_user_function($fn(\"x\"));\n",
            "dynamic function-call lowering rejects variable-call expressions",
        ),
        (
            "<?php\nunknown_user_function($slot = \"x\");\n",
            "mutation lowering rejects compound assignment",
        ),
    ];

    for (source, expected) in cases {
        let error = emit_ir_source(source).unwrap_err();
        assert!(
            error.message.contains(expected),
            "expected `{expected}` in `{}`",
            error.message
        );
    }
}

#[test]
fn native_value_handle_result_materializes_short_and_long_ternary_selection() {
    let ir = emit_ir_source(
        "<?php\n$flag = true;\n$value = \"V\";\n$short = $value ?: \"fallback\";\n$long = $flag ? $value : \"fallback\";\necho $short;\necho $long;\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches(" = phi %phpc.NativeValueHandle ").count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_empty")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        4,
        "{ir}"
    );
    assert!(
        ir.contains("native.select.true") && ir.contains("native.select.false"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string"),
        "{ir}"
    );

    let first_branch = ir
        .find("br i1")
        .expect("selection emits a conditional branch");
    let first_phi = ir
        .find(" = phi %phpc.NativeValueHandle ")
        .expect("selection merges branch-produced handles");
    let first_write = ir[first_phi..]
        .find("call i1 @phpc_native_symbol_table_write")
        .map(|index| first_phi + index)
        .expect("assignment writes the selected native handle");
    let first_selected_cleanup = ir[first_write..]
        .find("call void @phpc_native_value_free")
        .map(|index| first_write + index)
        .expect("selected handle is released after assignment transfer");

    assert!(first_branch < first_phi, "{ir}");
    assert!(first_phi < first_write, "{ir}");
    assert!(first_write < first_selected_cleanup, "{ir}");
}

#[test]
fn native_value_handle_result_keeps_branch_local_selection_cleanup_on_each_arm() {
    let ir = emit_ir_source(
        "<?php\n$flag = true;\n$label = $flag ? \"left\" : \"right\";\necho $label;\n",
    )
    .unwrap();

    assert_eq!(
        ir.matches(" = phi %phpc.NativeValueHandle ").count(),
        1,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_value_from_string")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call void @phpc_native_string_free").count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_symbol_table_write")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_symbol_table_read")
            .count(),
        1,
        "{ir}"
    );

    let true_block = ir
        .find("\nnative.select.true")
        .expect("selection emits a true branch block");
    let false_block = ir
        .find("\nnative.select.false")
        .expect("selection emits a false branch block");
    let merge_block = ir
        .find("\nnative.select.merge")
        .expect("selection emits a merge block");
    let string_frees: Vec<_> = ir
        .match_indices("call void @phpc_native_string_free")
        .map(|(index, _)| index)
        .collect();
    let phi = ir
        .find(" = phi %phpc.NativeValueHandle ")
        .expect("selection merges branch-produced native handles");
    let selected_write = ir[phi..]
        .find("call i1 @phpc_native_symbol_table_write")
        .map(|index| phi + index)
        .expect("selected branch handle is written after merge");
    let selected_value_free = ir[selected_write..]
        .find("call void @phpc_native_value_free")
        .map(|index| selected_write + index)
        .expect("selected branch handle is released after assignment");

    assert!(true_block < string_frees[0], "{ir}");
    assert!(string_frees[0] < false_block, "{ir}");
    assert!(false_block < string_frees[1], "{ir}");
    assert!(string_frees[1] < merge_block, "{ir}");
    assert!(merge_block < phi, "{ir}");
    assert!(phi < selected_write, "{ir}");
    assert!(selected_write < selected_value_free, "{ir}");
}

#[test]
fn native_value_handle_result_centralizes_selection_blockers() {
    let cases = [
        (
            "<?php\n$result = $_SERVER ?: \"x\";\n",
            "request-superglobal",
        ),
        (
            "<?php\n$flag = true;\n$result = $flag ? [] : \"x\";\n",
            "array lowering rejects arrays",
        ),
        (
            "<?php\n$flag = true;\n$result = $flag ? $box->name : \"x\";\n",
            "object-property lowering rejects instance property",
        ),
        (
            "<?php\n$flag = true;\n$result = $flag ? fopen(\"php://memory\", \"w\") : \"x\";\n",
            "stream-resource lowering rejects fopen()",
        ),
        (
            "<?php\n$flag = true;\n$result = $flag ? $fn(\"x\") : \"x\";\n",
            "dynamic function-call lowering rejects variable-call expressions",
        ),
        (
            "<?php\n$flag = true;\n$result = $flag ? ($slot = \"x\") : \"x\";\n",
            "mutation lowering rejects compound assignment",
        ),
    ];

    for (source, expected) in cases {
        let error = emit_ir_source(source).unwrap_err();
        assert!(
            error.message.contains(expected),
            "expected `{expected}` in `{}`",
            error.message
        );
    }
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

    let actual = render_cli_snapshot(&output);

    assert!(output.status.success(), "{actual}");
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
    assert!(actual.contains("phpc_native_string_from_bytes"), "{actual}");
    assert!(
        actual.contains("phpc_native_symbol_table_write"),
        "{actual}"
    );
    assert!(actual.contains("phpc_native_symbol_table_read"), "{actual}");
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

    let actual = render_cli_snapshot(&output);

    assert!(output.status.success(), "{actual}");
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
    assert!(actual.contains("phpc_native_string_from_bytes"), "{actual}");
    assert!(
        actual.contains("phpc_native_symbol_table_write"),
        "{actual}"
    );
    assert!(actual.contains("phpc_native_symbol_table_read"), "{actual}");
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

    let actual = render_cli_snapshot(&output);

    assert!(output.status.success(), "{actual}");
    assert!(
        actual.contains("phpc_native_symbol_table_empty"),
        "{actual}"
    );
    assert!(
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp") && actual.contains(", i64 4)"),
        "{actual}"
    );
    assert!(
        actual.contains("phpc_native_symbol_table_write"),
        "{actual}"
    );
    assert!(actual.contains("phpc_native_symbol_table_read"), "{actual}");
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

    let actual = render_cli_snapshot(&output);

    assert!(output.status.success(), "{actual}");
    assert!(
        actual.contains("phpc_native_symbol_table_empty"),
        "{actual}"
    );
    assert!(
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp") && actual.contains(", i64 %tmp"),
        "{actual}"
    );
    assert!(
        actual.contains("phpc_native_symbol_table_write"),
        "{actual}"
    );
    assert!(actual.contains("phpc_native_symbol_table_read"), "{actual}");
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

#[test]
fn native_array_handle_boundary_cli_snapshots_match_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1645/native_array_handle_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    for (mode, snapshot) in [
        (
            "--emit-ir",
            "tests/fixtures/milestone1645/native_array_handle_boundary_emit_ir.cli",
        ),
        (
            "--emit-asm",
            "tests/fixtures/milestone1645/native_array_handle_boundary_emit_asm.cli",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
            .current_dir(workspace_root)
            .args(["compile", &relative_fixture, mode])
            .output()
            .unwrap_or_else(|error| {
                panic!("failed to compile {relative_fixture} with {mode}: {error}")
            });

        let expected = fs::read_to_string(workspace_root.join(snapshot))
            .expect("native array handle boundary CLI snapshot is readable");
        let actual = render_cli_snapshot(&output);

        assert_eq!(actual, expected);
    }
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
