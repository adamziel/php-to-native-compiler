use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use php_compiler::{
    codegen::emit_native_executable_c_source, emit_asm_source, emit_ir_source, error::Phase,
    native_runtime_scalar_echo_probe_ir, native_runtime_scalar_echo_probe_ir_for_target, parse,
    NativeRuntimeIrTarget,
};

const STRING_INT_IR_SOURCE: &str = "<?php\n$payload = \"A\\0bA\\0b\";\necho strcasecmp($payload, \"a\\0B\");\necho strcmp($payload, \"A\\0c\");\necho strncmp($payload, \"A\\0bZ\", \"3\");\necho strncasecmp($payload, \"a\\0Bz\", 3);\necho ord(\"A\");\necho crc32($payload);\n";
const STRING_PREDICATE_IR_SOURCE: &str = "<?php\n$payload = \"A\\0B\";\necho str_starts_with($payload, \"A\\0\");\necho str_ends_with($payload, \"\\0B\");\necho str_contains(42, \"2\");\necho str_contains($payload, \"C\");\n";
const STRING_SEARCH_IR_SOURCE: &str = "<?php\n$payload = \"A\\0bA\\0b\";\necho strpos($payload, \"\\0b\", 2);\necho strpos($payload, \"missing\");\necho substr_count($payload, \"A\", false, \"5\");\n";
const STRING_RESULT_IR_SOURCE: &str = "<?php\n$payload = \"A\\0B\";\necho strrev($payload), \"|\";\nprint str_rot13(\"Az-09\");\necho \"|\";\necho bin2hex($payload), \"|\";\necho strtolower(\"MiXeD\"), \"|\";\necho strtoupper(\"MiXeD\"), \"|\";\necho ucfirst(\"word\"), \"|\";\necho lcfirst(\"Word\"), \"|\";\necho escapeshellarg(\"X ;\\$'Q\\\"\"), \"|\";\necho escapeshellcmd(\"X ;\\$'Q\\\"\"), \"|\";\necho strrev(42042);\n";
const VALUE_OFFSET_IR_SOURCE: &str = "<?php\n$payload = \"A\\0B\\xff\";\necho $payload[1];\necho isset($payload[2]);\necho empty($payload[3]);\necho strlen($payload[0]);\necho strcmp($payload[0], \"A\");\n";
const OUTPUT_BUFFER_RUNTIME_SOURCE: &str = "<?php\nob_start(null, strlen(\"aa\"), strlen(\"flags\"));\necho \"A\\0B\";\necho 42;\nob_get_contents();\nob_get_length();\nob_list_handlers();\nob_get_status(true);\nob_clean();\necho strtolower(\"HIDDEN\");\nob_get_clean();\nob_start();\necho \"A\";\nob_start();\necho \"B\";\nob_end_flush();\nob_get_clean();\nob_get_level();\n";

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
fn native_request_state_runtime_value_keys_share_key_result_accessors() {
    let ir = native_runtime_scalar_echo_probe_ir();

    assert!(
        ir.contains("declare %phpc.NativeRequestStateKeyResult @phpc_native_request_state_key_from_scalar(%phpc.NativeScalarValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeByteBuffer @phpc_native_request_state_key_result_buffer(%phpc.NativeRequestStateKeyResult)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i8 @phpc_native_request_state_key_result_status(%phpc.NativeRequestStateKeyResult)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeByteBuffer @phpc_native_request_state_key_result_buffer"),
        "{ir}"
    );
    assert!(
        ir.contains("call i8 @phpc_native_request_state_key_result_status"),
        "{ir}"
    );
    assert!(
        !ir.contains("extractvalue %phpc.NativeRequestStateKeyResult"),
        "{ir}"
    );
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
fn generated_ir_routes_nul_strings_through_formatter_stdout_abi() {
    let ir = emit_ir_source("<?php\n$payload = \"A\\0B\";\necho $payload, \"|\";\n").unwrap();

    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("alloca %phpc.NativeDiagnosticHandle")
            && ir.contains("@phpc_native_diagnostic_message_stderr")
            && !ir.contains("i8 0, ptr null"),
        "{ir}"
    );
    assert!(!ir.contains("phpc_native_value_echo_stdout"), "{ir}");
    assert!(
        ir.contains("@phpc_native_string_from_bytes(ptr @.str.0, i64 3)"),
        "{ir}"
    );
}

