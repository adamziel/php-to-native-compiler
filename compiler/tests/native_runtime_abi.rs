use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::{
    emit_ir_source, error::Phase, native_runtime_scalar_echo_probe_ir,
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
fn generated_ir_blocks_scalar_cast_builtins_at_shared_value_cast_boundary() {
    for source in [
        "<?php\n$payload = \"ABC\";\necho strval($payload);\n",
        "<?php\necho boolval(\"A\");\n",
        "<?php\n$payload = \" -12.8 \";\necho floatval($payload);\n",
        "<?php\n$payload = \"2.5\";\necho doubleval($payload);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert!(
            error.message.contains(
                "LLVM cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), and (array) casts plus strval(), boolval(), floatval(), and doubleval()"
            ),
            "{source}: {}",
            error.message
        );
    }
}

#[test]
fn generated_ir_blocks_filesystem_path_builtins_at_shared_boundary() {
    let cases = [
        ("realpath cache get", "<?php\nrealpath_cache_get();\n"),
        (
            "realpath cache size",
            "<?php\necho realpath_cache_size();\n",
        ),
    ];

    for (label, source) in cases {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen, "{label}");
        assert!(
            error.message.contains(
                "LLVM filesystem-path builtin lowering rejects realpath_cache_get() and realpath_cache_size()"
            ),
            "{label}: {}",
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
        ir.contains(
            "%phpc.NativeStringConversionResult = type { %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }"
        ),
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
        ir.contains("%phpc.NativeRequestStateHandle = type { ptr }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeRequestStateOperationResult = type { %phpc.NativeValueHandle, %phpc.NativeArrayHandle, i8, i8, i8, i8 }"),
        "{ir}"
    );
    assert!(
        ir.contains("%phpc.NativeRequestStateKeyResult = type { %phpc.NativeByteBuffer, i8 }"),
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
            "declare %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle)"
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
            "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)"
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
        ir.contains("declare i64 @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_diagnostic_contains_severity(%phpc.NativeDiagnosticHandle, i8)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_diagnostic_severity_is_known(i8)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i32 @phpc_native_diagnostic_severity_mask(i8)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i32 @phpc_native_diagnostic_error_control_suppression_mask()"),
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
            "declare %phpc.NativeValueHandle @phpc_native_array_read_int(%phpc.NativeArrayHandle, i64)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_array_free(%phpc.NativeArrayHandle)"),
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
        ir.contains("declare %phpc.NativeRequestStateHandle @phpc_native_request_state_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateHandle @phpc_native_request_state_empty()"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i1 @phpc_native_request_state_is_null(%phpc.NativeRequestStateHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateKeyResult @phpc_native_request_state_key_from_scalar(%phpc.NativeScalarValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateOperationResult @phpc_native_request_state_superglobal_operation(%phpc.NativeRequestStateHandle, i8, ptr, i64, ptr, i64, i8)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_request_state_operation_result_free(%phpc.NativeRequestStateOperationResult)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_request_state_free(%phpc.NativeRequestStateHandle)"),
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
        ir.contains("define i64 @phpc_probe_string_to_value_diagnostic_branch()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_to_string_conversion_result()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_reference_string_conversion_diagnostic()"),
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
        ir.contains("define i1 @phpc_probe_request_state_handle_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_request_state_empty_missing_value()"),
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
        ir.contains("call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes"),
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
            "call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %conversion)"
        ),
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
        ir.contains(
            "call i64 @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle %diagnostic, i64 0)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call i1 @phpc_native_diagnostic_contains_severity(%phpc.NativeDiagnosticHandle %diagnostic, i8 3)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_diagnostic_severity_is_known(i8 %severity)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 @phpc_native_diagnostic_severity_mask(i8 %severity)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 @phpc_native_diagnostic_error_control_suppression_mask()"),
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
        ir.contains("declare i32 @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle, i32)"
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
        ir.contains("define i32 @phpc_probe_reference_string_conversion_diagnostic()"),
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
        ir.contains("define i1 @phpc_probe_request_state_handle_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateOperationResult @phpc_native_request_state_superglobal_operation(%phpc.NativeRequestStateHandle, i8, ptr, i32, ptr, i32, i8)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_request_state_empty_missing_value()"),
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
            "declare %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_value_to_string_conversion_result()"),
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
        ir.contains("declare i64 @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle, i64)"
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
        ir.contains(
            "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_reference_string_conversion_diagnostic()"),
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
            "declare i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle, %phpc.NativeScalarValue)"
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
        ir.contains("declare %phpc.NativeRequestStateHandle @phpc_native_request_state_null()"),
        "{ir}"
    );
    assert!(
        ir.contains("define i1 @phpc_probe_request_state_handle_null_shape()"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeRequestStateOperationResult @phpc_native_request_state_superglobal_operation(%phpc.NativeRequestStateHandle, i8, ptr, i64, ptr, i64, i8)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_request_state_empty_missing_value()"),
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
            "declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
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
        ir.contains("call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_echo_stdout"),
        "{ir}"
    );
    assert!(
        ir.contains("br i1 %tmp4, label %native_report_diagnostic.0, label %native_echo_value.1"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_diagnostic_message_stderr"),
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
            "declare i64 @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
        ),
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
    assert!(
        ir.contains("br i1 %tmp4, label %native_report_diagnostic.0, label %native_echo_value.1"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_diagnostic_message_stderr"),
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
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp15, i64 4)"),
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
        actual.contains("@phpc_native_string_from_bytes(ptr %tmp16, i64 %tmp17)"),
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