#[test]
fn native_output_buffer_builtins_share_runtime_boundary_across_backends() {
    let ir = emit_ir_source(OUTPUT_BUFFER_RUNTIME_SOURCE).unwrap();

    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_output_buffer_operation_with_diagnostic"
        ),
        "{ir}"
    );
    assert!(
        ir.matches("@phpc_native_output_buffer_operation_with_diagnostic")
            .count()
            >= 13,
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 42)"),
        "{ir}"
    );
    assert!(!ir.contains("output-buffer lowering rejects"), "{ir}");

    let program = parse(OUTPUT_BUFFER_RUNTIME_SOURCE).unwrap();
    let c_source = emit_native_executable_c_source(&program).unwrap();
    assert!(
        c_source.contains(
            "extern phpc_NativeValueHandle phpc_native_output_buffer_operation_with_diagnostic"
        ),
        "{c_source}"
    );
    assert!(
        c_source
            .matches("phpc_native_output_buffer_operation_with_diagnostic(")
            .count()
            >= 13,
        "{c_source}"
    );
    assert!(
        c_source.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{c_source}"
    );
    assert!(
        !c_source.contains("output-buffer lowering rejects"),
        "{c_source}"
    );
}

#[test]
fn generated_ir_routes_string_int_builtins_through_runtime_contract() {
    let ir = emit_ir_source(STRING_INT_IR_SOURCE).unwrap();

    let usize_type = if usize::BITS == 32 { "i32" } else { "i64" };
    assert!(
        ir.contains("%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)"),
        "{ir}"
    );
    assert!(
        ir.contains(&format!(
            "declare %phpc.NativeValueHandle @phpc_native_value_from_string_bytes_with_diagnostic(ptr, {usize_type}, ptr)"
        )),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_to_int64_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_string_int_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i64, i64, i8, i8, ptr)"
        ),
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i64 @phpc_native_value_string_int_operation_with_diagnostic")
            .count(),
        6,
        "{ir}"
    );
    assert!(
        ir.matches("call i64 @phpc_native_value_to_int64_with_diagnostic")
            .count()
            >= 2,
        "{ir}"
    );
    for tag in [0, 2, 3, 4, 5, 6] {
        assert!(ir.contains(&format!("i8 {tag}, ptr %")), "{tag}: {ir}");
    }
    assert!(
        ir.matches("call void @phpc_native_value_free").count() >= 12,
        "{ir}"
    );
    assert!(
        !ir.contains("LLVM string-int builtin lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_string_predicates_through_runtime_contract() {
    let ir = emit_ir_source(STRING_PREDICATE_IR_SOURCE).unwrap();

    assert!(
        ir.contains(
            "declare i1 @phpc_native_value_string_predicate_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_value_string_predicate_with_diagnostic")
            .count(),
        4,
        "{ir}"
    );
    for tag in [0, 1, 2] {
        assert!(ir.contains(&format!("i8 {tag}, ptr %")), "{tag}: {ir}");
    }
    assert!(
        ir.matches("call i32 (ptr, ...) @printf").count() >= 4,
        "{ir}"
    );
    assert!(
        ir.matches("call void @phpc_native_value_free").count() >= 8,
        "{ir}"
    );
    assert!(
        !ir.contains("LLVM string-predicate lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_string_search_builtins_through_value_result_contract() {
    let ir = emit_ir_source(STRING_SEARCH_IR_SOURCE).unwrap();

    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_string_search_result_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i64, i64, i8, i8, ptr)"
        ),
        "{ir}"
    );
    assert_eq!(
        ir.matches(
            "call %phpc.NativeValueHandle @phpc_native_value_string_search_result_with_diagnostic"
        )
        .count(),
        3,
        "{ir}"
    );
    assert!(
        ir.contains("i8 0, ptr %") && ir.contains("i8 1, ptr %"),
        "{ir}"
    );
    assert!(
        ir.matches("call i64 @phpc_native_value_format_stdout_with_diagnostic")
            .count()
            >= 3,
        "{ir}"
    );
    assert!(
        ir.matches("call void @phpc_native_value_free").count() >= 6,
        "{ir}"
    );
    assert!(!ir.contains("LLVM function-call lowering rejects"), "{ir}");
}

#[test]
fn generated_ir_routes_string_result_builtins_through_value_result_contract() {
    let ir = emit_ir_source(STRING_RESULT_IR_SOURCE).unwrap();

    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_string_result_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, %phpc.NativeValueHandle, i64, i64, i8, i8, ptr)"
        ),
        "{ir}"
    );
    assert_eq!(
        ir.matches(
            "call %phpc.NativeValueHandle @phpc_native_value_string_result_operation_with_diagnostic"
        )
        .count(),
        10,
        "{ir}"
    );
    for tag in [4, 5, 13, 48, 49, 53, 54, 70, 71] {
        assert!(ir.contains(&format!("i8 {tag}, ptr %")), "{tag}: {ir}");
    }
    assert!(
        ir.matches("call i64 @phpc_native_value_format_stdout_with_diagnostic")
            .count()
            >= 10,
        "{ir}"
    );
    assert!(
        ir.matches("call void @phpc_native_value_free").count() >= 20,
        "{ir}"
    );
    assert!(
        !ir.contains("LLVM string-result builtin lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_reference_type_introspection_without_value_clone_detour() {
    let ir = emit_ir_source(
        "<?php\n$payload = \"A\\0B\";\n$alias =& $payload;\necho gettype($alias);\necho get_debug_type($alias);\necho is_string($alias);\necho is_scalar($alias);\necho is_int($alias);\n",
    )
    .unwrap();

    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_type_name_with_reference_slot_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeReferenceHandle, i1, ptr)"
        ) && ir.contains(
            "declare i1 @phpc_native_value_type_predicate_with_reference_slot_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeReferenceHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert_eq!(
        ir.matches("call %phpc.NativeValueHandle @phpc_native_value_type_name_with_reference_slot_with_diagnostic")
            .count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i1 @phpc_native_value_type_predicate_with_reference_slot_with_diagnostic")
            .count(),
        3,
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeReferenceHandle @phpc_native_reference_from_value_and_free")
            && ir.contains("call %phpc.NativeReferenceHandle @phpc_native_reference_clone")
            && ir.contains("call void @phpc_native_diagnostic_free"),
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeValueHandle @phpc_native_reference_value_clone")
            && !ir.contains("call %phpc.NativeValueHandle @phpc_native_value_type_name_result")
            && !ir.contains("call i1 @phpc_native_value_type_predicate(")
            && !ir.contains("call i1 @phpc_native_reference_set_value"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_text_membership_reference_slots_without_value_clone_detour() {
    let ir = emit_ir_source(
        "<?php\n$name = \"MYSQLI_QUERY\";\n$nameRef =& $name;\n$extension = \"Json\";\n$extensionRef =& $extension;\necho function_exists($nameRef);\necho extension_loaded($extensionRef);\n",
    )
    .unwrap();

    assert!(
        ir.contains(
            "declare i1 @phpc_native_text_membership_with_reference_slot_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeReferenceHandle, i8, ptr, ptr,"
        ),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_text_membership_with_reference_slot_with_diagnostic")
            .count()
            >= 2,
        "{ir}"
    );
    assert!(ir.contains("i8 4, ptr") && ir.contains("i8 6, ptr"), "{ir}");
    for native_known_name in [
        "mysqli_query",
        "stream_get_contents",
        "is_uploaded_file",
        "spl_autoload_register",
    ] {
        assert!(
            ir.contains(native_known_name),
            "function_exists text-membership table should use the full native-known semantic family: {native_known_name}\n{ir}"
        );
    }
    assert!(
        ir.contains("call %phpc.NativeReferenceHandle @phpc_native_reference_from_value_and_free"),
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeValueHandle @phpc_native_reference_value_clone")
            && !ir.contains("LLVM function-call lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_reference_held_native_value_comparisons_through_slot_boundary() {
    let ir = emit_ir_source(
        "<?php\n$left = \"A\\0B\\xFF\";\n$leftRef =& $left;\n$right = \"A\\0B\\xFF\";\n$rightRef =& $right;\necho $leftRef == $rightRef;\necho $leftRef < \"A\\0C\";\necho $leftRef === $rightRef;\n",
    )
    .unwrap();

    assert!(
        ir.contains(
            "declare i1 @phpc_native_value_comparison_with_reference_slots_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeReferenceHandle, %phpc.NativeValueHandle, %phpc.NativeReferenceHandle, i8, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_value_comparison_with_reference_slots_with_diagnostic")
            .count()
            >= 3,
        "{ir}"
    );
    assert!(
        ir.contains(", i8 0, ptr ") && ir.contains(", i8 2, ptr ") && ir.contains(", i8 6, ptr "),
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeValueHandle @phpc_native_reference_value_clone")
            && !ir.contains("LLVM comparison lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_reference_int_operands_without_value_clone_detour() {
    let ir = emit_ir_source(
        "<?php\n$length = 2;\n$lengthRef =& $length;\n$offset = 1;\n$offsetRef =& $offset;\necho strncmp(\"abcdef\", \"abcxyz\", $lengthRef);\necho strncasecmp(\"ABCDEF\", \"abcxyz\", $lengthRef);\necho substr_count(\"abcabc\", \"a\", $offsetRef, $lengthRef);\necho strpos(\"abcabc\", \"c\", $offsetRef);\n",
    )
    .unwrap();

    assert!(
        ir.contains(
            "declare i64 @phpc_native_value_to_int_with_reference_slot_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeReferenceHandle, ptr)"
        ),
        "{ir}"
    );
    assert!(
        ir.matches("call i64 @phpc_native_value_to_int_with_reference_slot_with_diagnostic")
            .count()
            >= 5,
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_string_int_operation_with_diagnostic")
            && ir.contains("call %phpc.NativeValueHandle @phpc_native_value_string_search_result_with_diagnostic"),
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeValueHandle @phpc_native_reference_value_clone")
            && !ir.contains("call i1 @phpc_native_reference_set_value")
            && !ir.contains("LLVM string-int builtin lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_extended_reference_string_result_slots_without_value_clone_detour() {
    let ir = emit_ir_source(
        "<?php\n$offset = 1;\n$length = 2;\n$offsetRef =& $offset;\n$lengthRef =& $length;\necho strncmp(\"abcdef\", \"abcxyz\", $lengthRef);\necho substr_count(\"abcabc\", \"a\", $offsetRef, $lengthRef);\necho strncasecmp(\"ABCDEF\", \"abcdxy\", $lengthRef);\necho strpos(\"abcabc\", \"a\", $offsetRef);\n",
    )
    .unwrap();

    assert!(
        ir.contains("call i64 @phpc_native_value_to_int_with_reference_slot_with_diagnostic")
            && ir.contains("call i64 @phpc_native_value_string_int_operation_with_diagnostic")
            && ir.contains("call %phpc.NativeValueHandle @phpc_native_value_string_search_result_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.matches("call i64 @phpc_native_value_to_int_with_reference_slot_with_diagnostic")
            .count()
            >= 5,
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeValueHandle @phpc_native_reference_value_clone")
            && !ir.contains("call i1 @phpc_native_reference_set_value")
            && !ir.contains("LLVM string-result builtin lowering rejects")
            && !ir.contains("LLVM string-int builtin lowering rejects"),
        "{ir}"
    );
}

#[test]
fn generated_ir_rejects_post_alias_direct_root_assignment_without_partial_write_through() {
    let cases = [
        (
            "statement assignment",
            "<?php\n$value = 1;\n$alias =& $value;\n$value = \"text\";\necho gettype($alias);\n",
        ),
        (
            "assignment expression",
            "<?php\n$value = 1;\n$alias =& $value;\necho ($value = \"text\");\necho gettype($alias);\n",
        ),
    ];

    for (label, source) in cases {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen, "{label}");
        assert!(
            error.message.contains(
                "LLVM reference write-through lowering rejects direct root-variable assignment after reference binding"
            ),
            "{label}: {}",
            error.message
        );
    }
}

#[test]
fn generated_ir_string_int_route_reaches_assembly_backend() {
    if !has_llvm_assembly_backend() {
        return;
    }

    let asm = emit_asm_source(STRING_INT_IR_SOURCE).unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn generated_ir_string_predicate_route_reaches_assembly_backend() {
    if !has_llvm_assembly_backend() {
        return;
    }

    let asm = emit_asm_source(STRING_PREDICATE_IR_SOURCE).unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn generated_ir_string_search_route_reaches_assembly_backend() {
    if !has_llvm_assembly_backend() {
        return;
    }

    let asm = emit_asm_source(STRING_SEARCH_IR_SOURCE).unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn generated_ir_string_result_route_reaches_assembly_backend() {
    if !has_llvm_assembly_backend() {
        return;
    }

    let asm = emit_asm_source(STRING_RESULT_IR_SOURCE).unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn generated_ir_routes_string_offset_reads_and_probes_through_value_offset_boundary() {
    let cases = [
        ("literal read", "<?php\necho \"A\\0B\\xff\"[1];\n", 0, false),
        (
            "variable read",
            "<?php\n$payload = \"A\\0B\\xff\";\necho $payload[1];\n",
            0,
            false,
        ),
        (
            "isset probe",
            "<?php\n$payload = \"A\\0B\\xff\";\necho isset($payload[1]);\n",
            1,
            true,
        ),
        (
            "empty probe",
            "<?php\n$payload = \"A\\0B\\xff\";\necho empty($payload[1]);\n",
            2,
            true,
        ),
    ];

    for (label, source, tag, bool_probe) in cases {
        let ir = emit_ir_source(source).unwrap();

        assert!(
            ir.contains("declare %phpc.NativeValueHandle @phpc_native_value_offset_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i8, ptr)"),
            "{label}: {ir}"
        );
        assert!(
            ir.contains(&format!(
                "call %phpc.NativeValueHandle @phpc_native_value_offset_operation_with_diagnostic"
            )),
            "{label}: {ir}"
        );
        assert!(
            ir.contains(&format!("i8 {tag}, ptr")),
            "{label}: expected value-offset operation tag {tag}\n{ir}"
        );
        assert!(
            ir.contains("call void @phpc_native_diagnostic_free"),
            "{label}: {ir}"
        );
        assert!(
            ir.contains("call void @phpc_native_value_free"),
            "{label}: {ir}"
        );
        assert!(
            !ir.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
            "{label}: string-only offset ABI should not be used\n{ir}"
        );
        if bool_probe {
            assert!(
                ir.contains("call i1 @phpc_native_value_bool_with_diagnostic"),
                "{label}: {ir}"
            );
        } else {
            assert!(
                ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
                "{label}: {ir}"
            );
        }
    }
}

#[test]
fn generated_ir_routes_offset_results_through_string_consumers() {
    let ir = emit_ir_source(VALUE_OFFSET_IR_SOURCE).unwrap();

    assert!(
        ir.matches(
            "call %phpc.NativeValueHandle @phpc_native_value_offset_operation_with_diagnostic"
        )
        .count()
            >= 5,
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes"),
        "{ir}"
    );
    assert!(
        ir.contains("call void @phpc_native_string_conversion_result_free"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_string_int_operation_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("call i1 @phpc_native_value_bool_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.matches("call void @phpc_native_value_free").count() >= 10,
        "{ir}"
    );
    assert!(
        !ir.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_native_value_truthiness_through_runtime_abi() {
    let source = "<?php\necho !(\"0\"[0]);\necho ((\"0\"[0]) xor (\"A\"[0]));\n";
    let ir = emit_ir_source(source).unwrap();

    assert!(
        ir.contains("declare i1 @phpc_native_value_is_truthy(%phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_value_is_truthy").count() >= 3,
        "{ir}"
    );
    assert!(
        ir.matches(
            "call %phpc.NativeValueHandle @phpc_native_value_offset_operation_with_diagnostic"
        )
        .count()
            >= 3,
        "{ir}"
    );
    assert!(
        ir.contains(" = xor i1 "),
        "logical XOR and unary not should consume boolean truthiness operands\n{ir}"
    );
    assert!(
        ir.matches("call void @phpc_native_value_free").count() >= 6,
        "{ir}"
    );
    assert!(!ir.contains("logical lowering rejects"), "{ir}");
    assert!(!ir.contains("unary lowering rejects"), "{ir}");
}

#[test]
fn generated_ir_routes_native_value_variable_isset_empty_through_null_and_truthiness_boundaries() {
    let source = concat!(
        "<?php\n",
        "$payload = \"0\";\n",
        "$ref =& $payload;\n",
        "echo isset($ref);\n",
        "echo empty($ref);\n",
        "$payload2 = \"A\\0B\";\n",
        "$ref2 =& $payload2;\n",
        "echo isset($ref2);\n",
        "echo empty($ref2);\n",
    );
    let ir = emit_ir_source(source).unwrap();

    assert!(
        ir.contains("declare i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            && ir.contains(
                "declare i1 @phpc_native_value_type_predicate_with_reference_slot_with_diagnostic"
            ),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            .count()
            >= 2
            && ir
                .matches(
                    "call i1 @phpc_native_value_type_predicate_with_reference_slot_with_diagnostic"
                )
                .count()
                >= 2,
        "{ir}"
    );
    assert!(
        !ir.contains("call i1 @phpc_native_value_is_truthy")
            && !ir.contains("call i1 @phpc_native_value_truthy_with_diagnostic"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_empty_static_values_through_truthiness_boundary() {
    let source = concat!(
        "<?php\n",
        "$zero = \"0\";\n",
        "$payload = \"A\\0B\";\n",
        "$intZero = 0;\n",
        "$intOne = 1;\n",
        "echo empty($zero);\n",
        "echo empty($payload);\n",
        "echo empty($intZero);\n",
        "echo empty($intOne);\n",
    );
    let ir = emit_ir_source(source).unwrap();

    assert!(
        ir.contains("declare i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            && ir.contains("declare %phpc.NativeValueHandle @phpc_native_value_from_scalar"),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            .count()
            >= 4
            && !ir.contains("call i1 @phpc_native_value_is_truthy")
            && !ir.contains("call i1 @phpc_native_value_truthy_with_diagnostic"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_native_value_unary_not_through_truthiness_boundary() {
    let source = concat!(
        "<?php\n",
        "$payload = \"A\\0B|0|\";\n",
        "echo !$payload[0];\n",
        "echo !$payload[1];\n",
        "echo !$payload[4];\n",
        "$refPayload = \"0\";\n",
        "$ref =& $refPayload;\n",
        "echo !$ref;\n",
    );
    let ir = emit_ir_source(source).unwrap();

    assert!(
        ir.contains("declare i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            && ir.contains("call %phpc.NativeConversionResult @phpc_native_offset_read_source"),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            .count()
            >= 4
            && ir.matches(" = xor i1 ").count() >= 4
            && !ir.contains("call i1 @phpc_native_value_is_truthy")
            && !ir.contains("call i1 @phpc_native_value_truthy_with_diagnostic"),
        "{ir}"
    );
}

#[test]
fn generated_ir_routes_reference_truthiness_operands_without_value_clone_detour() {
    let source = concat!(
        "<?php\n",
        "$payload = \"0\";\n",
        "$ref =& $payload;\n",
        "echo !$ref;\n",
        "echo empty($ref);\n",
        "$payload2 = \"A\\0B\";\n",
        "$ref2 =& $payload2;\n",
        "echo !$ref2;\n",
        "echo empty($ref2);\n",
    );
    let ir = emit_ir_source(source).unwrap();

    assert!(
        ir.contains("declare i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            && ir.contains("declare %phpc.NativeReferenceHandle"),
        "{ir}"
    );
    assert!(
        ir.matches("call i1 @phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            .count()
            >= 4
            && !ir.contains("call %phpc.NativeValueHandle @phpc_native_reference_value_clone")
            && !ir.contains("call i1 @phpc_native_value_is_truthy")
            && !ir.contains("call i1 @phpc_native_value_truthy_with_diagnostic"),
        "{ir}"
    );
}

#[test]
fn generated_scalar_offset_reads_feed_warning_continuations_across_consumers() {
    let ir_source = concat!(
        "<?php\n",
        "$scalar = 42;\n",
        "$text = \"A\\0B\";\n",
        "echo $scalar[0];\n",
        "echo \"|\";\n",
        "echo strlen($text[1]);\n",
    );
    let ir = emit_ir_source(ir_source).unwrap();

    let c_source_input = concat!(
        "<?php\n",
        "$scalar = 42;\n",
        "$text = \"A\\0B\";\n",
        "$items = [\"name\" => \"Ada\"];\n",
        "echo $scalar[0];\n",
        "echo \"|\";\n",
        "echo strlen($text[1]);\n",
        "echo \"|\";\n",
        "echo $items[\"name\"];\n",
    );

    assert!(ir.contains("%phpc.NativeConversionSource = type"), "{ir}");
    assert!(
        ir.contains("declare %phpc.NativeConversionResult @phpc_native_offset_read_source"),
        "{ir}"
    );
    assert!(
        ir.matches("call %phpc.NativeConversionResult @phpc_native_offset_read_source")
            .count()
            >= 2,
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic")
            && ir.contains(
                "call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes"
            ),
        "{ir}"
    );

    let program = parse(c_source_input).unwrap();
    let c_source = emit_native_executable_c_source(&program).unwrap();
    assert!(
        c_source.contains("phpc_NativeConversionResult"),
        "{c_source}"
    );
    assert!(
        c_source.matches("phpc_native_offset_read_source").count() >= 3,
        "{c_source}"
    );
    assert!(
        !c_source.contains("= phpc_native_value_offset_operation_with_diagnostic("),
        "{c_source}"
    );
}

#[test]
fn native_executable_scalar_offset_reads_continue_across_conversion_consumers() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "$scalar = 42;\n",
        "$text = \"A\\0B\";\n",
        "$items = [\"name\" => \"Ada\"];\n",
        "echo $scalar[0];\n",
        "echo \"|\";\n",
        "echo strlen($text[1]);\n",
        "echo \"|\";\n",
        "echo $items[\"name\"];\n",
    );
    let (_source_path, exe_path) =
        compile_native_runtime_abi_executable("scalar_offset_source_result", source);
    let output = Command::new(&exe_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {exe_path:?}: {error}"));
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "|1|Ada");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Trying to access array offset on value of type int"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_object_property_offset_reads_use_shared_source_results() {
    let source = concat!(
        "<?php\n",
        "class Box { public $name; }\n",
        "$box = new Box();\n",
        "$box->name = \"Ada\";\n",
        "echo $box->name[1];\n",
        "echo strlen($box->name[2]);\n",
    );
    let program = parse(source).unwrap();
    let c_source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        c_source.contains("phpc_native_object_property_offset_read_source"),
        "{c_source}"
    );
    assert!(
        c_source
            .matches("phpc_native_object_property_offset_read_source")
            .count()
            >= 2,
        "{c_source}"
    );
    assert!(
        c_source.contains("phpc_NativeConversionSource")
            && c_source.contains("phpc_NativeConversionResult"),
        "{c_source}"
    );
}

#[test]
fn generated_ir_value_offset_route_reaches_assembly_backend() {
    if !has_llvm_assembly_backend() {
        return;
    }

    let asm = emit_asm_source(VALUE_OFFSET_IR_SOURCE).unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn generated_ir_blocks_string_int_unsupported_forms_at_shared_boundary() {
    for source in [
        "<?php\nstrcmp('a');\n",
        "<?php\nstrncmp('a', 'b');\n",
        "<?php\nord('a', 'b');\n",
        "<?php\nsubstr_count('abc');\n",
        "<?php\nstrpos('abc');\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen, "{source}");
        assert!(
            error.message.contains(
                "LLVM string-int/search builtin lowering rejects strcasecmp(), strcmp(), strncmp(), strncasecmp(), strpos(), substr_count(), ord(), and crc32() forms outside the reusable native string operation contracts"
            ),
            "{source}: {}",
            error.message
        );
    }
}

#[test]
fn generated_ir_blocks_string_result_unsupported_forms_at_shared_boundary() {
    for source in [
        "<?php\nstrrev();\n",
        "<?php\nstrtolower('a', 'b');\n",
        "<?php\nescapeshellarg('a', 'b');\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen, "{source}");
        assert!(
            error.message.contains(
                "LLVM string-result builtin lowering rejects forms outside the reusable native string-result operation contract"
            ),
            "{source}: {}",
            error.message
        );
    }
}

#[test]
fn generated_ir_blocks_string_predicate_unsupported_forms_at_shared_boundary() {
    for source in [
        "<?php\nstr_contains('abc');\n",
        "<?php\nstr_starts_with('abc', 'a', 'extra');\n",
        "<?php\nstr_ends_with('abc');\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen, "{source}");
        assert!(
            error.message.contains(
                "LLVM string-predicate lowering rejects forms outside the reusable native string predicate contract"
            ),
            "{source}: {}",
            error.message
        );
    }
}

#[test]
fn generated_ir_routes_nested_string_call_results_through_native_value_operands() {
    let ir = emit_ir_source(
        r#"<?php
echo strtoupper(strtolower("MiXeD")), "|";
echo str_contains(strrev("abc"), "b"), "|";
echo strlen(strrev("abc")), "|";
echo strrev(strlen(strrev("abc"))), "|";
echo strcasecmp(strrev("AbC"), strtolower("abc")), "|";
echo strrev(strpos("abc", "b"));
"#,
    )
    .unwrap();

    assert!(
        ir.matches("phpc_native_value_string_result_operation_with_diagnostic")
            .count()
            >= 4,
        "{ir}"
    );
    assert!(
        ir.contains("phpc_native_value_string_predicate_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("phpc_native_value_string_search_result_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("phpc_native_value_string_int_operation_with_diagnostic"),
        "{ir}"
    );
    assert!(ir.contains("phpc_native_value_to_string_bytes"), "{ir}");
    assert!(
        ir.matches("call void @phpc_native_value_free(%phpc.NativeValueHandle")
            .count()
            >= 7,
        "{ir}"
    );
    assert!(
        !ir.contains("function-call lowering rejects function calls"),
        "{ir}"
    );
}

fn has_llvm_assembly_backend() -> bool {
    ["clang", "llc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok())
}

fn has_cc() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}

fn compile_native_runtime_abi_executable(name: &str, source: &str) -> (PathBuf, PathBuf) {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent");
    let base = std::env::temp_dir().join(format!(
        "phpc-native-runtime-abi-{name}-{}",
        std::process::id()
    ));
    let source_path = base.with_extension("php");
    let output_path = base.with_extension("exe");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, source).expect("write native runtime ABI source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native runtime ABI source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native runtime ABI executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    (source_path, output_path)
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
        ir.contains(
            "declare i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
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
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_symbol_table_read_with_diagnostic(%phpc.NativeSymbolTableHandle, ptr, i64, ptr)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)"),
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
        ir.contains("declare %phpc.NativeByteBuffer @phpc_native_request_state_key_result_buffer(%phpc.NativeRequestStateKeyResult)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i8 @phpc_native_request_state_key_result_status(%phpc.NativeRequestStateKeyResult)"),
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
        ir.contains("define i64 @phpc_probe_symbol_table_write_read()"),
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
        ir.contains(
            "call i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle %value, i8 0, ptr null)"
        ),
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
        ir.contains("call i1 @phpc_native_symbol_table_write")
            && ir.contains(
                "call %phpc.NativeValueHandle @phpc_native_symbol_table_read_with_diagnostic"
            )
            && ir.contains("call void @phpc_native_symbol_table_free"),
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
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_symbol_table_read_with_diagnostic(%phpc.NativeSymbolTableHandle, ptr, i32, ptr)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i32, %phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i32 @phpc_probe_symbol_table_write_read()"),
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
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_request_state_superglobal_snapshot_value(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_request_state_rebuild_request_from_order(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_request_state_rebuild_request_from_policy(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle, %phpc.NativeStringHandle)"),
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
        ir.contains(
            "declare i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
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
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_symbol_table_read_with_diagnostic(%phpc.NativeSymbolTableHandle, ptr, i64, ptr)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, i64, %phpc.NativeValueHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_symbol_table_write_read()"),
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
        ir.contains("declare %phpc.NativeValueHandle @phpc_native_request_state_superglobal_snapshot_value(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_request_state_rebuild_request_from_order(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_request_state_rebuild_request_from_policy(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle, %phpc.NativeStringHandle)"),
        "{ir}"
    );
    assert!(
        ir.contains("define i64 @phpc_probe_request_state_empty_missing_value()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeByteBuffer @phpc_native_request_state_key_result_buffer"),
        "{ir}"
    );
    assert!(
        ir.contains("call i8 @phpc_native_request_state_key_result_status"),
        "{ir}"
    );
    assert!(
        !ir.contains("extractvalue %phpc.NativeRequestStateKeyResult"),
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
        ir.contains(
            "declare i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
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
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
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
        ir.contains(
            "declare i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
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
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
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
        ir.contains(
            "declare i64 @phpc_native_value_format_stdout_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)"
        ),
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
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
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
        ir.contains("call i64 @phpc_native_value_format_stdout_with_diagnostic"),
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
    assert!(
        actual.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{actual}"
    );
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
    assert!(
        actual.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{actual}"
    );
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
    assert!(
        actual.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{actual}"
    );
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
    assert!(
        actual.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{actual}"
    );
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
