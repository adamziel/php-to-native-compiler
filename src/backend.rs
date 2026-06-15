use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ast::{AssignmentOp, IncludeKind};
use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{
    ArrayElement as IrArrayElement, ArrayElementValue as IrArrayElementValue, AssignmentTarget,
    BinaryOp, CastKind, CatchClause as IrCatchClause, ClassDecl, ClosureCapture, FunctionDecl,
    FunctionParameter, IncDecOp, IncDecResult, IncDecTarget, IncludeFile, Instruction,
    ListAssignmentElement, ListAssignmentElementTarget, ListAssignmentTarget, MagicConstantKind,
    Module, PropertyVisibility, ReferenceTarget, TypeHint, UnaryOp, ValueExpr,
};

mod runtime;

const PHP_BINARY_BYTE_SENTINEL_BASE: u32 = 0xE000;
const LEGACY_DOLLAR_BRACE_DEPRECATION_MESSAGE: &str =
    "Using ${var} in strings is deprecated, use {$var} instead";

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    let runtime_requirements = module_runtime_requirements(module);
    let legacy_dollar_brace_deprecations = collect_module_legacy_dollar_brace_deprecations(module);
    let magic_visibility_warnings = collect_module_magic_visibility_warnings(module);
    let needs_callable_dispatch = runtime_requirements.internal_function_dispatch
        || runtime_requirements.dynamic_function_dispatch;
    let has_declared_methods = module.classes.iter().any(|class| !class.methods.is_empty());
    let needs_method_dispatch = runtime_requirements.method_dispatch || has_declared_methods;
    let needs_magic_property_read = module
        .classes
        .iter()
        .any(|class| class_magic_isset_method(class, &module.classes).is_some());
    emit_private_property_metadata_prototype(&mut out);
    emit_runtime(&mut out, &runtime_requirements);
    emit_method_visibility_prototypes(&mut out);
    emit_type_hint_runtime_helpers(&mut out);
    emit_include_prototypes(&mut out, &module.includes);
    emit_include_once_state(&mut out, &module.includes);
    if !module.includes.is_empty() {
        emit_include_runtime_helpers(&mut out);
    }
    emit_user_function_prototypes(
        &mut out,
        &module.functions,
        runtime_requirements.internal_function_dispatch,
        needs_callable_dispatch,
        needs_method_dispatch,
    );
    emit_include_helpers(
        &mut out,
        &module.includes,
        &module.functions,
        &module.classes,
    );
    emit_user_functions(
        &mut out,
        &module.classes,
        &module.functions,
        &module.includes,
        &module.source_file,
        &module.source_dir,
    );
    if runtime_requirements.internal_function_dispatch {
        emit_user_function_dispatch(&mut out, &module.functions, &module.classes);
    }
    emit_class_metadata_helpers(&mut out, &module.classes);
    if runtime_requirements.internal_function_dispatch {
        emit_callable_validation_helpers(&mut out);
    }
    if needs_method_dispatch {
        emit_method_dispatch(
            &mut out,
            &module.classes,
            runtime_requirements.closure_invoke_method_dispatch,
        );
    }
    if needs_magic_property_read {
        emit_magic_property_read_dispatch(&mut out, &module.classes);
    }
    if needs_callable_dispatch {
        emit_dynamic_function_dispatch(&mut out);
        emit_callable_dispatch(&mut out, &module.functions, needs_method_dispatch);
    }
    out.push_str("\nint main(void) {\n");
    out.push_str("    PtnRuntime runtime;\n");
    out.push_str("    ptn_runtime_init(&runtime);\n");
    if !module.functions.is_empty() {
        out.push_str("    static int ptn_declared_user_functions[");
        out.push_str(&module.functions.len().to_string());
        out.push_str("] = { ");
        for (index, function) in module.functions.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(if function.initially_declared {
                "1"
            } else {
                "0"
            });
        }
        out.push_str(" };\n");
        out.push_str("    runtime.declared_user_functions = ptn_declared_user_functions;\n");
    }
    if module.strict_types {
        out.push_str("    runtime.strict_types = 1;\n");
    }
    if needs_method_dispatch {
        out.push_str("    runtime.method_dispatch = ptn_call_declared_method;\n");
        out.push_str("    runtime.declared_method_exists = ptn_declared_class_method_exists;\n");
    }
    if needs_magic_property_read {
        out.push_str("    runtime.magic_property_read = ptn_declared_magic_property_read;\n");
    }
    out.push_str("    runtime.class_scope_allows = ptn_declared_class_scope_allows;\n");
    out.push_str("    runtime.declared_class_is_readonly = ptn_declared_class_is_readonly;\n");
    out.push_str("    runtime.source_path = \"");
    out.push_str(&c_string(&module.source_file));
    out.push_str("\";\n");
    let mut values = ValueEmitter::new(
        &module.source_file,
        &module.source_dir,
        &module.functions,
        &module.classes,
        &module.includes,
    );
    emit_legacy_dollar_brace_deprecations(&mut out, &legacy_dollar_brace_deprecations);
    emit_magic_visibility_warnings(&mut out, &magic_visibility_warnings);
    emit_class_constant_initializers(&mut out, &mut values, &module.classes);
    emit_static_property_initializers(&mut out, &mut values, &module.classes);
    for warning in collect_module_control_warnings(module) {
        emit_control_warning(
            &mut out,
            &warning.message,
            &module.source_file,
            warning.line,
        );
    }
    let mut control_targets = Vec::new();
    for instruction in &module.instructions {
        emit_instruction(
            &mut out,
            &mut values,
            instruction,
            &mut control_targets,
            &module.source_file,
            None,
        );
    }
    out.push_str("    ptn_runtime_free(&runtime);\n");
    out.push_str("    return 0;\n}\n");
    out
}

fn emit_include_prototypes(out: &mut String, includes: &[IncludeFile]) {
    for index in 0..includes.len() {
        out.push_str("static PTN_UNUSED PtnValue ");
        out.push_str(&include_c_name(index));
        out.push_str("(PtnRuntime *include_runtime);\n");
    }
}

fn emit_include_once_state(out: &mut String, includes: &[IncludeFile]) {
    if includes.is_empty() {
        return;
    }
    out.push_str("static PTN_UNUSED unsigned char ptn_include_seen[");
    out.push_str(&includes.len().to_string());
    out.push_str("] = {0};\n");
}

fn emit_include_runtime_helpers(out: &mut String) {
    out.push_str("\nstatic PTN_UNUSED int ptn_include_path_is_absolute(PtnStringOperand path) {\n");
    out.push_str("#if defined(_WIN32)\n");
    out.push_str(
        "    return path.len > 0 && (path.data[0] == '/' || path.data[0] == '\\\\' || (path.len >= 3 && isalpha((unsigned char)path.data[0]) && path.data[1] == ':' && (path.data[2] == '/' || path.data[2] == '\\\\')));\n",
    );
    out.push_str("#else\n");
    out.push_str("    return path.len > 0 && path.data[0] == '/';\n");
    out.push_str("#endif\n");
    out.push_str("}\n");
    out.push_str(
        "\nstatic PTN_UNUSED char *ptn_include_resolve_path(const char *source_dir, PtnStringOperand path) {\n",
    );
    out.push_str("    if (memchr(path.data, '\\0', path.len) != NULL) {\n");
    out.push_str("        return NULL;\n");
    out.push_str("    }\n");
    out.push_str("    if (ptn_include_path_is_absolute(path) || source_dir == NULL || source_dir[0] == '\\0') {\n");
    out.push_str("        return ptn_duplicate_string_len(path.data, path.len);\n");
    out.push_str("    }\n");
    out.push_str("    size_t dir_len = strlen(source_dir);\n");
    out.push_str("    int needs_separator = dir_len > 0 && source_dir[dir_len - 1] != '/';\n");
    out.push_str("    size_t len = dir_len + (needs_separator ? 1 : 0) + path.len;\n");
    out.push_str("    char *resolved = malloc(len + 1);\n");
    out.push_str("    if (resolved == NULL) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    memcpy(resolved, source_dir, dir_len);\n");
    out.push_str("    size_t offset = dir_len;\n");
    out.push_str("    if (needs_separator) {\n");
    out.push_str("        resolved[offset++] = '/';\n");
    out.push_str("    }\n");
    out.push_str("    memcpy(resolved + offset, path.data, path.len);\n");
    out.push_str("    resolved[len] = '\\0';\n");
    out.push_str("    return resolved;\n");
    out.push_str("}\n");
    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_compiled_include_path_error(PtnRuntime *runtime, const char *kind, size_t line, int required) {\n",
    );
    out.push_str("    char message[96];\n");
    out.push_str("    int written = snprintf(message, sizeof(message), \"%s(): Filename contains null byte\", kind);\n");
    out.push_str("    if (written < 0 || (size_t)written >= sizeof(message)) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    ptn_emit_warning(&runtime->diagnostics, message, line);\n");
    out.push_str("    if (required) {\n");
    out.push_str("        ptn_emit_type_error(&runtime->diagnostics, \"Failed opening required compiled include\");\n");
    out.push_str("        exit(255);\n");
    out.push_str("    }\n");
    out.push_str("    return ptn_bool(0);\n");
    out.push_str("}\n");
    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_compiled_include_failure(PtnRuntime *runtime, const char *kind, const char *path, size_t line, int required) {\n",
    );
    out.push_str("    int needed = snprintf(NULL, 0, \"%s(%s): compiled include target is not available\", kind, path);\n");
    out.push_str("    if (needed < 0) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    char *message = malloc((size_t)needed + 1);\n");
    out.push_str("    if (message == NULL) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    int written = snprintf(message, (size_t)needed + 1, \"%s(%s): compiled include target is not available\", kind, path);\n");
    out.push_str("    if (written < 0 || written != needed) {\n");
    out.push_str("        free(message);\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    ptn_emit_warning(&runtime->diagnostics, message, line);\n");
    out.push_str("    free(message);\n");
    out.push_str("    if (required) {\n");
    out.push_str("        ptn_emit_type_error(&runtime->diagnostics, \"Failed opening required compiled include\");\n");
    out.push_str("        exit(255);\n");
    out.push_str("    }\n");
    out.push_str("    return ptn_bool(0);\n");
    out.push_str("}\n");
}

#[derive(Default)]
struct RuntimeRequirements {
    internal_function_dispatch: bool,
    dynamic_function_dispatch: bool,
    method_dispatch: bool,
    closure_invoke_method_dispatch: bool,
    direct_internal_helpers: bool,
}

fn emit_runtime(out: &mut String, requirements: &RuntimeRequirements) {
    if requirements.internal_function_dispatch {
        out.push_str("#define PTN_HAS_INTERNAL_FUNCTION_DISPATCH 1\n");
    }
    let runtime_c = runtime::runtime_c();
    let direct_helpers = runtime_chunk_range(
        runtime_c,
        runtime::DIRECT_INTERNAL_HELPERS_START,
        runtime::DIRECT_INTERNAL_HELPERS_END,
    );
    let internal_functions = runtime_chunk_range(
        runtime_c,
        runtime::INTERNAL_FUNCTIONS_START,
        runtime::INTERNAL_FUNCTIONS_END,
    );
    assert!(
        direct_helpers.after_end <= internal_functions.start,
        "runtime direct-helper chunk should precede internal-function chunk"
    );

    out.push_str(&runtime_c[..direct_helpers.start]);
    if requirements.direct_internal_helpers || requirements.internal_function_dispatch {
        out.push_str(&runtime_c[direct_helpers.after_start..direct_helpers.end]);
    }
    out.push_str(&runtime_c[direct_helpers.after_end..internal_functions.start]);
    if requirements.internal_function_dispatch {
        out.push_str(&runtime_c[internal_functions.after_start..internal_functions.end]);
    }
    out.push_str(&runtime_c[internal_functions.after_end..]);
}

fn emit_type_hint_runtime_helpers(out: &mut String) {
    out.push_str(
        "\nstatic int ptn_declared_class_is_same_or_descendant(const char *class_name, const char *ancestor_name);\n",
    );
    out.push_str(
        "static int ptn_declared_class_implements_interface(const char *class_name, const char *interface_name);\n",
    );
    out.push_str(
        "\nstatic PTN_UNUSED const char *ptn_user_type_hint_given_name(PtnValue value) {\n",
    );
    out.push_str("    value = ptn_value_deref(value);\n");
    out.push_str("    switch (value.type) {\n");
    out.push_str("        case PTN_OBJECT:\n");
    out.push_str("            return value.as.object->class_name;\n");
    out.push_str("        case PTN_EXCEPTION:\n");
    out.push_str("            return value.as.exception->class_name;\n");
    out.push_str("        case PTN_CLOSURE:\n");
    out.push_str("            return \"Closure\";\n");
    out.push_str("        case PTN_NULL:\n");
    out.push_str("        case PTN_BOOL:\n");
    out.push_str("        case PTN_INT:\n");
    out.push_str("        case PTN_FLOAT:\n");
    out.push_str("        case PTN_STRING:\n");
    out.push_str("        case PTN_RESOURCE:\n");
    out.push_str("        case PTN_ARRAY:\n");
    out.push_str("        case PTN_REFERENCE:\n");
    out.push_str("            return ptn_offset_container_type_name(value);\n");
    out.push_str("    }\n");
    out.push_str("    return ptn_offset_container_type_name(value);\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_value_satisfies_class_type_hint(PtnValue value, const char *expected_class_name) {\n",
    );
    out.push_str("    value = ptn_value_deref(value);\n");
    out.push_str("    if (value.type == PTN_OBJECT) {\n");
    out.push_str("        return ptn_declared_class_is_same_or_descendant(value.as.object->class_name, expected_class_name) ||\n");
    out.push_str("            ptn_declared_class_implements_interface(value.as.object->class_name, expected_class_name);\n");
    out.push_str("    }\n");
    out.push_str("    if (value.type == PTN_EXCEPTION) {\n");
    out.push_str("        return ptn_exception_type_matches_name(value.as.exception->class_name, expected_class_name);\n");
    out.push_str("    }\n");
    out.push_str("    if (value.type == PTN_CLOSURE) {\n");
    out.push_str("        return ptn_ascii_case_equal(expected_class_name, \"Closure\");\n");
    out.push_str("    }\n");
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED void ptn_throw_user_parameter_class_type_error(PtnRuntime *runtime, const char *function_name, size_t position, const char *parameter_name, const char *expected_class_name, PtnValue value, size_t line) {\n",
    );
    out.push_str("    const char *given = ptn_user_type_hint_given_name(value);\n");
    out.push_str(
        "    const char *path = runtime->source_path != NULL ? runtime->source_path : \"ptn\";\n",
    );
    out.push_str("    int needed;\n");
    out.push_str("    if (line != 0) {\n");
    out.push_str("        needed = snprintf(NULL, 0, \"%s(): Argument #%zu ($%s) must be of type %s, %s given, called in %s on line %zu\", function_name, position, parameter_name, expected_class_name, given, path, line);\n");
    out.push_str("    } else {\n");
    out.push_str("        needed = snprintf(NULL, 0, \"%s(): Argument #%zu ($%s) must be of type %s, %s given\", function_name, position, parameter_name, expected_class_name, given);\n");
    out.push_str("    }\n");
    out.push_str("    if (needed < 0) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    char *message = malloc((size_t)needed + 1);\n");
    out.push_str("    if (message == NULL) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    if (line != 0) {\n");
    out.push_str("        snprintf(message, (size_t)needed + 1, \"%s(): Argument #%zu ($%s) must be of type %s, %s given, called in %s on line %zu\", function_name, position, parameter_name, expected_class_name, given, path, line);\n");
    out.push_str("    } else {\n");
    out.push_str("        snprintf(message, (size_t)needed + 1, \"%s(): Argument #%zu ($%s) must be of type %s, %s given\", function_name, position, parameter_name, expected_class_name, given);\n");
    out.push_str("    }\n");
    out.push_str("    ptn_throw_exception_owned_message(runtime, \"TypeError\", message);\n");
    out.push_str("}\n");
}

struct RuntimeChunkRange {
    start: usize,
    after_start: usize,
    end: usize,
    after_end: usize,
}

fn runtime_chunk_range(runtime_c: &str, start_marker: &str, end_marker: &str) -> RuntimeChunkRange {
    let start = runtime_c
        .find(start_marker)
        .expect("runtime start marker should exist");
    let after_start = start + start_marker.len();
    let relative_end = runtime_c[after_start..]
        .find(end_marker)
        .expect("runtime end marker should exist");
    let end = after_start + relative_end;
    let after_end = end + end_marker.len();
    RuntimeChunkRange {
        start,
        after_start,
        end,
        after_end,
    }
}

fn emit_user_function_prototypes(
    out: &mut String,
    functions: &[FunctionDecl],
    needs_function_dispatch: bool,
    needs_dynamic_function_dispatch: bool,
    needs_method_dispatch: bool,
) {
    if needs_function_dispatch {
        out.push_str(
            "\nstatic PTN_UNUSED PtnValue ptn_call_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line);\n",
        );
        out.push_str(
            "static PTN_UNUSED PtnValue ptn_call_callable(PtnRuntime *runtime, PtnValue callable, size_t argc, const PtnValue *args, size_t line);\n",
        );
    }
    if needs_dynamic_function_dispatch {
        out.push_str("static PTN_UNUSED char *ptn_dynamic_function_name(PtnValue callable);\n");
        out.push_str(
            "static PTN_UNUSED PtnValue ptn_call_dynamic_function_name(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line);\n",
        );
    }
    if needs_method_dispatch {
        out.push_str(
            "static PTN_UNUSED PtnValue ptn_call_declared_method(PtnRuntime *runtime, PtnValue receiver, const char *method_name, size_t argc, const PtnValue *args, size_t line);\n",
        );
        out.push_str(
            "static PTN_UNUSED int ptn_call_declared_method_in_scope(PtnRuntime *runtime, PtnValue receiver, const char *target_class_name, const char *method_name, const char *called_class_name, size_t argc, const PtnValue *args, size_t line, PtnValue *result_out);\n",
        );
        out.push_str(
            "static PTN_UNUSED void ptn_throw_declared_method_visibility_error(PtnRuntime *runtime, const char *visibility_name, const char *declaring_class, const char *method_name, size_t line);\n",
        );
    }
    for (index, _) in functions.iter().enumerate() {
        out.push_str("static PTN_UNUSED PtnValue ");
        out.push_str(&user_function_c_name(index));
        out.push_str(
            "(PtnRuntime *caller_runtime, PtnValue receiver, size_t argc, const PtnValue *args, size_t line);\n",
        );
    }
}

fn emit_user_functions(
    out: &mut String,
    classes: &[ClassDecl],
    functions: &[FunctionDecl],
    includes: &[IncludeFile],
    source_file: &str,
    source_dir: &str,
) {
    for (index, function) in functions.iter().enumerate() {
        let required_parameter_count = function_required_parameter_count(function);
        let call_frame_parameter_count = function_call_frame_parameter_count(function);
        let arity_error_is_exact = required_parameter_count == function.parameters.len()
            && !function
                .parameters
                .iter()
                .any(|parameter| parameter.is_variadic);
        let c_name = user_function_c_name(index);
        out.push_str("\nstatic PTN_UNUSED PtnValue ");
        out.push_str(&c_name);
        out.push_str(
            "(PtnRuntime *caller_runtime, PtnValue receiver, size_t argc, const PtnValue *args, size_t line) {\n",
        );
        if (function.class_name.is_none() && !function.is_anonymous) || function.is_static {
            out.push_str("    (void)receiver;\n");
        }
        out.push_str("    (void)line;\n");
        if required_parameter_count > 0 {
            out.push_str("    if (argc < ");
            out.push_str(&required_parameter_count.to_string());
            out.push_str(") {\n");
            out.push_str("        if (caller_runtime->throw_argument_count_errors) {\n");
            out.push_str("            ptn_throw_user_argument_count_error(caller_runtime, \"");
            out.push_str(&c_string(&function.display_name));
            out.push_str("\", ");
            out.push_str(&required_parameter_count.to_string());
            out.push_str(", argc, ");
            out.push_str(if arity_error_is_exact { "1" } else { "0" });
            out.push_str(");\n");
            out.push_str("            return ptn_null();\n");
            out.push_str("        }\n");
            out.push_str("        ptn_emit_argument_count_error(&caller_runtime->diagnostics, \"");
            out.push_str(&c_string(&function.display_name));
            out.push_str("\", ");
            out.push_str(&required_parameter_count.to_string());
            out.push_str(", argc);\n");
            out.push_str("        exit(255);\n");
            out.push_str("    }\n");
        }
        out.push_str("    PtnRuntime runtime;\n");
        out.push_str("    ptn_runtime_init_function_frame(&runtime, caller_runtime);\n");
        out.push_str("    runtime.current_function_name = \"");
        out.push_str(&c_string(&function.display_name));
        out.push_str("\";\n");
        out.push_str("    runtime.current_class_name = ");
        out.push_str(&c_optional_string(function.class_name.as_deref()));
        out.push_str(";\n");
        out.push_str("    runtime.current_called_class_name = caller_runtime->called_class_name_override != NULL ? caller_runtime->called_class_name_override : ");
        out.push_str(&c_optional_string(function.class_name.as_deref()));
        out.push_str(";\n");
        out.push_str("    runtime.call_site_line = line;\n");
        if call_frame_parameter_count == 0 {
            out.push_str("    ptn_runtime_set_call_frame(&runtime, argc, args, 0, NULL);\n");
        } else {
            out.push_str("    static const char *ptn_parameter_names[] = { ");
            for (parameter_index, parameter) in function
                .parameters
                .iter()
                .take(call_frame_parameter_count)
                .enumerate()
            {
                if parameter_index > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                out.push_str(&c_string(&parameter.name));
                out.push('"');
            }
            out.push_str(" };\n");
            out.push_str("    ptn_runtime_set_call_frame(&runtime, argc, args, ");
            out.push_str(&call_frame_parameter_count.to_string());
            out.push_str(", ptn_parameter_names);\n");
        }
        if function.method_name.is_some() && !function.is_static {
            out.push_str("    runtime.has_current_receiver = 1;\n");
            out.push_str("    runtime.current_receiver = receiver;\n");
            out.push_str("    ptn_runtime_write_variable(&runtime, \"this\", receiver);\n");
        }
        if function.is_anonymous {
            out.push_str("    ptn_runtime_import_closure_captures(&runtime, receiver);\n");
        }
        out.push_str("    PtnValue ptn_return_value = ptn_null();\n");
        let mut values = ValueEmitter::new_for_function(
            source_file,
            source_dir,
            functions,
            classes,
            includes,
            function,
        );
        for (parameter_index, parameter) in function.parameters.iter().enumerate() {
            if parameter.is_variadic {
                emit_variadic_parameter_binding(out, function, parameter_index, parameter);
                continue;
            }
            let (parameter_source, default_guard) =
                if let Some(default_value) = &parameter.default_value {
                    let guard_name = format!("ptn_parameter_{parameter_index}_uses_default");
                    let value_name = format!("ptn_parameter_{parameter_index}_value");
                    out.push_str("    int ");
                    out.push_str(&guard_name);
                    out.push_str(" = argc <= ");
                    out.push_str(&parameter_index.to_string());
                    out.push_str(";\n");
                    out.push_str("    PtnValue ");
                    out.push_str(&value_name);
                    out.push_str(";\n");
                    out.push_str("    if (");
                    out.push_str(&guard_name);
                    out.push_str(") {\n");
                    let default_temp = values.emit_materialized_value(out, default_value);
                    out.push_str("        ");
                    out.push_str(&value_name);
                    out.push_str(" = ");
                    out.push_str(&default_temp);
                    out.push_str(";\n");
                    out.push_str("    } else {\n");
                    out.push_str("        ");
                    out.push_str(&value_name);
                    out.push_str(" = args[");
                    out.push_str(&parameter_index.to_string());
                    out.push_str("];\n");
                    out.push_str("    }\n");
                    (value_name, Some(guard_name))
                } else {
                    (format!("args[{parameter_index}]"), None)
                };
            if parameter.by_ref {
                let check_indent = if let Some(default_guard) = &default_guard {
                    out.push_str("    if (!");
                    out.push_str(default_guard);
                    out.push_str(") {\n");
                    "        "
                } else {
                    "    "
                };
                out.push_str(check_indent);
                out.push_str("if (");
                out.push_str(&parameter_source);
                out.push_str(".type != PTN_REFERENCE) {\n");
                out.push_str(check_indent);
                out.push_str("    if (caller_runtime->warn_by_ref_argument_mismatch) {\n");
                out.push_str(check_indent);
                out.push_str("        ptn_emit_by_reference_argument_warning(caller_runtime, ptn_by_reference_argument_function_name(caller_runtime, \"");
                out.push_str(&c_string(&function.display_name));
                out.push_str("\"), ");
                out.push_str(&(parameter_index + 1).to_string());
                out.push_str(", \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", line);\n");
                out.push_str(check_indent);
                out.push_str("    } else {\n");
                out.push_str(check_indent);
                out.push_str("        ptn_abort_by_reference_argument_error(ptn_by_reference_argument_function_name(caller_runtime, \"");
                out.push_str(&c_string(&function.display_name));
                out.push_str("\"), ");
                out.push_str(&(parameter_index + 1).to_string());
                out.push_str(", \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\");\n");
                out.push_str(check_indent);
                out.push_str("    }\n");
                out.push_str(check_indent);
                out.push_str("}\n");
                if default_guard.is_some() {
                    out.push_str("    }\n");
                }
            }
            if matches!(parameter.type_hint.as_ref(), Some(TypeHint::Null)) {
                out.push_str("    if (ptn_value_deref(");
                out.push_str(&parameter_source);
                out.push_str(").type != PTN_NULL) {\n");
                out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
                out.push_str(&c_string(&function.display_name));
                out.push_str("() argument $");
                out.push_str(&c_string(&parameter.name));
                out.push_str(" must be of type null\");\n");
                out.push_str("        ptn_runtime_free(&runtime);\n");
                out.push_str("        exit(255);\n");
                out.push_str("    }\n");
            }
            if matches!(parameter.type_hint.as_ref(), Some(TypeHint::Array)) {
                out.push_str("    if (ptn_value_deref(");
                out.push_str(&parameter_source);
                out.push_str(").type != PTN_ARRAY) {\n");
                out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
                out.push_str(&c_string(&function.display_name));
                out.push_str("() argument $");
                out.push_str(&c_string(&parameter.name));
                out.push_str(" must be of type array\");\n");
                out.push_str("        ptn_runtime_free(&runtime);\n");
                out.push_str("        exit(255);\n");
                out.push_str("    }\n");
            }
            if let Some(TypeHint::Class(class_name)) = parameter.type_hint.as_ref() {
                out.push_str("    if (!ptn_value_satisfies_class_type_hint(");
                out.push_str(&parameter_source);
                out.push_str(", \"");
                out.push_str(&c_string(class_name));
                out.push_str("\")) {\n");
                out.push_str("        ptn_runtime_free(&runtime);\n");
                out.push_str(
                    "        ptn_throw_user_parameter_class_type_error(caller_runtime, \"",
                );
                out.push_str(&c_string(&function.display_name));
                out.push_str("\", ");
                out.push_str(&(parameter_index + 1).to_string());
                out.push_str(", \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", \"");
                out.push_str(&c_string(class_name));
                out.push_str("\", ");
                out.push_str(&parameter_source);
                out.push_str(", line);\n");
                out.push_str("        return ptn_null();\n");
                out.push_str("    }\n");
            }
            let parameter_cast_temp = if let Some(cast_helper) =
                type_hint_scalar_cast_helper(parameter.type_hint.as_ref())
            {
                let temp = format!("ptn_parameter_{}", parameter_index);
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ");
                out.push_str(cast_helper);
                out.push('(');
                out.push_str(&parameter_source);
                out.push_str(");\n");
                Some(temp)
            } else {
                None
            };
            let parameter_value = parameter_cast_temp
                .as_deref()
                .map(str::to_string)
                .unwrap_or_else(|| parameter_source.clone());
            if parameter.by_ref {
                if let Some(default_guard) = &default_guard {
                    out.push_str("    if (!");
                    out.push_str(default_guard);
                    out.push_str(") {\n");
                }
                out.push_str("    if (");
                out.push_str(&parameter_source);
                out.push_str(".type == PTN_REFERENCE) {\n");
                if let Some(temp) = &parameter_cast_temp {
                    out.push_str("        ptn_reference_assign(");
                    out.push_str(&parameter_source);
                    out.push_str(".as.reference, ");
                    out.push_str(temp);
                    out.push_str(");\n");
                }
                out.push_str("        ptn_runtime_bind_variable_reference(&runtime, \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", ");
                out.push_str(&parameter_source);
                out.push_str(");\n");
                out.push_str("    } else {\n");
                out.push_str("        ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", ");
                out.push_str(&parameter_value);
                out.push_str(");\n");
                out.push_str("    }\n");
                if default_guard.is_some() {
                    out.push_str("    } else {\n");
                    out.push_str("        ptn_runtime_write_variable(&runtime, \"");
                    out.push_str(&c_string(&parameter.name));
                    out.push_str("\", ");
                    out.push_str(&parameter_value);
                    out.push_str(");\n");
                    out.push_str("    }\n");
                }
            } else {
                out.push_str("    ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", ");
                out.push_str(&parameter_value);
                out.push_str(");\n");
            }
            if let Some(temp) = &parameter_cast_temp {
                emit_value_cleanup(out, "    ", temp);
            }
            if let Some(default_guard) = &default_guard {
                out.push_str("    if (");
                out.push_str(default_guard);
                out.push_str(") {\n");
                emit_value_cleanup(out, "        ", &parameter_source);
                out.push_str("    }\n");
            }
        }
        let mut break_targets = Vec::new();
        let return_label = values.next_label("ptn_function_return");
        for instruction in &function.body {
            emit_instruction(
                out,
                &mut values,
                instruction,
                &mut break_targets,
                source_file,
                Some(&return_label),
            );
        }
        emit_label_reference(out, &return_label);
        out.push_str("    ");
        out.push_str(&return_label);
        out.push_str(":\n");
        if let Some(return_type) = function.return_type.as_ref() {
            emit_return_type_boundary(
                out,
                return_type,
                &function.display_name,
                function.return_by_ref,
            );
        }
        out.push_str("    caller_runtime->diagnostics.error_reporting = runtime.diagnostics.error_reporting;\n");
        out.push_str("    ptn_runtime_free(&runtime);\n");
        out.push_str("    return ptn_return_value;\n");
        out.push_str("}\n");
    }
}

fn function_required_parameter_count(function: &FunctionDecl) -> usize {
    function
        .parameters
        .iter()
        .take_while(|parameter| !parameter.is_variadic && parameter.default_value.is_none())
        .count()
}

fn function_call_frame_parameter_count(function: &FunctionDecl) -> usize {
    function
        .parameters
        .iter()
        .position(|parameter| parameter.is_variadic)
        .unwrap_or(function.parameters.len())
}

fn emit_function_metadata_parameter_names(
    out: &mut String,
    indent: &str,
    name: &str,
    parameters: &[FunctionParameter],
) -> String {
    if parameters.is_empty() {
        return "NULL".to_string();
    }
    out.push_str(indent);
    out.push_str("static const char *const ");
    out.push_str(name);
    out.push_str("[] = { ");
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(&c_string(&parameter.name));
        out.push('"');
    }
    out.push_str(" };\n");
    name.to_string()
}

fn emit_variadic_parameter_binding(
    out: &mut String,
    function: &FunctionDecl,
    parameter_index: usize,
    parameter: &FunctionParameter,
) {
    let array_temp = format!("ptn_variadic_{}", parameter_index);
    let index_temp = format!("ptn_variadic_i_{}", parameter_index);
    let offset_temp = format!("ptn_variadic_offset_{}", parameter_index);

    out.push_str("    PtnValue ");
    out.push_str(&array_temp);
    out.push_str(" = ptn_array_from_literal_entries(0, NULL);\n");
    out.push_str("    for (size_t ");
    out.push_str(&index_temp);
    out.push_str(" = ");
    out.push_str(&parameter_index.to_string());
    out.push_str("; ");
    out.push_str(&index_temp);
    out.push_str(" < argc; ");
    out.push_str(&index_temp);
    out.push_str("++) {\n");
    out.push_str("        size_t ");
    out.push_str(&offset_temp);
    out.push_str(" = ");
    out.push_str(&index_temp);
    out.push_str(" - ");
    out.push_str(&parameter_index.to_string());
    out.push_str(";\n");
    out.push_str("        if (");
    out.push_str(&offset_temp);
    out.push_str(" > (size_t)INT64_MAX) {\n");
    out.push_str("            ptn_abort_out_of_memory();\n");
    out.push_str("        }\n");

    if parameter.by_ref {
        out.push_str("        if (args[");
        out.push_str(&index_temp);
        out.push_str("].type != PTN_REFERENCE) {\n");
        out.push_str("            if (caller_runtime->warn_by_ref_argument_mismatch) {\n");
        out.push_str("                ptn_emit_by_reference_argument_warning(caller_runtime, ptn_by_reference_argument_function_name(caller_runtime, \"");
        out.push_str(&c_string(&function.display_name));
        out.push_str("\"), ");
        out.push_str(&index_temp);
        out.push_str(" + 1, \"");
        out.push_str(&c_string(&parameter.name));
        out.push_str("\", line);\n");
        out.push_str("            } else {\n");
        out.push_str("            ptn_abort_by_reference_argument_error(ptn_by_reference_argument_function_name(caller_runtime, \"");
        out.push_str(&c_string(&function.display_name));
        out.push_str("\"), ");
        out.push_str(&index_temp);
        out.push_str(" + 1, \"");
        out.push_str(&c_string(&parameter.name));
        out.push_str("\");\n");
        out.push_str("            }\n");
        out.push_str("        }\n");
    }

    if matches!(parameter.type_hint.as_ref(), Some(TypeHint::Null)) {
        out.push_str("        if (ptn_value_deref(args[");
        out.push_str(&index_temp);
        out.push_str("]).type != PTN_NULL) {\n");
        out.push_str("            ptn_emit_type_error(&caller_runtime->diagnostics, \"");
        out.push_str(&c_string(&function.display_name));
        out.push_str("() argument $");
        out.push_str(&c_string(&parameter.name));
        out.push_str(" must be of type null\");\n");
        out.push_str("            ptn_value_drop(&");
        out.push_str(&array_temp);
        out.push_str(");\n");
        out.push_str("            ptn_runtime_free(&runtime);\n");
        out.push_str("            exit(255);\n");
        out.push_str("        }\n");
    }
    if matches!(parameter.type_hint.as_ref(), Some(TypeHint::Array)) {
        out.push_str("        if (ptn_value_deref(args[");
        out.push_str(&index_temp);
        out.push_str("]).type != PTN_ARRAY) {\n");
        out.push_str("            ptn_emit_type_error(&caller_runtime->diagnostics, \"");
        out.push_str(&c_string(&function.display_name));
        out.push_str("() argument $");
        out.push_str(&c_string(&parameter.name));
        out.push_str(" must be of type array\");\n");
        out.push_str("            ptn_value_drop(&");
        out.push_str(&array_temp);
        out.push_str(");\n");
        out.push_str("            ptn_runtime_free(&runtime);\n");
        out.push_str("            exit(255);\n");
        out.push_str("        }\n");
    }
    if let Some(TypeHint::Class(class_name)) = parameter.type_hint.as_ref() {
        out.push_str("        if (!ptn_value_satisfies_class_type_hint(args[");
        out.push_str(&index_temp);
        out.push_str("], \"");
        out.push_str(&c_string(class_name));
        out.push_str("\")) {\n");
        out.push_str("            ptn_value_drop(&");
        out.push_str(&array_temp);
        out.push_str(");\n");
        out.push_str("            ptn_runtime_free(&runtime);\n");
        out.push_str("            ptn_throw_user_parameter_class_type_error(caller_runtime, \"");
        out.push_str(&c_string(&function.name));
        out.push_str("\", ");
        out.push_str(&index_temp);
        out.push_str(" + 1, \"");
        out.push_str(&c_string(&parameter.name));
        out.push_str("\", \"");
        out.push_str(&c_string(class_name));
        out.push_str("\", args[");
        out.push_str(&index_temp);
        out.push_str("], line);\n");
        out.push_str("            return ptn_null();\n");
        out.push_str("        }\n");
    }

    let value_expr = if let Some(cast_helper) =
        type_hint_scalar_cast_helper(parameter.type_hint.as_ref())
    {
        let value_temp = format!("ptn_variadic_value_{}", parameter_index);
        out.push_str("        PtnValue ");
        out.push_str(&value_temp);
        out.push_str(" = ");
        out.push_str(cast_helper);
        out.push_str("(args[");
        out.push_str(&index_temp);
        out.push_str("]);\n");
        if parameter.by_ref {
            out.push_str("        if (args[");
            out.push_str(&index_temp);
            out.push_str("].type == PTN_REFERENCE) {\n");
            out.push_str("            ptn_reference_assign(args[");
            out.push_str(&index_temp);
            out.push_str("].as.reference, ");
            out.push_str(&value_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "            ", &value_temp);
            out.push_str("        }\n");
            format!(
                "args[{index_temp}].type == PTN_REFERENCE ? ptn_value_clone(args[{index_temp}]) : {value_temp}"
            )
        } else {
            value_temp
        }
    } else if parameter.by_ref {
        format!("ptn_value_clone(args[{index_temp}])")
    } else {
        format!("ptn_value_clone_deref(args[{index_temp}])")
    };

    out.push_str("        ptn_array_set_entry(");
    out.push_str(&array_temp);
    out.push_str(".as.array, ptn_array_int_key((int64_t)");
    out.push_str(&offset_temp);
    out.push_str("), ");
    out.push_str(&value_expr);
    out.push_str(");\n");
    out.push_str("    }\n");
    out.push_str("    ptn_runtime_write_variable(&runtime, \"");
    out.push_str(&c_string(&parameter.name));
    out.push_str("\", ");
    out.push_str(&array_temp);
    out.push_str(");\n");
    emit_value_cleanup(out, "    ", &array_temp);
}

fn by_ref_parameter_for_argument(
    parameters: &[FunctionParameter],
    argument_index: usize,
) -> Option<&FunctionParameter> {
    if let Some(parameter) = parameters.get(argument_index) {
        if parameter.by_ref {
            return Some(parameter);
        }
    }

    parameters
        .iter()
        .enumerate()
        .find(|(parameter_index, parameter)| {
            parameter.is_variadic && parameter.by_ref && argument_index >= *parameter_index
        })
        .map(|(_, parameter)| parameter)
}

fn internal_by_ref_parameter_name(name: &str, argument_index: usize) -> Option<&'static str> {
    if name.eq_ignore_ascii_case("array_pop") && argument_index == 0 {
        return Some("array");
    }
    if name.eq_ignore_ascii_case("array_shift") && argument_index == 0 {
        return Some("array");
    }
    if name.eq_ignore_ascii_case("array_splice") && argument_index == 0 {
        return Some("array");
    }
    if name.eq_ignore_ascii_case("array_walk_recursive") && argument_index == 0 {
        return Some("array");
    }
    if name.eq_ignore_ascii_case("preg_match") && argument_index == 2 {
        return Some("matches");
    }
    if name.eq_ignore_ascii_case("str_replace") && argument_index == 3 {
        return Some("count");
    }
    if name.eq_ignore_ascii_case("is_callable") && argument_index == 2 {
        return Some("callable_name");
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "end" | "next" | "prev" | "reset"
    ) && argument_index == 0
    {
        return Some("array");
    }
    None
}

fn internal_by_ref_temporary_argument_allowed(name: &str, argument_index: usize) -> bool {
    argument_index == 0
        && matches!(
            name.to_ascii_lowercase().as_str(),
            "array_pop" | "array_shift" | "end" | "next" | "prev" | "reset"
        )
}

fn emit_include_helpers(
    out: &mut String,
    includes: &[IncludeFile],
    functions: &[FunctionDecl],
    classes: &[ClassDecl],
) {
    for (index, include) in includes.iter().enumerate() {
        out.push_str("\nstatic PTN_UNUSED PtnValue ");
        out.push_str(&include_c_name(index));
        out.push_str("(PtnRuntime *include_runtime) {\n");
        out.push_str("    (void)include_runtime;\n");
        out.push_str("#define runtime (*include_runtime)\n");
        out.push_str("    PtnValue ptn_return_value = ptn_int(1);\n");
        let legacy_dollar_brace_deprecations =
            collect_include_legacy_dollar_brace_deprecations(include);
        emit_legacy_dollar_brace_deprecations(out, &legacy_dollar_brace_deprecations);
        let mut values = ValueEmitter::new(
            &include.source_file,
            &include.source_dir,
            functions,
            classes,
            includes,
        );
        let mut control_targets = Vec::new();
        let return_label = values.next_label("ptn_include_return");
        for instruction in &include.instructions {
            emit_instruction(
                out,
                &mut values,
                instruction,
                &mut control_targets,
                &include.source_file,
                Some(&return_label),
            );
        }
        emit_label_reference(out, &return_label);
        out.push_str("    ");
        out.push_str(&return_label);
        out.push_str(":\n");
        out.push_str("#undef runtime\n");
        out.push_str("    return ptn_return_value;\n");
        out.push_str("}\n");
    }
}

fn emit_static_property_initializers(
    out: &mut String,
    values: &mut ValueEmitter,
    classes: &[ClassDecl],
) {
    for class in classes {
        let previous_class_name = values.current_class_name.replace(class.name.clone());
        for property in &class.static_properties {
            let value_temp = match &property.value {
                Some(value) => values.emit_materialized_value(out, value),
                None => {
                    let temp = values.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&temp);
                    out.push_str(" = ptn_null();\n");
                    temp
                }
            };
            out.push_str("    ptn_runtime_define_static_property(&runtime, \"");
            out.push_str(&c_string(&class.name));
            out.push_str("\", \"");
            out.push_str(&c_string(&property.name));
            out.push_str("\", ");
            out.push_str(c_property_visibility(property.visibility));
            out.push_str(", ");
            out.push_str(c_property_visibility(property.set_visibility));
            out.push_str(", ");
            out.push_str(&value_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &value_temp);
        }
        values.current_class_name = previous_class_name;
    }
}

fn emit_class_constant_initializers(
    out: &mut String,
    values: &mut ValueEmitter,
    classes: &[ClassDecl],
) {
    for class in classes {
        let previous_class_name = values.current_class_name.replace(class.name.clone());
        for constant in &class.constants {
            let value_temp = values.emit_const_materialized_value(out, &constant.value);
            out.push_str("    ptn_runtime_define_class_constant(&runtime, \"");
            out.push_str(&c_string(&class.name));
            out.push_str("\", \"");
            out.push_str(&c_string(&constant.name));
            out.push_str("\", ");
            out.push_str(&value_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &value_temp);
        }
        values.current_class_name = previous_class_name;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LegacyDollarBraceDeprecation {
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MagicVisibilityWarning {
    class_name: String,
    method_name: String,
    line: usize,
}

fn emit_legacy_dollar_brace_deprecations(
    out: &mut String,
    deprecations: &[LegacyDollarBraceDeprecation],
) {
    for deprecation in deprecations {
        out.push_str("    ptn_emit_deprecation(&runtime.diagnostics, \"");
        out.push_str(&c_string(LEGACY_DOLLAR_BRACE_DEPRECATION_MESSAGE));
        out.push_str("\", ");
        out.push_str(&deprecation.line.to_string());
        out.push_str(");\n");
    }
}

fn emit_magic_visibility_warnings(out: &mut String, warnings: &[MagicVisibilityWarning]) {
    for warning in warnings {
        out.push_str("    ptn_emit_warning(&runtime.diagnostics, \"The magic method ");
        out.push_str(&c_string(&warning.class_name));
        out.push_str("::");
        out.push_str(&c_string(&warning.method_name));
        out.push_str("() must have public visibility\", ");
        out.push_str(&warning.line.to_string());
        out.push_str(");\n");
    }
}

fn emit_return_type_boundary(
    out: &mut String,
    return_type: &TypeHint,
    function_name: &str,
    return_by_ref: bool,
) {
    match return_type {
        TypeHint::Null => {
            out.push_str("    if (ptn_value_deref(ptn_return_value).type != PTN_NULL) {\n");
            out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
            out.push_str(&c_string(function_name));
            out.push_str("() return value must be of type null\");\n");
            out.push_str("        ptn_runtime_free(&runtime);\n");
            out.push_str("        exit(255);\n");
            out.push_str("    }\n");
        }
        TypeHint::Array => {
            out.push_str("    if (ptn_value_deref(ptn_return_value).type != PTN_ARRAY) {\n");
            out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
            out.push_str(&c_string(function_name));
            out.push_str("() return value must be of type array\");\n");
            out.push_str("        ptn_runtime_free(&runtime);\n");
            out.push_str("        exit(255);\n");
            out.push_str("    }\n");
        }
        TypeHint::Int | TypeHint::Float | TypeHint::String | TypeHint::Bool => {
            let cast_helper = type_hint_scalar_cast_helper(Some(return_type))
                .expect("scalar return type should map to cast helper");
            if return_by_ref {
                out.push_str("    PtnValue ptn_typed_return_value = ");
                out.push_str(cast_helper);
                out.push_str("(ptn_return_value);\n");
                out.push_str("    if (ptn_return_value.type == PTN_REFERENCE) {\n");
                out.push_str("        ptn_reference_assign(ptn_return_value.as.reference, ptn_typed_return_value);\n");
                out.push_str("        ptn_value_drop(&ptn_typed_return_value);\n");
                out.push_str("    } else {\n");
                out.push_str("        ptn_value_drop(&ptn_return_value);\n");
                out.push_str("        ptn_return_value = ptn_typed_return_value;\n");
                out.push_str("    }\n");
            } else {
                out.push_str("    PtnValue ptn_typed_return_value = ");
                out.push_str(cast_helper);
                out.push_str("(ptn_return_value);\n");
                out.push_str("    ptn_value_drop(&ptn_return_value);\n");
                out.push_str("    ptn_return_value = ptn_typed_return_value;\n");
            }
        }
        TypeHint::Class(class_name) => {
            out.push_str("    if (!ptn_value_satisfies_class_type_hint(ptn_return_value, \"");
            out.push_str(&c_string(class_name));
            out.push_str("\")) {\n");
            out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
            out.push_str(&c_string(function_name));
            out.push_str("() return value must be of type ");
            out.push_str(&c_string(class_name));
            out.push_str("\");\n");
            out.push_str("        ptn_runtime_free(&runtime);\n");
            out.push_str("        exit(255);\n");
            out.push_str("    }\n");
        }
        TypeHint::Mixed | TypeHint::Void => {}
    }
}

fn type_hint_scalar_cast_helper(type_hint: Option<&TypeHint>) -> Option<&'static str> {
    match type_hint {
        Some(TypeHint::Int) => Some("ptn_cast_int"),
        Some(TypeHint::Float) => Some("ptn_cast_float"),
        Some(TypeHint::String) => Some("ptn_cast_string"),
        Some(TypeHint::Bool) => Some("ptn_cast_bool"),
        Some(
            TypeHint::Null
            | TypeHint::Array
            | TypeHint::Mixed
            | TypeHint::Void
            | TypeHint::Class(_),
        )
        | None => None,
    }
}

fn emit_user_function_dispatch(
    out: &mut String,
    functions: &[FunctionDecl],
    classes: &[ClassDecl],
) {
    out.push_str(
        "\nstatic int ptn_user_function_exists(PtnRuntime *runtime, const char *name) {\n",
    );
    if functions
        .iter()
        .all(|function| function.is_anonymous || function.class_name.is_some())
    {
        out.push_str("    (void)runtime;\n");
        out.push_str("    (void)name;\n");
    }
    for (index, function) in functions.iter().enumerate() {
        if function.is_anonymous || function.class_name.is_some() {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&function.name));
        out.push_str("\")) {\n");
        out.push_str("        return runtime->declared_user_functions == NULL || runtime->declared_user_functions[");
        out.push_str(&index.to_string());
        out.push_str("];\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str("\nstatic PtnFunctionMetadata ptn_user_function_metadata(const char *name) {\n");
    if functions
        .iter()
        .all(|function| function.is_anonymous || function.class_name.is_some())
    {
        out.push_str("    (void)name;\n");
    }
    for (function_index, function) in functions.iter().enumerate() {
        if function.is_anonymous || function.class_name.is_some() {
            continue;
        }
        let required_parameter_count = function
            .parameters
            .iter()
            .filter(|parameter| !parameter.is_variadic && parameter.default_value.is_none())
            .count();
        let is_variadic = function
            .parameters
            .iter()
            .any(|parameter| parameter.is_variadic);
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&function.name));
        out.push_str("\")) {\n");
        let parameter_names = emit_function_metadata_parameter_names(
            out,
            "        ",
            &format!("ptn_function_{function_index}_parameter_names"),
            &function.parameters,
        );
        out.push_str("        return ptn_function_metadata_found(\"");
        out.push_str(&c_string(&function.name));
        out.push_str("\", 0, ");
        out.push_str(&function.parameters.len().to_string());
        out.push_str(", ");
        out.push_str(&required_parameter_count.to_string());
        out.push_str(", ");
        out.push_str(if is_variadic { "1" } else { "0" });
        out.push_str(", ");
        out.push_str(&parameter_names);
        out.push_str(");\n");
        out.push_str("    }\n");
    }
    for class in classes {
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            let function = &functions[method.function_index];
            let required_parameter_count = function
                .parameters
                .iter()
                .filter(|parameter| !parameter.is_variadic && parameter.default_value.is_none())
                .count();
            let is_variadic = function
                .parameters
                .iter()
                .any(|parameter| parameter.is_variadic);
            out.push_str("    if (ptn_ascii_case_equal(name, \"");
            out.push_str(&c_string(&class.name));
            out.push_str("::");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            let parameter_names = emit_function_metadata_parameter_names(
                out,
                "        ",
                &format!("ptn_function_{}_parameter_names", method.function_index),
                &function.parameters,
            );
            out.push_str("        return ptn_function_metadata_found(\"");
            out.push_str(&c_string(&class.name));
            out.push_str("::");
            out.push_str(&c_string(&method.name));
            out.push_str("\", 0, ");
            out.push_str(&function.parameters.len().to_string());
            out.push_str(", ");
            out.push_str(&required_parameter_count.to_string());
            out.push_str(", ");
            out.push_str(if is_variadic { "1" } else { "0" });
            out.push_str(", ");
            out.push_str(&parameter_names);
            out.push_str(");\n");
            out.push_str("    }\n");
        }
    }
    out.push_str("    return ptn_function_metadata_not_found();\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PtnValue ptn_call_user_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line, int *found) {\n",
    );
    if functions.iter().all(|function| {
        function.is_anonymous || (function.class_name.is_some() && !function.is_static)
    }) {
        out.push_str("    (void)runtime;\n");
        out.push_str("    (void)name;\n");
        out.push_str("    (void)argc;\n");
        out.push_str("    (void)args;\n");
        out.push_str("    (void)line;\n");
    }
    for (index, function) in functions.iter().enumerate() {
        if function.is_anonymous || (function.class_name.is_some() && !function.is_static) {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&function.name));
        out.push_str("\")) {\n");
        out.push_str("        if (runtime->declared_user_functions != NULL && !runtime->declared_user_functions[");
        out.push_str(&index.to_string());
        out.push_str("]) {\n");
        out.push_str("            return ptn_null();\n");
        out.push_str("        }\n");
        out.push_str("        *found = 1;\n");
        if function.is_static {
            if let Some((declaring_class, method_name, visibility)) =
                static_method_visibility_for_function(index, classes)
            {
                out.push_str("        if (!ptn_declared_method_visible(");
                out.push_str(c_property_visibility(visibility));
                out.push_str(", \"");
                out.push_str(&c_string(declaring_class));
                out.push_str("\", \"");
                out.push_str(&c_string(declaring_class));
                out.push_str("\", \"");
                out.push_str(&c_string(method_name));
                out.push_str("\", runtime->current_class_name)) {\n");
                out.push_str("            return ptn_throw_method_visibility_error(runtime, \"");
                out.push_str(&c_string(declaring_class));
                out.push_str("\", \"");
                out.push_str(&c_string(method_name));
                out.push_str("\", ");
                out.push_str(c_property_visibility(visibility));
                out.push_str(", line);\n");
                out.push_str("        }\n");
            }
        }
        out.push_str("        return ");
        out.push_str(&user_function_c_name(index));
        if function.is_static {
            let called_class_name = function
                .class_name
                .as_deref()
                .unwrap_or(function.name.as_str());
            out.push_str("(runtime, ptn_string(\"");
            out.push_str(&c_string(called_class_name));
            out.push_str("\"), argc, args, line);\n");
        } else {
            out.push_str("(runtime, ptn_null(), argc, args, line);\n");
        }
        out.push_str("    }\n");
    }
    for class in classes {
        for method in class_public_method_lookup_chain(class, classes) {
            if !method.is_static {
                continue;
            }
            out.push_str("    if (ptn_ascii_case_equal(name, \"");
            out.push_str(&c_string(&class.name));
            out.push_str("::");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("        *found = 1;\n");
            out.push_str("        if (!ptn_declared_method_visible(");
            out.push_str(c_property_visibility(method.visibility));
            out.push_str(", \"");
            out.push_str(&c_string(method.declaring_class));
            out.push_str("\", \"");
            out.push_str(&c_string(&class.name));
            out.push_str("\", \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\", runtime->current_class_name)) {\n");
            out.push_str("            return ptn_throw_method_visibility_error(runtime, \"");
            out.push_str(&c_string(method.declaring_class));
            out.push_str("\", \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\", ");
            out.push_str(c_property_visibility(method.visibility));
            out.push_str(", line);\n");
            out.push_str("        }\n");
            out.push_str("        const char *ptn_previous_called_class = runtime->called_class_name_override;\n");
            out.push_str("        runtime->called_class_name_override = \"");
            out.push_str(&c_string(&class.name));
            out.push_str("\";\n");
            out.push_str("        PtnValue ptn_static_result = ");
            out.push_str(&user_function_c_name(method.function_index));
            out.push_str("(runtime, ptn_string(\"");
            out.push_str(&c_string(&class.name));
            out.push_str("\"), argc, args, line);\n");
            out.push_str(
                "        runtime->called_class_name_override = ptn_previous_called_class;\n",
            );
            out.push_str("        return ptn_static_result;\n");
            out.push_str("    }\n");
        }
    }
    out.push_str("    *found = 0;\n");
    out.push_str("    return ptn_null();\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_call_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line) {\n",
    );
    out.push_str("    int found = 0;\n");
    out.push_str(
        "    PtnValue result = ptn_call_user_function(runtime, name, argc, args, line, &found);\n",
    );
    out.push_str("    if (found) {\n");
    out.push_str("        return result;\n");
    out.push_str("    }\n");
    out.push_str("    return ptn_call_internal(runtime, name, argc, args, line);\n");
    out.push_str("}\n");
}

fn emit_private_property_metadata_prototype(out: &mut String) {
    out.push_str(
        "static const char *ptn_declared_private_property_class(const char *class_name, const char *property_name);\n",
    );
    out.push_str("static const char *ptn_declared_class_parent_name(const char *name);\n");
}

fn emit_method_visibility_prototypes(out: &mut String) {
    out.push_str("static PTN_UNUSED int ptn_declared_method_visible(int visibility, const char *declaring_class, const char *target_class_name, const char *method_name, const char *access_scope);\n");
    out.push_str("static PTN_UNUSED PtnValue ptn_throw_method_visibility_error(PtnRuntime *runtime, const char *declaring_class, const char *method_name, int visibility, size_t line);\n");
    out.push_str("static PTN_UNUSED int ptn_declared_class_method_is_callable(const char *class_name, const char *method_name, const char *access_scope);\n");
    out.push_str("static PTN_UNUSED int ptn_declared_class_static_method_is_callable(const char *class_name, const char *method_name, const char *access_scope);\n");
}

fn emit_class_metadata_helpers(out: &mut String, classes: &[ClassDecl]) {
    out.push_str(
        "\nstatic PTN_UNUSED const char *ptn_declared_private_property_class(const char *class_name, const char *property_name) {\n",
    );
    if classes.iter().all(|class| {
        class
            .properties
            .iter()
            .all(|property| property.visibility != PropertyVisibility::Private)
    }) {
        out.push_str("    (void)class_name;\n");
        out.push_str("    (void)property_name;\n");
    }
    for class in classes {
        let private_properties = class
            .properties
            .iter()
            .filter(|property| property.visibility == PropertyVisibility::Private)
            .collect::<Vec<_>>();
        if private_properties.is_empty() {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for property in private_properties {
            out.push_str("        if (strcmp(property_name, \"");
            out.push_str(&c_string(&property.name));
            out.push_str("\") == 0) {\n");
            out.push_str("            return \"");
            out.push_str(&c_string(&class.name));
            out.push_str("\";\n");
            out.push_str("        }\n");
        }
        out.push_str("    }\n");
    }
    out.push_str("    return NULL;\n");
    out.push_str("}\n");

    out.push_str("\nstatic PTN_UNUSED int ptn_declared_class_exists(const char *name) {\n");
    out.push_str("    if (ptn_ascii_case_equal(name, \"stdClass\")) {\n");
    out.push_str("        return 1;\n");
    out.push_str("    }\n");
    for class in classes {
        if class.is_interface {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        out.push_str("        return 1;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str("\nstatic PTN_UNUSED int ptn_declared_interface_exists(const char *name) {\n");
    for builtin in [
        "ArrayAccess",
        "Iterator",
        "IteratorAggregate",
        "Traversable",
        "Stringable",
        "Throwable",
        "DateTimeInterface",
        "Serializable",
    ] {
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(builtin);
        out.push_str("\")) {\n");
        out.push_str("        return 1;\n");
        out.push_str("    }\n");
    }
    for class in classes {
        if !class.is_interface {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        out.push_str("        return 1;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str("\nstatic PTN_UNUSED int ptn_declared_class_is_readonly(const char *name) {\n");
    if classes.is_empty() {
        out.push_str("    (void)name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        if class.is_readonly {
            out.push_str("        return 1;\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED const char *ptn_declared_class_parent_name(const char *name) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        if let Some(parent_name) = &class.parent_name {
            out.push_str("        return \"");
            out.push_str(&c_string(parent_name));
            out.push_str("\";\n");
        } else {
            out.push_str("        return NULL;\n");
        }
        out.push_str("    }\n");
    }
    out.push_str("    return NULL;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_is_same_or_descendant(const char *class_name, const char *ancestor_name) {\n",
    );
    out.push_str("    if (class_name == NULL || ancestor_name == NULL) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    const char *current = class_name;\n");
    out.push_str("    while (current != NULL) {\n");
    out.push_str("        if (ptn_ascii_case_equal(current, ancestor_name)) {\n");
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        current = ptn_declared_class_parent_name(current);\n");
    out.push_str("    }\n");
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_implements_interface_direct(const char *class_name, const char *interface_name) {\n",
    );
    if classes.iter().all(|class| class.interfaces.is_empty()) {
        out.push_str("    (void)class_name;\n");
        out.push_str("    (void)interface_name;\n");
    }
    for class in classes {
        if class.interfaces.is_empty() {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for interface in &class.interfaces {
            out.push_str("        if (ptn_ascii_case_equal(interface_name, \"");
            out.push_str(&c_string(interface));
            out.push_str("\")) {\n");
            out.push_str("            return 1;\n");
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_implements_interface(const char *class_name, const char *interface_name) {\n",
    );
    out.push_str("    if (class_name == NULL || interface_name == NULL) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    const char *current = class_name;\n");
    out.push_str("    while (current != NULL) {\n");
    out.push_str(
        "        if (ptn_declared_class_implements_interface_direct(current, interface_name)) {\n",
    );
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        current = ptn_declared_class_parent_name(current);\n");
    out.push_str("    }\n");
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_scope_allows(const char *access_scope, const char *declaring_class) {\n",
    );
    out.push_str("    if (access_scope == NULL || declaring_class == NULL) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str(
        "    return ptn_declared_class_is_same_or_descendant(access_scope, declaring_class) ||\n",
    );
    out.push_str(
        "        ptn_declared_class_is_same_or_descendant(declaring_class, access_scope);\n",
    );
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_method_visibility_allows(const char *access_scope, const char *declaring_class, int visibility) {\n",
    );
    out.push_str("    if (visibility == PTN_PROPERTY_PUBLIC) {\n");
    out.push_str("        return 1;\n");
    out.push_str("    }\n");
    out.push_str("    if (visibility == PTN_PROPERTY_PRIVATE) {\n");
    out.push_str("        return access_scope != NULL && declaring_class != NULL && ptn_ascii_case_equal(access_scope, declaring_class);\n");
    out.push_str("    }\n");
    out.push_str("    return ptn_declared_class_scope_allows(access_scope, declaring_class);\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_protected_static_method_root_allows(const char *access_scope, const char *target_class, const char *method_name) {\n",
    );
    out.push_str(
        "    if (access_scope == NULL || target_class == NULL || method_name == NULL) {\n",
    );
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    for class in classes {
        let has_protected_static = class
            .methods
            .iter()
            .any(|method| method.is_static && method.visibility == PropertyVisibility::Protected);
        if !has_protected_static {
            continue;
        }
        out.push_str("    if (ptn_declared_class_is_same_or_descendant(access_scope, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\") && ptn_declared_class_is_same_or_descendant(target_class, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for method in class
            .methods
            .iter()
            .filter(|method| method.is_static && method.visibility == PropertyVisibility::Protected)
        {
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            return 1;\n");
            out.push_str("        }\n");
        }
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED void ptn_throw_declared_method_visibility_error(PtnRuntime *runtime, const char *visibility_name, const char *declaring_class, const char *method_name, size_t line) {\n",
    );
    out.push_str(
        "    const char *access_scope = runtime != NULL ? runtime->current_class_name : NULL;\n",
    );
    out.push_str("    int needed;\n");
    out.push_str("    if (access_scope == NULL) {\n");
    out.push_str("        needed = snprintf(NULL, 0, \"Call to %s method %s::%s() from global scope\", visibility_name, declaring_class, method_name);\n");
    out.push_str("    } else {\n");
    out.push_str("        needed = snprintf(NULL, 0, \"Call to %s method %s::%s() from scope %s\", visibility_name, declaring_class, method_name, access_scope);\n");
    out.push_str("    }\n");
    out.push_str("    if (needed < 0) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    char *message = malloc((size_t)needed + 1);\n");
    out.push_str("    if (message == NULL) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    if (access_scope == NULL) {\n");
    out.push_str("        snprintf(message, (size_t)needed + 1, \"Call to %s method %s::%s() from global scope\", visibility_name, declaring_class, method_name);\n");
    out.push_str("    } else {\n");
    out.push_str("        snprintf(message, (size_t)needed + 1, \"Call to %s method %s::%s() from scope %s\", visibility_name, declaring_class, method_name, access_scope);\n");
    out.push_str("    }\n");
    out.push_str("    ptn_throw_exception_owned_message_at(runtime, \"Error\", message, runtime != NULL ? runtime->source_path : NULL, line);\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_property_exists(const char *class_name, const char *property_name) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    if classes
        .iter()
        .all(|class| class_property_exists_chain(class, classes).is_empty())
    {
        out.push_str("    (void)property_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for property in class_property_exists_chain(class, classes) {
            if property.visibility == PropertyVisibility::Private
                && !property
                    .declaring_class
                    .eq_ignore_ascii_case(class.name.as_str())
            {
                continue;
            }
            out.push_str("        if (strcmp(property_name, \"");
            out.push_str(&c_string(property.name));
            out.push_str("\") == 0) {\n");
            out.push_str("            return 1;\n");
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_method_exists(const char *class_name, const char *method_name) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    if classes
        .iter()
        .all(|class| class_method_lookup_chain(class, classes).is_empty())
    {
        out.push_str("    (void)method_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            return 1;\n");
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_method_is_callable(const char *class_name, const char *method_name, const char *access_scope) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    if classes
        .iter()
        .all(|class| class_method_lookup_chain(class, classes).is_empty())
    {
        out.push_str("    (void)method_name;\n");
        out.push_str("    (void)access_scope;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            if (ptn_declared_method_visibility_allows(access_scope, \"");
            out.push_str(&c_string(entry.declaring_class));
            out.push_str("\", ");
            out.push_str(c_method_visibility(method.visibility));
            out.push_str(")) {\n");
            out.push_str("                return 1;\n");
            out.push_str("            }\n");
            if method.visibility == PropertyVisibility::Protected {
                out.push_str("            return ptn_declared_protected_static_method_root_allows(access_scope, class_name, method_name);\n");
            } else {
                out.push_str("            return 0;\n");
            }
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_static_method_exists(const char *class_name, const char *method_name, const char *access_scope) {\n",
    );
    out.push_str("    (void)access_scope;\n");
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    let has_static_methods = classes.iter().any(|class| {
        class_method_lookup_chain(class, classes)
            .into_iter()
            .any(|entry| entry.method.is_static)
    });
    if !has_static_methods {
        out.push_str("    (void)method_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            if !method.is_static {
                continue;
            }
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            if (ptn_declared_method_visibility_allows(access_scope, \"");
            out.push_str(&c_string(entry.declaring_class));
            out.push_str("\", ");
            out.push_str(c_method_visibility(method.visibility));
            out.push_str(")) {\n");
            out.push_str("                return 1;\n");
            out.push_str("            }\n");
            if method.visibility == PropertyVisibility::Protected {
                out.push_str("            return ptn_declared_protected_static_method_root_allows(access_scope, class_name, method_name);\n");
            } else {
                out.push_str("            return 0;\n");
            }
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_has_static_method(const char *class_name, const char *method_name) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    if !has_static_methods {
        out.push_str("    (void)method_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            if !method.is_static {
                continue;
            }
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            return 1;\n");
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str("\nstatic PTN_UNUSED const char *ptn_method_visibility_name(int visibility) {\n");
    out.push_str("    if (visibility == PTN_PROPERTY_PRIVATE) {\n");
    out.push_str("        return \"private\";\n");
    out.push_str("    }\n");
    out.push_str("    if (visibility == PTN_PROPERTY_PROTECTED) {\n");
    out.push_str("        return \"protected\";\n");
    out.push_str("    }\n");
    out.push_str("    return \"public\";\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_throw_method_visibility_error(PtnRuntime *runtime, const char *declaring_class, const char *method_name, int visibility, size_t line) {\n",
    );
    out.push_str("    const char *visibility_name = ptn_method_visibility_name(visibility);\n");
    out.push_str("    const char *scope = runtime->current_class_name;\n");
    out.push_str("    int needed = scope == NULL ?\n");
    out.push_str("        snprintf(NULL, 0, \"Call to %s method %s::%s() from global scope\", visibility_name, declaring_class, method_name) :\n");
    out.push_str("        snprintf(NULL, 0, \"Call to %s method %s::%s() from scope %s\", visibility_name, declaring_class, method_name, scope);\n");
    out.push_str("    if (needed < 0) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    char *message = malloc((size_t)needed + 1);\n");
    out.push_str("    if (message == NULL) {\n");
    out.push_str("        ptn_abort_out_of_memory();\n");
    out.push_str("    }\n");
    out.push_str("    if (scope == NULL) {\n");
    out.push_str("        snprintf(message, (size_t)needed + 1, \"Call to %s method %s::%s() from global scope\", visibility_name, declaring_class, method_name);\n");
    out.push_str("    } else {\n");
    out.push_str("        snprintf(message, (size_t)needed + 1, \"Call to %s method %s::%s() from scope %s\", visibility_name, declaring_class, method_name, scope);\n");
    out.push_str("    }\n");
    out.push_str(
        "    ptn_throw_exception_at(runtime, \"Error\", message, runtime->source_path, line);\n",
    );
    out.push_str("    free(message);\n");
    out.push_str("    return ptn_null();\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_direct_non_private_method_exists(const char *class_name, const char *method_name) {\n",
    );
    if classes.iter().all(|class| {
        class
            .methods
            .iter()
            .all(|method| method.visibility == PropertyVisibility::Private)
    }) {
        out.push_str("    (void)class_name;\n");
        out.push_str("    (void)method_name;\n");
    }
    for class in classes {
        let direct_non_private_methods = class
            .methods
            .iter()
            .filter(|method| method.visibility != PropertyVisibility::Private)
            .collect::<Vec<_>>();
        if direct_non_private_methods.is_empty() {
            continue;
        }
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for method in direct_non_private_methods {
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            return 1;\n");
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_classes_share_non_private_ancestor_method(const char *left_class, const char *right_class, const char *method_name) {\n",
    );
    out.push_str("    if (left_class == NULL || right_class == NULL) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    const char *left = left_class;\n");
    out.push_str("    while (left != NULL) {\n");
    out.push_str("        const char *right = right_class;\n");
    out.push_str("        while (right != NULL) {\n");
    out.push_str("            if (ptn_ascii_case_equal(left, right) && ptn_declared_class_direct_non_private_method_exists(left, method_name)) {\n");
    out.push_str("                return 1;\n");
    out.push_str("            }\n");
    out.push_str("            right = ptn_declared_class_parent_name(right);\n");
    out.push_str("        }\n");
    out.push_str("        left = ptn_declared_class_parent_name(left);\n");
    out.push_str("    }\n");
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_method_visible(int visibility, const char *declaring_class, const char *target_class_name, const char *method_name, const char *access_scope) {\n",
    );
    out.push_str("    if (visibility == PTN_PROPERTY_PUBLIC) {\n");
    out.push_str("        return 1;\n");
    out.push_str("    }\n");
    out.push_str("    if (access_scope == NULL) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    if (visibility == PTN_PROPERTY_PRIVATE) {\n");
    out.push_str("        return ptn_ascii_case_equal(access_scope, declaring_class);\n");
    out.push_str("    }\n");
    out.push_str(
        "    return ptn_declared_class_is_same_or_descendant(access_scope, declaring_class) ||\n",
    );
    out.push_str("        ptn_declared_classes_share_non_private_ancestor_method(access_scope, target_class_name, method_name);\n");
    out.push_str("}\n");
    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_static_method_is_callable(const char *class_name, const char *method_name, const char *access_scope) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    let has_static_methods = classes.iter().any(|class| {
        class_method_lookup_chain(class, classes)
            .into_iter()
            .any(|method| method.is_static)
    });
    if !has_static_methods {
        out.push_str("    (void)method_name;\n");
        out.push_str("    (void)access_scope;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for method in class_method_lookup_chain(class, classes) {
            if !method.is_static {
                continue;
            }
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            return ptn_declared_method_visible(");
            out.push_str(c_property_visibility(method.visibility));
            out.push_str(", \"");
            out.push_str(&c_string(method.declaring_class));
            out.push_str("\", \"");
            out.push_str(&c_string(&class.name));
            out.push_str("\", \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\", access_scope);\n");
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_has_call_magic(const char *class_name) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        if class_magic_call_method(class, classes).is_some() {
            out.push_str("        return 1;\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_class_has_invoke_magic(const char *class_name) {\n",
    );
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        if class_magic_invoke_method(class, classes).is_some() {
            out.push_str("        return 1;\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");
}

fn class_by_name<'a>(classes: &'a [ClassDecl], name: &str) -> Option<&'a ClassDecl> {
    classes
        .iter()
        .find(|class| class.name.eq_ignore_ascii_case(name))
}

fn static_method_visibility_for_function(
    function_index: usize,
    classes: &[ClassDecl],
) -> Option<(&str, &str, PropertyVisibility)> {
    classes.iter().find_map(|class| {
        class
            .methods
            .iter()
            .find(|method| method.function_index == function_index && method.is_static)
            .map(|method| (class.name.as_str(), method.name.as_str(), method.visibility))
    })
}

struct ClassMethodLookupEntry<'a> {
    declaring_class: &'a str,
    method: &'a crate::ir::MethodDecl,
}

impl std::ops::Deref for ClassMethodLookupEntry<'_> {
    type Target = crate::ir::MethodDecl;

    fn deref(&self) -> &Self::Target {
        self.method
    }
}

fn class_method_lookup_chain<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Vec<ClassMethodLookupEntry<'a>> {
    fn collect<'a>(
        class: &'a ClassDecl,
        classes: &'a [ClassDecl],
        seen_classes: &mut HashSet<String>,
        seen_methods: &mut HashSet<String>,
        methods: &mut Vec<ClassMethodLookupEntry<'a>>,
    ) {
        for method in &class.methods {
            let method_key = method.name.to_ascii_lowercase();
            if method.visibility == PropertyVisibility::Private || seen_methods.insert(method_key) {
                methods.push(ClassMethodLookupEntry {
                    declaring_class: class.name.as_str(),
                    method,
                });
            }
        }

        let Some(parent_name) = &class.parent_name else {
            return;
        };
        let lookup_name = parent_name.to_ascii_lowercase();
        if !seen_classes.insert(lookup_name) {
            return;
        }
        if let Some(parent) = class_by_name(classes, parent_name) {
            collect(parent, classes, seen_classes, seen_methods, methods);
        }
    }

    let mut methods = Vec::new();
    let mut seen_classes = HashSet::from([class.name.to_ascii_lowercase()]);
    let mut seen_methods = HashSet::new();
    collect(
        class,
        classes,
        &mut seen_classes,
        &mut seen_methods,
        &mut methods,
    );
    methods
}

fn class_public_method_lookup_chain<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Vec<ClassMethodLookupEntry<'a>> {
    class_method_lookup_chain(class, classes)
        .into_iter()
        .filter(|entry| entry.method.visibility == PropertyVisibility::Public)
        .collect()
}

struct ClassPropertyExistsEntry<'a> {
    declaring_class: &'a str,
    name: &'a str,
    visibility: PropertyVisibility,
}

fn class_property_exists_chain<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Vec<ClassPropertyExistsEntry<'a>> {
    fn collect<'a>(
        class: &'a ClassDecl,
        classes: &'a [ClassDecl],
        seen_classes: &mut HashSet<String>,
        properties: &mut Vec<ClassPropertyExistsEntry<'a>>,
    ) {
        let lookup_name = class.name.to_ascii_lowercase();
        if !seen_classes.insert(lookup_name) {
            return;
        }
        properties.extend(
            class
                .properties
                .iter()
                .map(|property| ClassPropertyExistsEntry {
                    declaring_class: class.name.as_str(),
                    name: property.name.as_str(),
                    visibility: property.visibility,
                }),
        );
        properties.extend(class.static_properties.iter().map(|property| {
            ClassPropertyExistsEntry {
                declaring_class: class.name.as_str(),
                name: property.name.as_str(),
                visibility: property.visibility,
            }
        }));

        let Some(parent_name) = &class.parent_name else {
            return;
        };
        if let Some(parent) = class_by_name(classes, parent_name) {
            collect(parent, classes, seen_classes, properties);
        }
    }

    let mut properties = Vec::new();
    let mut seen_classes = HashSet::new();
    collect(class, classes, &mut seen_classes, &mut properties);
    properties
}

fn class_property_initialization_chain(
    class: &ClassDecl,
    classes: &[ClassDecl],
) -> Vec<(String, crate::ir::PropertyDecl)> {
    fn collect(
        class: &ClassDecl,
        classes: &[ClassDecl],
        seen_classes: &mut HashSet<String>,
        properties: &mut Vec<(String, crate::ir::PropertyDecl)>,
    ) {
        let lookup_name = class.name.to_ascii_lowercase();
        if !seen_classes.insert(lookup_name) {
            return;
        }
        if let Some(parent_name) = &class.parent_name {
            if let Some(parent) = class_by_name(classes, parent_name) {
                collect(parent, classes, seen_classes, properties);
            }
        }
        properties.extend(
            class
                .properties
                .iter()
                .cloned()
                .map(|property| (class.name.clone(), property)),
        );
    }

    let mut properties = Vec::new();
    let mut seen_classes = HashSet::new();
    collect(class, classes, &mut seen_classes, &mut properties);
    properties
}

fn class_constructor_method<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Option<&'a crate::ir::MethodDecl> {
    class_public_method_lookup_chain(class, classes)
        .into_iter()
        .find(|method| !method.is_static && method.name.eq_ignore_ascii_case("__construct"))
        .map(|method| method.method)
}

fn class_magic_call_method<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Option<&'a crate::ir::MethodDecl> {
    class_public_method_lookup_chain(class, classes)
        .into_iter()
        .find(|method| !method.is_static && method.name.eq_ignore_ascii_case("__call"))
        .map(|method| method.method)
}

fn class_magic_invoke_method<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Option<&'a crate::ir::MethodDecl> {
    class_public_method_lookup_chain(class, classes)
        .into_iter()
        .find(|method| !method.is_static && method.name.eq_ignore_ascii_case("__invoke"))
        .map(|method| method.method)
}

fn class_magic_isset_method<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Option<&'a crate::ir::MethodDecl> {
    class_public_method_lookup_chain(class, classes)
        .into_iter()
        .find(|method| !method.is_static && method.name.eq_ignore_ascii_case("__isset"))
        .map(|method| method.method)
}

fn class_magic_get_method<'a>(
    class: &'a ClassDecl,
    classes: &'a [ClassDecl],
) -> Option<&'a crate::ir::MethodDecl> {
    class_public_method_lookup_chain(class, classes)
        .into_iter()
        .find(|method| !method.is_static && method.name.eq_ignore_ascii_case("__get"))
        .map(|method| method.method)
}

fn emit_magic_property_read_dispatch(out: &mut String, classes: &[ClassDecl]) {
    out.push_str(
        "\nstatic PTN_UNUSED int ptn_declared_magic_property_read(PtnRuntime *runtime, PtnValue receiver, const char *property, size_t line, PtnValue *value_out) {\n",
    );
    out.push_str("    PtnValue resolved = ptn_value_deref(receiver);\n");
    out.push_str("    if (resolved.type != PTN_OBJECT) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    const char *class_name = resolved.as.object->class_name;\n");
    for class in classes {
        let Some(isset_method) = class_magic_isset_method(class, classes) else {
            continue;
        };
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        out.push_str("        PtnValue ptn_isset_args[1];\n");
        out.push_str("        ptn_isset_args[0] = ptn_string(property);\n");
        out.push_str("        PtnValue ptn_isset_result = ");
        out.push_str(&user_function_c_name(isset_method.function_index));
        out.push_str("(runtime, resolved, 1, ptn_isset_args, line);\n");
        out.push_str(
            "        int ptn_isset_truthy = ptn_is_truthy(ptn_value_deref(ptn_isset_result));\n",
        );
        out.push_str("        ptn_value_destroy(&ptn_isset_result);\n");
        out.push_str("        ptn_value_destroy(&ptn_isset_args[0]);\n");
        out.push_str("        if (!ptn_isset_truthy) {\n");
        out.push_str("            return 0;\n");
        out.push_str("        }\n");
        if let Some(get_method) = class_magic_get_method(class, classes) {
            out.push_str("        PtnValue ptn_get_args[1];\n");
            out.push_str("        ptn_get_args[0] = ptn_string(property);\n");
            out.push_str("        *value_out = ");
            out.push_str(&user_function_c_name(get_method.function_index));
            out.push_str("(runtime, resolved, 1, ptn_get_args, line);\n");
            out.push_str("        ptn_value_destroy(&ptn_get_args[0]);\n");
        } else {
            out.push_str(
                "        *value_out = ptn_object_read_property(runtime, resolved, property, NULL, line);\n",
            );
        }
        out.push_str("        return 1;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");
}

fn emit_callable_validation_helpers(out: &mut String) {
    out.push_str(
        "\nstatic int ptn_callable_array_parts(PtnValue callable, PtnValue *scope_out, PtnValue *method_out) {\n",
    );
    out.push_str("    callable = ptn_value_deref(callable);\n");
    out.push_str("    if (callable.type != PTN_ARRAY || callable.as.array->len != 2) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    PtnArrayKey scope_key = ptn_array_int_key(0);\n");
    out.push_str("    PtnArrayKey method_key = ptn_array_int_key(1);\n");
    out.push_str(
        "    PtnArrayEntry *scope_entry = ptn_array_entry_for_key(callable.as.array, scope_key);\n",
    );
    out.push_str("    PtnArrayEntry *method_entry = ptn_array_entry_for_key(callable.as.array, method_key);\n");
    out.push_str("    ptn_array_key_free(scope_key);\n");
    out.push_str("    ptn_array_key_free(method_key);\n");
    out.push_str("    if (scope_entry == NULL || method_entry == NULL) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    *scope_out = ptn_value_deref(scope_entry->value);\n");
    out.push_str("    *method_out = ptn_value_deref(method_entry->value);\n");
    out.push_str("    return 1;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic int ptn_callable_is_valid(PtnRuntime *runtime, PtnValue callable, int syntax_only) {\n",
    );
    out.push_str("    PtnValue resolved = ptn_value_deref(callable);\n");
    out.push_str(
        "    const char *access_scope = runtime == NULL ? NULL : runtime->current_class_name;\n",
    );
    out.push_str("    if (resolved.type == PTN_CLOSURE) {\n");
    out.push_str("        return 1;\n");
    out.push_str("    }\n");
    out.push_str("    if (resolved.type == PTN_STRING) {\n");
    out.push_str("        if (syntax_only) {\n");
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        char *name = ptn_value_to_string(resolved);\n");
    out.push_str("        char *separator = strstr(name, \"::\");\n");
    out.push_str("        int valid = 0;\n");
    out.push_str("        if (separator != NULL) {\n");
    out.push_str("            *separator = '\\0';\n");
    out.push_str(
        "            valid = ptn_declared_class_static_method_is_callable(name, separator + 2, access_scope);\n",
    );
    out.push_str("            *separator = ':';\n");
    out.push_str("        }\n");
    out.push_str("        if (!valid) {\n");
    out.push_str("            valid = ptn_user_function_exists(runtime, name) || ptn_find_internal_function(name) != NULL;\n");
    out.push_str("        }\n");
    out.push_str("        free(name);\n");
    out.push_str("        return valid;\n");
    out.push_str("    }\n");
    out.push_str("    if (resolved.type == PTN_OBJECT) {\n");
    out.push_str(
        "        return ptn_declared_class_has_invoke_magic(resolved.as.object->class_name);\n",
    );
    out.push_str("    }\n");
    out.push_str("    PtnValue scope;\n");
    out.push_str("    PtnValue method;\n");
    out.push_str("    if (!ptn_callable_array_parts(resolved, &scope, &method) || method.type != PTN_STRING) {\n");
    out.push_str("        return 0;\n");
    out.push_str("    }\n");
    out.push_str("    if (scope.type == PTN_OBJECT) {\n");
    out.push_str("        if (syntax_only) {\n");
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        char *method_name = ptn_value_to_string(method);\n");
    out.push_str("        int valid = 0;\n");
    out.push_str("        if (ptn_internal_class_exists_name(scope.as.object->class_name)) {\n");
    out.push_str("            valid = ptn_internal_class_method_exists(scope.as.object->class_name, method_name);\n");
    out.push_str("        }\n");
    out.push_str("        valid = valid || ptn_declared_class_method_is_callable(scope.as.object->class_name, method_name, access_scope) || ptn_declared_class_has_call_magic(scope.as.object->class_name);\n");
    out.push_str("        free(method_name);\n");
    out.push_str("        return valid;\n");
    out.push_str("    }\n");
    out.push_str("    if (scope.type == PTN_EXCEPTION) {\n");
    out.push_str("        if (syntax_only) {\n");
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        char *method_name = ptn_value_to_string(method);\n");
    out.push_str("        int valid = ptn_exception_name_equal(method_name, \"getMessage\") || ptn_exception_name_equal(method_name, \"getTrace\");\n");
    out.push_str("        free(method_name);\n");
    out.push_str("        return valid;\n");
    out.push_str("    }\n");
    out.push_str("    if (scope.type == PTN_CLOSURE) {\n");
    out.push_str("        if (syntax_only) {\n");
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        char *method_name = ptn_value_to_string(method);\n");
    out.push_str("        int valid = ptn_ascii_case_equal(method_name, \"__invoke\");\n");
    out.push_str("        free(method_name);\n");
    out.push_str("        return valid;\n");
    out.push_str("    }\n");
    out.push_str("    if (scope.type == PTN_STRING) {\n");
    out.push_str("        if (syntax_only) {\n");
    out.push_str("            return 1;\n");
    out.push_str("        }\n");
    out.push_str("        char *class_name = ptn_value_to_string(scope);\n");
    out.push_str("        char *method_name = ptn_value_to_string(method);\n");
    out.push_str("        int needed = snprintf(NULL, 0, \"%s::%s\", class_name, method_name);\n");
    out.push_str("        if (needed < 0) {\n");
    out.push_str("            free(method_name);\n");
    out.push_str("            free(class_name);\n");
    out.push_str("            ptn_abort_out_of_memory();\n");
    out.push_str("        }\n");
    out.push_str("        char *function_name = malloc((size_t)needed + 1);\n");
    out.push_str("        if (function_name == NULL) {\n");
    out.push_str("            free(method_name);\n");
    out.push_str("            free(class_name);\n");
    out.push_str("            ptn_abort_out_of_memory();\n");
    out.push_str("        }\n");
    out.push_str("        snprintf(function_name, (size_t)needed + 1, \"%s::%s\", class_name, method_name);\n");
    out.push_str("        int valid = ptn_declared_class_static_method_is_callable(class_name, method_name, access_scope) || ptn_find_internal_function(function_name) != NULL;\n");
    out.push_str("        free(function_name);\n");
    out.push_str("        free(method_name);\n");
    out.push_str("        free(class_name);\n");
    out.push_str("        return valid;\n");
    out.push_str("    }\n");
    out.push_str("    return 0;\n");
    out.push_str("}\n");
}

fn emit_method_dispatch(
    out: &mut String,
    classes: &[ClassDecl],
    needs_closure_invoke_dispatch: bool,
) {
    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_call_declared_method(PtnRuntime *runtime, PtnValue receiver, const char *method_name, size_t argc, const PtnValue *args, size_t line) {\n",
    );
    out.push_str("    PtnValue resolved = ptn_value_deref(receiver);\n");
    out.push_str("    if (resolved.type == PTN_EXCEPTION) {\n");
    out.push_str(
        "        return ptn_call_method(runtime, resolved, method_name, argc, args, line);\n",
    );
    out.push_str("    }\n");
    out.push_str("    if (resolved.type == PTN_CLOSURE) {\n");
    out.push_str("        if (ptn_ascii_case_equal(method_name, \"bindTo\")) {\n");
    out.push_str("            (void)argc;\n");
    out.push_str("            (void)args;\n");
    out.push_str("            (void)line;\n");
    out.push_str("            return ptn_closure_clone(runtime, resolved);\n");
    out.push_str("        }\n");
    if needs_closure_invoke_dispatch {
        out.push_str("        if (ptn_ascii_case_equal(method_name, \"__invoke\")) {\n");
        out.push_str("            const char *previous_name = runtime->by_ref_argument_function_name_override;\n");
        out.push_str("            runtime->by_ref_argument_function_name_override = \"Closure::__invoke\";\n");
        out.push_str("            PtnValue result = ptn_call_callable(runtime, resolved, argc, args, line);\n");
        out.push_str(
            "            runtime->by_ref_argument_function_name_override = previous_name;\n",
        );
        out.push_str("            return result;\n");
        out.push_str("        }\n");
    }
    out.push_str("        fputs(\"Fatal error: Call to undefined method Closure::\", stderr);\n");
    out.push_str("        fputs(method_name, stderr);\n");
    out.push_str("        fputc('\\n', stderr);\n");
    out.push_str("        exit(255);\n");
    out.push_str("    }\n");
    out.push_str("    if (resolved.type != PTN_OBJECT) {\n");
    out.push_str(
        "        fputs(\"Fatal error: call to a member function on non-object\\n\", stderr);\n",
    );
    out.push_str("        exit(255);\n");
    out.push_str("    }\n");
    out.push_str("    const char *class_name = resolved.as.object->class_name;\n");
    if classes.is_empty() {
        out.push_str("    (void)class_name;\n");
    }
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        out.push_str("        const char *ptn_inaccessible_visibility = NULL;\n");
        out.push_str("        const char *ptn_inaccessible_class = NULL;\n");
        out.push_str("        const char *ptn_inaccessible_method = NULL;\n");
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            if (ptn_declared_method_visibility_allows(runtime->current_class_name, \"");
            out.push_str(&c_string(entry.declaring_class));
            out.push_str("\", ");
            out.push_str(c_method_visibility(method.visibility));
            out.push_str(")");
            if method.visibility == PropertyVisibility::Protected {
                out.push_str(" || ptn_declared_protected_static_method_root_allows(runtime->current_class_name, class_name, method_name)");
            }
            out.push_str(") {\n");
            out.push_str("            return ");
            out.push_str(&user_function_c_name(method.function_index));
            out.push_str("(runtime, resolved, argc, args, line);\n");
            out.push_str("            }\n");
            out.push_str("            if (ptn_inaccessible_visibility == NULL) {\n");
            out.push_str("                ptn_inaccessible_visibility = \"");
            out.push_str(method_visibility_name(method.visibility));
            out.push_str("\";\n");
            out.push_str("                ptn_inaccessible_class = \"");
            out.push_str(&c_string(entry.declaring_class));
            out.push_str("\";\n");
            out.push_str("                ptn_inaccessible_method = \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\";\n");
            out.push_str("            }\n");
            out.push_str("        }\n");
        }
        if let Some(method) = class_magic_call_method(class, classes) {
            out.push_str("        PtnValue ptn_magic_args[2];\n");
            out.push_str("        ptn_magic_args[0] = ptn_string(method_name);\n");
            out.push_str("        ptn_magic_args[1] = ptn_array_from_literal_entries(0, NULL);\n");
            out.push_str("        for (size_t ptn_magic_arg_i = 0; ptn_magic_arg_i < argc; ptn_magic_arg_i++) {\n");
            out.push_str("            if (ptn_magic_arg_i > (size_t)INT64_MAX) {\n");
            out.push_str("                ptn_abort_out_of_memory();\n");
            out.push_str("            }\n");
            out.push_str("            ptn_array_set_entry(ptn_magic_args[1].as.array, ptn_array_int_key((int64_t)ptn_magic_arg_i), ptn_value_clone_deref(args[ptn_magic_arg_i]));\n");
            out.push_str("        }\n");
            out.push_str("        PtnValue ptn_magic_result = ");
            out.push_str(&user_function_c_name(method.function_index));
            out.push_str("(runtime, resolved, 2, ptn_magic_args, line);\n");
            out.push_str("        ptn_value_destroy(&ptn_magic_args[0]);\n");
            out.push_str("        ptn_value_destroy(&ptn_magic_args[1]);\n");
            out.push_str("        return ptn_magic_result;\n");
        }
        out.push_str("        if (ptn_inaccessible_visibility != NULL) {\n");
        out.push_str("            ptn_throw_declared_method_visibility_error(runtime, ptn_inaccessible_visibility, ptn_inaccessible_class, ptn_inaccessible_method, line);\n");
        out.push_str("            return ptn_null();\n");
        out.push_str("        }\n");
        out.push_str("    }\n");
    }
    out.push_str("    return ptn_call_method(runtime, resolved, method_name, argc, args, line);\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED int ptn_call_declared_method_in_scope(PtnRuntime *runtime, PtnValue receiver, const char *target_class_name, const char *method_name, const char *called_class_name, size_t argc, const PtnValue *args, size_t line, PtnValue *result_out) {\n",
    );
    out.push_str("    PtnValue resolved_receiver = ptn_value_deref(receiver);\n");
    out.push_str("    const char *effective_called_class = called_class_name != NULL ? called_class_name : target_class_name;\n");
    out.push_str("    (void)runtime;\n");
    out.push_str("    (void)receiver;\n");
    out.push_str("    (void)target_class_name;\n");
    out.push_str("    (void)method_name;\n");
    out.push_str("    (void)called_class_name;\n");
    out.push_str("    (void)argc;\n");
    out.push_str("    (void)args;\n");
    out.push_str("    (void)line;\n");
    out.push_str("    (void)result_out;\n");
    out.push_str("    (void)resolved_receiver;\n");
    out.push_str("    (void)effective_called_class;\n");
    for class in classes {
        out.push_str("    if (ptn_ascii_case_equal(target_class_name, \"");
        out.push_str(&c_string(&class.name));
        out.push_str("\")) {\n");
        for entry in class_method_lookup_chain(class, classes) {
            let method = entry.method;
            out.push_str("        if (ptn_ascii_case_equal(method_name, \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\")) {\n");
            out.push_str("            if (!(ptn_declared_method_visibility_allows(runtime->current_class_name, \"");
            out.push_str(&c_string(entry.declaring_class));
            out.push_str("\", ");
            out.push_str(c_method_visibility(method.visibility));
            out.push_str(")");
            if method.visibility == PropertyVisibility::Protected {
                out.push_str(" || ptn_declared_protected_static_method_root_allows(runtime->current_class_name, target_class_name, method_name)");
            }
            out.push_str(")) {\n");
            out.push_str("                ptn_throw_declared_method_visibility_error(runtime, \"");
            out.push_str(method_visibility_name(method.visibility));
            out.push_str("\", \"");
            out.push_str(&c_string(entry.declaring_class));
            out.push_str("\", \"");
            out.push_str(&c_string(&method.name));
            out.push_str("\", line);\n");
            out.push_str("                *result_out = ptn_null();\n");
            out.push_str("                return 1;\n");
            out.push_str("            }\n");
            if method.is_static {
                out.push_str("            const char *ptn_previous_called_class = runtime->called_class_name_override;\n");
                out.push_str(
                    "            runtime->called_class_name_override = effective_called_class;\n",
                );
                out.push_str("            *result_out = ");
                out.push_str(&user_function_c_name(method.function_index));
                out.push_str("(runtime, ptn_null(), argc, args, line);\n");
                out.push_str("            runtime->called_class_name_override = ptn_previous_called_class;\n");
                out.push_str("            return 1;\n");
            } else {
                out.push_str("            if (resolved_receiver.type != PTN_OBJECT || !ptn_declared_class_is_same_or_descendant(resolved_receiver.as.object->class_name, target_class_name)) {\n");
                out.push_str("                return 0;\n");
                out.push_str("            }\n");
                out.push_str("            const char *ptn_previous_called_class = runtime->called_class_name_override;\n");
                out.push_str(
                    "            runtime->called_class_name_override = effective_called_class;\n",
                );
                out.push_str("            *result_out = ");
                out.push_str(&user_function_c_name(method.function_index));
                out.push_str("(runtime, resolved_receiver, argc, args, line);\n");
                out.push_str("            runtime->called_class_name_override = ptn_previous_called_class;\n");
                out.push_str("            return 1;\n");
            }
            out.push_str("        }\n");
        }
        out.push_str("        return 0;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");
}

fn emit_dynamic_function_dispatch(out: &mut String) {
    out.push_str("\nstatic PTN_UNUSED char *ptn_dynamic_function_name(PtnValue callable) {\n");
    out.push_str("    return ptn_callable_function_name(callable);\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic int ptn_dynamic_call_mutates_first_array_argument(const char *name) {\n",
    );
    for name in [
        "array_pop",
        "array_push",
        "array_shift",
        "array_splice",
        "array_unshift",
        "array_walk",
        "array_walk_recursive",
        "arsort",
        "asort",
        "end",
        "krsort",
        "ksort",
        "natcasesort",
        "natsort",
        "next",
        "prev",
        "reset",
        "rsort",
        "shuffle",
        "sort",
        "uasort",
        "uksort",
        "usort",
    ] {
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(name);
        out.push_str("\")) {\n");
        out.push_str("        return 1;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic void ptn_dynamic_call_detach_first_reference_argument(const char *name, size_t argc, const PtnValue *args) {\n",
    );
    out.push_str(
        "    if (argc == 0 || args == NULL || !ptn_dynamic_call_mutates_first_array_argument(name)) {\n",
    );
    out.push_str("        return;\n");
    out.push_str("    }\n");
    out.push_str("    if (args[0].type != PTN_REFERENCE) {\n");
    out.push_str("        return;\n");
    out.push_str("    }\n");
    out.push_str("    PtnValue *value = &args[0].as.reference->value;\n");
    out.push_str("    if (value->type == PTN_ARRAY) {\n");
    out.push_str("        (void)ptn_value_detach_array(value);\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic void ptn_dynamic_call_warn_first_reference_argument_mismatch(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line) {\n",
    );
    out.push_str("    if (!runtime->warn_by_ref_argument_mismatch || argc == 0 || args == NULL || !ptn_dynamic_call_mutates_first_array_argument(name)) {\n");
    out.push_str("        return;\n");
    out.push_str("    }\n");
    out.push_str("    if (args[0].type == PTN_REFERENCE) {\n");
    out.push_str("        return;\n");
    out.push_str("    }\n");
    out.push_str(
        "    ptn_emit_by_reference_argument_warning(runtime, name, 1, \"array\", line);\n",
    );
    out.push_str("}\n");

    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_call_dynamic_function_name(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line) {\n",
    );
    out.push_str("    ptn_dynamic_call_warn_first_reference_argument_mismatch(runtime, name, argc, args, line);\n");
    out.push_str("    ptn_dynamic_call_detach_first_reference_argument(name, argc, args);\n");
    out.push_str("    return ptn_call_function(runtime, name, argc, args, line);\n");
    out.push_str("}\n");
}

fn emit_callable_dispatch(
    out: &mut String,
    functions: &[FunctionDecl],
    needs_method_dispatch: bool,
) {
    if needs_method_dispatch {
        out.push_str(
            "\nstatic PTN_UNUSED char *ptn_callable_resolve_class_scope(PtnRuntime *runtime, const char *scope_name, const char *relative_class_name) {\n",
        );
        out.push_str("    if (ptn_ascii_case_equal(scope_name, \"self\") || ptn_ascii_case_equal(scope_name, \"static\")) {\n");
        out.push_str("        const char *resolved = relative_class_name;\n");
        out.push_str("        if (resolved == NULL && runtime != NULL) {\n");
        out.push_str("            resolved = runtime->current_called_class_name != NULL ? runtime->current_called_class_name : runtime->current_class_name;\n");
        out.push_str("        }\n");
        out.push_str(
            "        return ptn_duplicate_string(resolved != NULL ? resolved : scope_name);\n",
        );
        out.push_str("    }\n");
        out.push_str("    if (ptn_ascii_case_equal(scope_name, \"parent\")) {\n");
        out.push_str("        const char *base = relative_class_name;\n");
        out.push_str("        if (base == NULL && runtime != NULL) {\n");
        out.push_str("            base = runtime->current_class_name;\n");
        out.push_str("        }\n");
        out.push_str("        const char *parent = base == NULL ? NULL : ptn_declared_class_parent_name(base);\n");
        out.push_str(
            "        return ptn_duplicate_string(parent != NULL ? parent : scope_name);\n",
        );
        out.push_str("    }\n");
        out.push_str("    return ptn_duplicate_string(scope_name);\n");
        out.push_str("}\n");

        out.push_str(
            "\nstatic PTN_UNUSED void ptn_emit_scoped_callable_deprecation(PtnRuntime *runtime, const char *scope_name, const char *method_name, size_t line) {\n",
        );
        out.push_str("    int needed = snprintf(NULL, 0, \"Callables of the form [\\\"%s\\\", \\\"%s\\\"] are deprecated\", scope_name, method_name);\n");
        out.push_str("    if (needed < 0) {\n");
        out.push_str("        ptn_abort_out_of_memory();\n");
        out.push_str("    }\n");
        out.push_str("    char *message = malloc((size_t)needed + 1);\n");
        out.push_str("    if (message == NULL) {\n");
        out.push_str("        ptn_abort_out_of_memory();\n");
        out.push_str("    }\n");
        out.push_str("    snprintf(message, (size_t)needed + 1, \"Callables of the form [\\\"%s\\\", \\\"%s\\\"] are deprecated\", scope_name, method_name);\n");
        out.push_str("    ptn_emit_deprecation(&runtime->diagnostics, message, line);\n");
        out.push_str("    free(message);\n");
        out.push_str("}\n");
    }

    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_call_callable(PtnRuntime *runtime, PtnValue callable, size_t argc, const PtnValue *args, size_t line) {\n",
    );
    out.push_str("    PtnValue resolved = ptn_value_deref(callable);\n");
    if needs_method_dispatch {
        out.push_str("    if (resolved.type == PTN_ARRAY && resolved.as.array->len == 2) {\n");
        out.push_str("        PtnArrayKey receiver_key = ptn_array_int_key(0);\n");
        out.push_str("        PtnArrayKey method_key = ptn_array_int_key(1);\n");
        out.push_str("        PtnArrayEntry *receiver_entry = ptn_array_entry_for_key(resolved.as.array, receiver_key);\n");
        out.push_str("        PtnArrayEntry *method_entry = ptn_array_entry_for_key(resolved.as.array, method_key);\n");
        out.push_str("        ptn_array_key_free(receiver_key);\n");
        out.push_str("        ptn_array_key_free(method_key);\n");
        out.push_str("        if (receiver_entry != NULL && method_entry != NULL) {\n");
        out.push_str("            PtnValue receiver = ptn_value_deref(receiver_entry->value);\n");
        out.push_str("            PtnValue method = ptn_value_deref(method_entry->value);\n");
        out.push_str(
            "            if (receiver.type == PTN_CLOSURE && method.type == PTN_STRING) {\n",
        );
        out.push_str("                char *method_name = ptn_value_to_string(method);\n");
        out.push_str("                if (ptn_ascii_case_equal(method_name, \"__invoke\")) {\n");
        out.push_str("                    const char *previous_name = runtime->by_ref_argument_function_name_override;\n");
        out.push_str("                    runtime->by_ref_argument_function_name_override = \"Closure::__invoke\";\n");
        out.push_str("                    PtnValue result = ptn_call_callable(runtime, receiver, argc, args, line);\n");
        out.push_str("                    runtime->by_ref_argument_function_name_override = previous_name;\n");
        out.push_str("                    free(method_name);\n");
        out.push_str("                    return result;\n");
        out.push_str("                }\n");
        out.push_str("                free(method_name);\n");
        out.push_str("            }\n");
        out.push_str("            if ((receiver.type == PTN_OBJECT || receiver.type == PTN_EXCEPTION) && method.type == PTN_STRING) {\n");
        out.push_str("                char *method_name = ptn_value_to_string(method);\n");
        out.push_str("                char *target_class_name = NULL;\n");
        out.push_str("                char *target_method_name = NULL;\n");
        out.push_str("                char *separator = strstr(method_name, \"::\");\n");
        out.push_str("                if (separator != NULL) {\n");
        out.push_str("                    const char *callable_scope_name = receiver.type == PTN_OBJECT ? receiver.as.object->class_name : receiver.as.exception->class_name;\n");
        out.push_str("                    ptn_emit_scoped_callable_deprecation(runtime, callable_scope_name, method_name, line);\n");
        out.push_str("                    *separator = '\\0';\n");
        out.push_str("                    const char *relative_class_name = receiver.type == PTN_OBJECT ? receiver.as.object->class_name : receiver.as.exception->class_name;\n");
        out.push_str("                    target_class_name = ptn_callable_resolve_class_scope(runtime, method_name, relative_class_name);\n");
        out.push_str(
            "                    target_method_name = ptn_duplicate_string(separator + 2);\n",
        );
        out.push_str("                }\n");
        out.push_str("                PtnValue result;\n");
        out.push_str("                if (target_class_name != NULL && ptn_call_declared_method_in_scope(runtime, receiver, target_class_name, target_method_name, receiver.type == PTN_OBJECT ? receiver.as.object->class_name : receiver.as.exception->class_name, argc, args, line, &result)) {\n");
        out.push_str("                    free(target_method_name);\n");
        out.push_str("                    free(target_class_name);\n");
        out.push_str("                    free(method_name);\n");
        out.push_str("                    return result;\n");
        out.push_str("                }\n");
        out.push_str("                result = ptn_call_declared_method(runtime, receiver, separator == NULL ? method_name : target_method_name, argc, args, line);\n");
        out.push_str("                free(target_method_name);\n");
        out.push_str("                free(target_class_name);\n");
        out.push_str("                free(method_name);\n");
        out.push_str("                return result;\n");
        out.push_str("            }\n");
        out.push_str(
            "            if (receiver.type == PTN_STRING && method.type == PTN_STRING) {\n",
        );
        out.push_str("                char *scope_name = ptn_value_to_string(receiver);\n");
        out.push_str("                char *method_name = ptn_value_to_string(method);\n");
        out.push_str("                char *target_class_name = NULL;\n");
        out.push_str("                char *target_method_name = NULL;\n");
        out.push_str("                char *separator = strstr(method_name, \"::\");\n");
        out.push_str("                if (separator != NULL) {\n");
        out.push_str("                    ptn_emit_scoped_callable_deprecation(runtime, scope_name, method_name, line);\n");
        out.push_str("                    *separator = '\\0';\n");
        out.push_str("                    target_class_name = ptn_callable_resolve_class_scope(runtime, method_name, scope_name);\n");
        out.push_str(
            "                    target_method_name = ptn_duplicate_string(separator + 2);\n",
        );
        out.push_str("                } else {\n");
        out.push_str("                    if (ptn_ascii_case_equal(scope_name, \"parent\")) {\n");
        out.push_str("                        ptn_emit_deprecation(&runtime->diagnostics, \"Use of \\\"parent\\\" in callables is deprecated\", line);\n");
        out.push_str("                    }\n");
        out.push_str("                    target_class_name = ptn_callable_resolve_class_scope(runtime, scope_name, NULL);\n");
        out.push_str(
            "                    target_method_name = ptn_duplicate_string(method_name);\n",
        );
        out.push_str("                }\n");
        out.push_str("                PtnValue scoped_receiver = ptn_null();\n");
        out.push_str("                const char *called_class_name = target_class_name;\n");
        out.push_str("                if (runtime->has_current_receiver) {\n");
        out.push_str("                    PtnValue current_receiver = ptn_value_deref(runtime->current_receiver);\n");
        out.push_str("                    if (current_receiver.type == PTN_OBJECT && ptn_declared_class_is_same_or_descendant(current_receiver.as.object->class_name, target_class_name)) {\n");
        out.push_str("                        scoped_receiver = current_receiver;\n");
        out.push_str(
            "                        called_class_name = current_receiver.as.object->class_name;\n",
        );
        out.push_str("                    }\n");
        out.push_str("                }\n");
        out.push_str("                PtnValue result;\n");
        out.push_str("                if (ptn_call_declared_method_in_scope(runtime, scoped_receiver, target_class_name, target_method_name, called_class_name, argc, args, line, &result)) {\n");
        out.push_str("                    free(target_method_name);\n");
        out.push_str("                    free(target_class_name);\n");
        out.push_str("                    free(method_name);\n");
        out.push_str("                    free(scope_name);\n");
        out.push_str("                    return result;\n");
        out.push_str("                }\n");
        out.push_str("                free(target_method_name);\n");
        out.push_str("                free(target_class_name);\n");
        out.push_str("                free(method_name);\n");
        out.push_str("                free(scope_name);\n");
        out.push_str("            }\n");
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("    if (resolved.type == PTN_OBJECT && ptn_declared_class_has_invoke_magic(resolved.as.object->class_name)) {\n");
        out.push_str("        return ptn_call_declared_method(runtime, resolved, \"__invoke\", argc, args, line);\n");
        out.push_str("    }\n");
    }
    out.push_str("    if (resolved.type == PTN_CLOSURE) {\n");
    out.push_str("        if (resolved.as.closure->has_wrapped_callable) {\n");
    out.push_str("            return ptn_call_callable(runtime, resolved.as.closure->wrapped_callable, argc, args, line);\n");
    out.push_str("        }\n");
    out.push_str("        switch (resolved.as.closure->function_index) {\n");
    for (index, function) in functions.iter().enumerate() {
        if !function.is_anonymous {
            continue;
        }
        out.push_str("            case ");
        out.push_str(&index.to_string());
        out.push_str(": return ");
        out.push_str(&user_function_c_name(index));
        out.push_str("(runtime, resolved, argc, args, line);\n");
    }
    out.push_str("            default:\n");
    out.push_str("                fputs(\"Fatal error: invalid closure\\n\", stderr);\n");
    out.push_str("                exit(255);\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("    char *name = ptn_dynamic_function_name(resolved);\n");
    out.push_str(
        "    PtnValue result = ptn_call_dynamic_function_name(runtime, name, argc, args, line);\n",
    );
    out.push_str("    free(name);\n");
    out.push_str("    return result;\n");
    out.push_str("}\n");
}

fn emit_instruction(
    out: &mut String,
    values: &mut ValueEmitter,
    instruction: &Instruction,
    control_targets: &mut Vec<ControlTarget>,
    source_path: &str,
    return_target: Option<&str>,
) {
    match instruction {
        Instruction::Store { name, value } => {
            let emitted_value = values.emit_materialized_value(out, value);
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&emitted_value);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &emitted_value);
        }
        Instruction::StoreRef { name, source, line } => {
            values.emit_store_reference_source_to_variable(out, name, source, *line);
        }
        Instruction::StoreArrayDim {
            array,
            dimensions,
            value,
            compound_op,
            line,
        } => {
            if compound_op.is_some() {
                out.push_str("    ptn_runtime_array_warn_missing_base_for_assign_op(&runtime, \"");
                out.push_str(&c_string(array));
                out.push_str("\", \"");
                out.push_str(&c_string(source_path));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            }
            let path = emit_array_path_segments(out, values, dimensions);
            let value_temp = values.emit_materialized_value(out, value);
            let stored_temp = if let Some(op) = compound_op {
                let current_temp = values.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_array_path_read_for_assign_op(&runtime, \"");
                out.push_str(&c_string(array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let result_temp = values.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                if matches!(op, BinaryOp::Concat) {
                    out.push_str("ptn_concat(&runtime, ");
                    out.push_str(&current_temp);
                    out.push_str(", ");
                    out.push_str(&value_temp);
                    out.push_str(", ");
                    out.push_str(&line.to_string());
                    out.push_str(")");
                } else {
                    out.push_str(binary_runtime_function(*op));
                    out.push('(');
                    if binary_runtime_function_uses_context(*op) {
                        out.push_str("&runtime, ");
                    }
                    out.push_str(&current_temp);
                    out.push_str(", ");
                    out.push_str(&value_temp);
                    if binary_runtime_function_uses_context(*op) {
                        out.push_str(", ");
                        out.push_str(&line.to_string());
                    }
                    out.push(')');
                }
                out.push_str(";\n");
                emit_value_cleanup(out, "    ", &current_temp);
                result_temp
            } else {
                let snapshot_temp = values.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&snapshot_temp);
                out.push_str(" = ptn_value_snapshot_for_array_path_write(");
                out.push_str(&value_temp);
                out.push_str(");\n");
                snapshot_temp
            };
            out.push_str("    ");
            out.push_str(if compound_op.is_some() {
                "ptn_runtime_array_path_set_from_assign_op"
            } else {
                "ptn_runtime_array_path_set"
            });
            out.push_str("(&runtime, \"");
            out.push_str(&c_string(array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&stored_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &stored_temp);
            emit_value_cleanup(out, "    ", &value_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        Instruction::StoreArrayDimRef { target, source } => {
            values.emit_store_reference_source_to_array_dim(out, target, source, source_path);
        }
        Instruction::DefineConstant { name, value, line } => {
            let emitted_value = values.emit_const_materialized_value(out, value);
            out.push_str("    (void)ptn_runtime_define_constant_if_absent(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&emitted_value);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &emitted_value);
        }
        Instruction::Expression(value) => {
            let emitted_value = values.emit_materialized_value(out, value);
            out.push_str("    (void)");
            out.push_str(&emitted_value);
            out.push_str(";\n");
            emit_value_cleanup(out, "    ", &emitted_value);
        }
        Instruction::Echo(value) => {
            let emitted_value = values.emit_materialized_value(out, value);
            out.push_str("    ptn_echo(&runtime, ");
            out.push_str(&emitted_value);
            out.push_str(", 0);\n");
            emit_value_cleanup(out, "    ", &emitted_value);
        }
        Instruction::Increment { target, op, line } => {
            emit_increment_statement(out, values, target, *op, *line, source_path);
        }
        Instruction::UnsetVariable { name } => {
            out.push_str("    ptn_runtime_unset_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\");\n");
        }
        Instruction::UnsetDynamicVariable { name, line } => {
            let name_temp = values.emit_dynamic_variable_name(out, name, *line);
            out.push_str("    ptn_runtime_unset_variable(&runtime, ");
            out.push_str(&name_temp);
            out.push_str(");\n");
            out.push_str("    free(");
            out.push_str(&name_temp);
            out.push_str(");\n");
        }
        Instruction::BindGlobal { name } => {
            out.push_str("    ptn_runtime_bind_global_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\");\n");
        }
        Instruction::DeclareFunction { function_index } => {
            out.push_str("    if (runtime.declared_user_functions != NULL) {\n");
            out.push_str("        runtime.declared_user_functions[");
            out.push_str(&function_index.to_string());
            out.push_str("] = 1;\n");
            out.push_str("    }\n");
        }
        Instruction::UnsetArrayDim {
            array,
            dimensions,
            line,
        } => {
            let path = emit_array_unset_path_segments(out, values, dimensions);
            out.push_str("    ptn_runtime_array_path_unset(&runtime, \"");
            out.push_str(&c_string(array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        Instruction::UnsetDynamicArrayDim {
            name,
            dimensions,
            line,
        } => {
            let name_temp = values.emit_dynamic_variable_name(out, name, *line);
            let path = emit_array_unset_path_segments(out, values, dimensions);
            out.push_str("    ptn_runtime_array_path_unset(&runtime, ");
            out.push_str(&name_temp);
            out.push_str(", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    free(");
            out.push_str(&name_temp);
            out.push_str(");\n");
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        Instruction::UnsetPropertyArrayDim {
            receiver,
            name,
            dimensions,
            line,
        } => {
            let receiver_temp = values.emit_materialized_value(out, receiver);
            let path = emit_array_unset_path_segments(out, values, dimensions);
            let current_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&current_temp);
            out.push_str(" = ptn_object_read_property(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&c_optional_string(values.current_class_name.as_deref()));
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_value_array_path_unset(&runtime, &");
            out.push_str(&current_temp);
            out.push_str(", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            let assigned_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&assigned_temp);
            out.push_str(" = ptn_object_write_property_indirect(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&c_optional_string(values.current_class_name.as_deref()));
            out.push_str(", ");
            out.push_str(&current_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &assigned_temp);
            emit_value_cleanup(out, "    ", &current_temp);
            emit_value_cleanup(out, "    ", &receiver_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        Instruction::UnsetProperty {
            receiver,
            name,
            line,
        } => {
            let receiver_temp = values.emit_materialized_value(out, receiver);
            out.push_str("    ptn_object_unset_property(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&c_optional_string(values.current_class_name.as_deref()));
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &receiver_temp);
        }
        Instruction::InternalCall {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            line,
        } => {
            let result_temp = values.emit_internal_call(
                out,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            );
            out.push_str("    (void)");
            out.push_str(&result_temp);
            out.push_str(";\n");
            emit_value_cleanup(out, "    ", &result_temp);
        }
        Instruction::Return { value, .. } => match return_target {
            Some(target) => {
                if let Some(value) = value {
                    if values.current_function_return_by_ref {
                        let reference_value = values.emit_reference_source(out, value);
                        out.push_str("    ptn_return_value = ptn_value_share(");
                        out.push_str(&reference_value);
                        out.push_str(");\n");
                        emit_value_cleanup(out, "    ", &reference_value);
                    } else {
                        let result_value = values.emit_materialized_value(out, value);
                        out.push_str("    ptn_return_value = ptn_value_clone(ptn_value_deref(");
                        out.push_str(&result_value);
                        out.push_str("));\n");
                        emit_value_cleanup(out, "    ", &result_value);
                    }
                } else {
                    out.push_str("    ptn_return_value = ptn_null();\n");
                }
                out.push_str("    goto ");
                out.push_str(target);
                out.push_str(";\n");
            }
            None => {
                if let Some(value) = value {
                    let return_temp = values.emit_materialized_value(out, value);
                    out.push_str("    (void)");
                    out.push_str(&return_temp);
                    out.push_str(";\n");
                    emit_value_cleanup(out, "    ", &return_temp);
                }
                out.push_str("    ptn_runtime_free(&runtime);\n");
                out.push_str("    return 0;\n");
            }
        },
        Instruction::Throw { value, line } => {
            let value_temp = values.emit_materialized_value(out, value);
            out.push_str("    ptn_throw_value(&runtime, ");
            out.push_str(&value_temp);
            out.push_str(", \"");
            out.push_str(&c_string(source_path));
            out.push_str("\", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
        }
        Instruction::Try { body, catches } => {
            emit_try(
                out,
                values,
                body,
                catches,
                control_targets,
                source_path,
                return_target,
            );
        }
        Instruction::Branch {
            condition,
            then_body,
            else_body,
        } => {
            let condition_predicate = values.emit_condition(out, condition);
            out.push_str("    if (");
            out.push_str(&condition_predicate);
            out.push_str(") {\n");
            for body_instruction in then_body {
                emit_instruction(
                    out,
                    values,
                    body_instruction,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            if !else_body.is_empty() {
                out.push_str("    } else {\n");
                for body_instruction in else_body {
                    emit_instruction(
                        out,
                        values,
                        body_instruction,
                        control_targets,
                        source_path,
                        return_target,
                    );
                }
            }
            out.push_str("    }\n");
        }
        Instruction::While { condition, body } => {
            let end_label = values.next_label("ptn_loop_end");
            let continue_label = values.next_label("ptn_loop_continue");
            emit_label_reference(out, &end_label);
            out.push_str("    while (1) {\n");
            emit_label_reference(out, &continue_label);
            out.push_str("    ");
            out.push_str(&continue_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
            let condition_predicate = values.emit_condition(out, condition);
            out.push_str("        if (!(");
            out.push_str(&condition_predicate);
            out.push_str(")) {\n");
            out.push_str("            goto ");
            out.push_str(&end_label);
            out.push_str(";\n");
            out.push_str("        }\n");
            control_targets.push(ControlTarget::loop_target(
                end_label.clone(),
                continue_label,
            ));
            for body_instruction in body {
                emit_instruction(
                    out,
                    values,
                    body_instruction,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            control_targets.pop();
            out.push_str("    }\n");
            out.push_str("    ");
            out.push_str(&end_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
        }
        Instruction::DoWhile { body, condition } => {
            let end_label = values.next_label("ptn_loop_end");
            let continue_label = values.next_label("ptn_loop_continue");
            emit_label_reference(out, &end_label);
            out.push_str("    while (1) {\n");
            control_targets.push(ControlTarget::loop_target(
                end_label.clone(),
                continue_label.clone(),
            ));
            for body_instruction in body {
                emit_instruction(
                    out,
                    values,
                    body_instruction,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            control_targets.pop();
            emit_label_reference(out, &continue_label);
            out.push_str("    ");
            out.push_str(&continue_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
            let condition_predicate = values.emit_condition(out, condition);
            out.push_str("        if (!(");
            out.push_str(&condition_predicate);
            out.push_str(")) {\n");
            out.push_str("            goto ");
            out.push_str(&end_label);
            out.push_str(";\n");
            out.push_str("        }\n");
            out.push_str("    }\n");
            out.push_str("    ");
            out.push_str(&end_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
        }
        Instruction::For {
            initializers,
            condition,
            updates,
            body,
        } => {
            for initializer in initializers {
                emit_instruction(
                    out,
                    values,
                    initializer,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            let end_label = values.next_label("ptn_loop_end");
            let continue_label = values.next_label("ptn_loop_continue");
            emit_label_reference(out, &end_label);
            out.push_str("    while (1) {\n");
            if let Some(condition) = condition {
                let condition_predicate = values.emit_condition(out, condition);
                out.push_str("        if (!(");
                out.push_str(&condition_predicate);
                out.push_str(")) {\n");
                out.push_str("            goto ");
                out.push_str(&end_label);
                out.push_str(";\n");
                out.push_str("        }\n");
            }
            control_targets.push(ControlTarget::loop_target(
                end_label.clone(),
                continue_label.clone(),
            ));
            for body_instruction in body {
                emit_instruction(
                    out,
                    values,
                    body_instruction,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            control_targets.pop();
            emit_label_reference(out, &continue_label);
            out.push_str("    ");
            out.push_str(&continue_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
            for update in updates {
                emit_instruction(
                    out,
                    values,
                    update,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            out.push_str("    }\n");
            out.push_str("    ");
            out.push_str(&end_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
        }
        Instruction::Foreach {
            iterable,
            key,
            value,
            value_by_ref,
            body,
            line,
        } => {
            let end_label = values.next_label("ptn_foreach_end");
            let cleanup_label = values.next_label("ptn_foreach_cleanup");
            let continue_label = values.next_label("ptn_foreach_continue");
            let iterator_temp = values.next_temp();
            out.push_str("    PtnArrayIterator ");
            out.push_str(&iterator_temp);
            out.push_str(";\n");
            let iterable_temp = if *value_by_ref {
                match iterable {
                    ValueExpr::Load { name, .. } => {
                        out.push_str("    ");
                        out.push_str(&iterator_temp);
                        out.push_str(" = ptn_array_iterator_by_ref_from_variable(&runtime, \"");
                        out.push_str(&c_string(name));
                        out.push_str("\", ");
                        out.push_str(&c_optional_string(values.current_class_name.as_deref()));
                        out.push_str(", \"");
                        out.push_str(&c_string(source_path));
                        out.push_str("\", ");
                        out.push_str(&line.to_string());
                        out.push_str(");\n");
                        None
                    }
                    _ => {
                        if let Some(target) = reference_target_from_value(iterable) {
                            let reference_temp = values.emit_reference_target(out, &target);
                            out.push_str("    ");
                            out.push_str(&iterator_temp);
                            out.push_str(" = ptn_array_iterator_by_ref_from_reference(&runtime, ");
                            out.push_str(&reference_temp);
                            out.push_str(", ");
                            out.push_str(&c_optional_string(values.current_class_name.as_deref()));
                            out.push_str(", \"");
                            out.push_str(&c_string(source_path));
                            out.push_str("\", ");
                            out.push_str(&line.to_string());
                            out.push_str(");\n");
                            emit_value_cleanup(out, "    ", &reference_temp);
                            None
                        } else {
                            let iterable_temp = values.emit_materialized_value(out, iterable);
                            out.push_str("    ");
                            out.push_str(&iterator_temp);
                            out.push_str(" = ptn_array_iterator_by_ref_from_value(&runtime, &");
                            out.push_str(&iterable_temp);
                            out.push_str(", ");
                            out.push_str(&c_optional_string(values.current_class_name.as_deref()));
                            out.push_str(", \"");
                            out.push_str(&c_string(source_path));
                            out.push_str("\", ");
                            out.push_str(&line.to_string());
                            out.push_str(");\n");
                            Some(iterable_temp)
                        }
                    }
                }
            } else {
                let iterable_temp = values.emit_materialized_value(out, iterable);
                out.push_str("    ");
                out.push_str(&iterator_temp);
                out.push_str(" = ptn_array_iterator_from_value(&runtime, ");
                out.push_str(&iterable_temp);
                out.push_str(", ");
                out.push_str(&c_optional_string(values.current_class_name.as_deref()));
                out.push_str(", \"");
                out.push_str(&c_string(source_path));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                Some(iterable_temp)
            };
            emit_label_reference(out, &cleanup_label);
            emit_label_reference(out, &end_label);
            out.push_str("    while (");
            out.push_str(&iterator_temp);
            out.push_str(".valid) {\n");
            if let Some(key) = key {
                let key_temp = values.next_temp();
                out.push_str("        PtnValue ");
                out.push_str(&key_temp);
                out.push_str(" = ptn_array_iterator_current_key(&");
                out.push_str(&iterator_temp);
                out.push_str(");\n");
                let key_result_temp =
                    values.emit_store_assignment_target_from_temp(out, key, &key_temp);
                emit_value_cleanup(out, "        ", &key_result_temp);
                emit_value_cleanup(out, "        ", &key_temp);
            }
            let value_temp = values.next_temp();
            let value_needs_reference = *value_by_ref
                || matches!(value, AssignmentTarget::List(target) if list_assignment_has_reference(target));
            out.push_str("        PtnValue ");
            out.push_str(&value_temp);
            out.push_str(" = ");
            if value_needs_reference {
                out.push_str("ptn_array_iterator_current_reference(&");
            } else {
                out.push_str("ptn_array_iterator_current_value(&");
            }
            out.push_str(&iterator_temp);
            out.push_str(");\n");
            if *value_by_ref {
                values.emit_bind_assignment_target_reference(out, value, &value_temp);
            } else {
                let value_result_temp =
                    values.emit_store_assignment_target_from_temp(out, value, &value_temp);
                emit_value_cleanup(out, "        ", &value_result_temp);
            }
            emit_value_cleanup(out, "        ", &value_temp);
            control_targets.push(ControlTarget::loop_target(
                cleanup_label.clone(),
                continue_label.clone(),
            ));
            for body_instruction in body {
                emit_instruction(
                    out,
                    values,
                    body_instruction,
                    control_targets,
                    source_path,
                    return_target,
                );
            }
            control_targets.pop();
            emit_label_reference(out, &continue_label);
            out.push_str("    ");
            out.push_str(&continue_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
            out.push_str("        ptn_array_iterator_advance(&");
            out.push_str(&iterator_temp);
            out.push_str(");\n");
            out.push_str("    }\n");
            out.push_str("    ");
            out.push_str(&cleanup_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
            out.push_str("    ptn_array_iterator_destroy(&");
            out.push_str(&iterator_temp);
            out.push_str(");\n");
            if let Some(iterable_temp) = iterable_temp {
                emit_value_cleanup(out, "    ", &iterable_temp);
            }
            out.push_str("    ");
            out.push_str(&end_label);
            out.push_str(":\n");
            out.push_str("    ;\n");
        }
        Instruction::Switch { expression, cases } => {
            emit_switch(
                out,
                values,
                expression,
                cases,
                control_targets,
                source_path,
                return_target,
            );
        }
        Instruction::Label { name } => {
            out.push_str("    ");
            out.push_str(&c_label(name));
            out.push_str(":\n");
            out.push_str("    ;\n");
        }
        Instruction::Goto { label } => {
            out.push_str("    goto ");
            out.push_str(&c_label(label));
            out.push_str(";\n");
        }
        Instruction::Break { level, line } => {
            if *level > 0 && *level <= control_targets.len() {
                let target = &control_targets[control_targets.len() - *level].break_label;
                out.push_str("    goto ");
                out.push_str(target);
                out.push_str(";\n");
            } else {
                let suffix = if *level == 1 { "level" } else { "levels" };
                out.push_str("    ptn_abort_control_error(\"Cannot 'break' ");
                out.push_str(&level.to_string());
                out.push(' ');
                out.push_str(suffix);
                out.push_str("\", \"");
                out.push_str(&c_string(source_path));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            }
        }
        Instruction::Continue { level, line } => {
            if *level > 0 && *level <= control_targets.len() {
                let target = &control_targets[control_targets.len() - *level].continue_label;
                out.push_str("    goto ");
                out.push_str(target);
                out.push_str(";\n");
            } else {
                let suffix = if *level == 1 { "level" } else { "levels" };
                out.push_str("    ptn_abort_control_error(\"Cannot 'continue' ");
                out.push_str(&level.to_string());
                out.push(' ');
                out.push_str(suffix);
                out.push_str("\", \"");
                out.push_str(&c_string(source_path));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            }
        }
    }
}

fn emit_try(
    out: &mut String,
    values: &mut ValueEmitter,
    body: &[Instruction],
    catches: &[IrCatchClause],
    control_targets: &mut Vec<ControlTarget>,
    source_path: &str,
    return_target: Option<&str>,
) {
    let frame_temp = values.next_temp();
    let caught_temp = values.next_temp();
    let saved_trace_temp = values.next_temp();
    let end_label = values.next_label("ptn_try_end");
    out.push_str("    {\n");
    out.push_str("        PtnTryFrame ");
    out.push_str(&frame_temp);
    out.push_str(";\n");
    out.push_str("        PtnTraceFrame *");
    out.push_str(&saved_trace_temp);
    out.push_str(" = runtime.trace_frame;\n");
    out.push_str("        ptn_try_frame_push(&runtime, &");
    out.push_str(&frame_temp);
    out.push_str(");\n");
    out.push_str("        if (setjmp(");
    out.push_str(&frame_temp);
    out.push_str(".jump) == 0) {\n");
    for body_instruction in body {
        emit_instruction(
            out,
            values,
            body_instruction,
            control_targets,
            source_path,
            return_target,
        );
    }
    out.push_str("            ptn_try_frame_pop(&runtime, &");
    out.push_str(&frame_temp);
    out.push_str(");\n");
    out.push_str("        } else {\n");
    out.push_str("            ptn_try_frame_pop(&runtime, &");
    out.push_str(&frame_temp);
    out.push_str(");\n");
    out.push_str("            runtime.trace_frame = ");
    out.push_str(&saved_trace_temp);
    out.push_str(";\n");
    out.push_str("            runtime.warn_by_ref_argument_mismatch = 0;\n");
    out.push_str("            runtime.throw_argument_count_errors = 0;\n");
    out.push_str("            int ");
    out.push_str(&caught_temp);
    out.push_str(" = 0;\n");
    for catch in catches {
        out.push_str("            if (!");
        out.push_str(&caught_temp);
        out.push_str(" && ptn_exception_matches(&runtime, \"");
        out.push_str(&c_string(&catch.type_name));
        out.push_str("\")) {\n");
        out.push_str("                ");
        out.push_str(&caught_temp);
        out.push_str(" = 1;\n");
        if let Some(variable) = &catch.variable {
            out.push_str("                ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(variable));
            out.push_str("\", ptn_current_exception_value(&runtime));\n");
        }
        out.push_str("                ptn_clear_exception(&runtime);\n");
        for body_instruction in &catch.body {
            emit_instruction(
                out,
                values,
                body_instruction,
                control_targets,
                source_path,
                return_target,
            );
        }
        out.push_str("            }\n");
    }
    out.push_str("            if (!");
    out.push_str(&caught_temp);
    out.push_str(") {\n");
    out.push_str("                ptn_rethrow_exception(&runtime);\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        goto ");
    out.push_str(&end_label);
    out.push_str(";\n");
    out.push_str("    }\n");
    out.push_str("    ");
    out.push_str(&end_label);
    out.push_str(":\n");
    out.push_str("    ;\n");
}

fn user_function_c_name(index: usize) -> String {
    format!("ptn_user_function_{index}")
}

fn include_c_name(index: usize) -> String {
    format!("ptn_include_file_{index}")
}

fn include_kind_text(kind: IncludeKind) -> &'static str {
    match kind {
        IncludeKind::Include => "include",
        IncludeKind::IncludeOnce => "include_once",
        IncludeKind::Require => "require",
        IncludeKind::RequireOnce => "require_once",
    }
}

fn include_kind_is_required(kind: IncludeKind) -> bool {
    matches!(kind, IncludeKind::Require | IncludeKind::RequireOnce)
}

fn include_kind_is_once(kind: IncludeKind) -> bool {
    matches!(kind, IncludeKind::IncludeOnce | IncludeKind::RequireOnce)
}

struct ControlTarget {
    break_label: String,
    continue_label: String,
}

impl ControlTarget {
    fn loop_target(break_label: String, continue_label: String) -> Self {
        Self {
            break_label,
            continue_label,
        }
    }

    fn switch_target(end_label: String) -> Self {
        Self {
            break_label: end_label.clone(),
            continue_label: end_label,
        }
    }
}

struct EmittedArrayPath {
    name: String,
    len: usize,
    value_temps: Vec<String>,
}

fn emit_array_path_segments(
    out: &mut String,
    values: &mut ValueEmitter,
    dimensions: &[Option<ValueExpr>],
) -> EmittedArrayPath {
    let mut value_temps = Vec::new();
    let mut initializers = Vec::new();
    for dimension in dimensions {
        if let Some(dimension) = dimension {
            let temp = values.emit_materialized_value(out, dimension);
            initializers.push(format!("{{ 0, {temp} }}"));
            value_temps.push(temp);
        } else {
            initializers.push("{ 1, ptn_null() }".to_string());
        }
    }
    emit_array_path(out, values, dimensions.len(), initializers, value_temps)
}

fn emit_array_unset_path_segments(
    out: &mut String,
    values: &mut ValueEmitter,
    dimensions: &[ValueExpr],
) -> EmittedArrayPath {
    let mut value_temps = Vec::new();
    let mut initializers = Vec::new();
    for dimension in dimensions {
        let temp = values.emit_materialized_value(out, dimension);
        initializers.push(format!("{{ 0, {temp} }}"));
        value_temps.push(temp);
    }
    emit_array_path(out, values, dimensions.len(), initializers, value_temps)
}

fn emit_array_path(
    out: &mut String,
    values: &mut ValueEmitter,
    len: usize,
    mut initializers: Vec<String>,
    value_temps: Vec<String>,
) -> EmittedArrayPath {
    if initializers.is_empty() {
        initializers.push("{ 1, ptn_null() }".to_string());
    }
    let name = values.next_temp();
    out.push_str("    PtnArrayPathSegment ");
    out.push_str(&name);
    out.push_str("[] = { ");
    out.push_str(&initializers.join(", "));
    out.push_str(" };\n");
    EmittedArrayPath {
        name,
        len,
        value_temps,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlTargetKind {
    Loop,
    Switch,
}

struct ControlWarning {
    message: String,
    line: usize,
}

fn collect_control_warnings(instructions: &[Instruction]) -> Vec<ControlWarning> {
    let mut warnings = Vec::new();
    collect_control_warnings_in(instructions, &mut Vec::new(), &mut warnings);
    warnings
}

fn collect_module_control_warnings(module: &Module) -> Vec<ControlWarning> {
    let mut warnings = Vec::new();
    for function in &module.functions {
        warnings.extend(collect_control_warnings(&function.body));
    }
    warnings.extend(collect_control_warnings(&module.instructions));
    warnings
}

fn collect_module_legacy_dollar_brace_deprecations(
    module: &Module,
) -> Vec<LegacyDollarBraceDeprecation> {
    let mut deprecations = Vec::new();
    for class in &module.classes {
        for property in &class.properties {
            if let Some(value) = &property.value {
                collect_value_legacy_dollar_brace_deprecations(value, &mut deprecations);
            }
        }
        for property in &class.static_properties {
            if let Some(value) = &property.value {
                collect_value_legacy_dollar_brace_deprecations(value, &mut deprecations);
            }
        }
        for constant in &class.constants {
            collect_value_legacy_dollar_brace_deprecations(&constant.value, &mut deprecations);
        }
    }
    for function in &module.functions {
        for parameter in &function.parameters {
            if let Some(default_value) = &parameter.default_value {
                collect_value_legacy_dollar_brace_deprecations(default_value, &mut deprecations);
            }
        }
        collect_instructions_legacy_dollar_brace_deprecations(&function.body, &mut deprecations);
    }
    collect_instructions_legacy_dollar_brace_deprecations(&module.instructions, &mut deprecations);
    deprecations
}

fn collect_module_magic_visibility_warnings(module: &Module) -> Vec<MagicVisibilityWarning> {
    let mut warnings = Vec::new();
    for class in &module.classes {
        for method in &class.methods {
            if method.visibility == PropertyVisibility::Public
                || !magic_method_requires_public_visibility(&method.name)
            {
                continue;
            }
            warnings.push(MagicVisibilityWarning {
                class_name: class.name.clone(),
                method_name: method.name.clone(),
                line: method.line,
            });
        }
    }
    warnings
}

fn magic_method_requires_public_visibility(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "__call"
            | "__callstatic"
            | "__get"
            | "__set"
            | "__isset"
            | "__unset"
            | "__sleep"
            | "__wakeup"
            | "__serialize"
            | "__unserialize"
            | "__tostring"
            | "__set_state"
            | "__debuginfo"
    )
}

fn collect_include_legacy_dollar_brace_deprecations(
    include: &IncludeFile,
) -> Vec<LegacyDollarBraceDeprecation> {
    let mut deprecations = Vec::new();
    collect_instructions_legacy_dollar_brace_deprecations(&include.instructions, &mut deprecations);
    deprecations
}

fn collect_instructions_legacy_dollar_brace_deprecations(
    instructions: &[Instruction],
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    for instruction in instructions {
        collect_instruction_legacy_dollar_brace_deprecations(instruction, deprecations);
    }
}

fn collect_instruction_legacy_dollar_brace_deprecations(
    instruction: &Instruction,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    match instruction {
        Instruction::Store { value, .. }
        | Instruction::DefineConstant { value, .. }
        | Instruction::Expression(value)
        | Instruction::Echo(value) => {
            collect_value_legacy_dollar_brace_deprecations(value, deprecations);
        }
        Instruction::StoreRef { source, .. } => {
            collect_value_legacy_dollar_brace_deprecations(source, deprecations);
        }
        Instruction::StoreArrayDim {
            dimensions, value, ..
        } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
                }
            }
            collect_value_legacy_dollar_brace_deprecations(value, deprecations);
        }
        Instruction::StoreArrayDimRef { target, source } => {
            collect_array_dim_target_legacy_dollar_brace_deprecations(target, deprecations);
            collect_value_legacy_dollar_brace_deprecations(source, deprecations);
        }
        Instruction::UnsetArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
            }
        }
        Instruction::UnsetDynamicVariable { name, .. } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
        }
        Instruction::UnsetDynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
            for dimension in dimensions {
                collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
            }
        }
        Instruction::UnsetPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
            for dimension in dimensions {
                collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
            }
        }
        Instruction::UnsetProperty { receiver, .. } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
        }
        Instruction::InternalCall { arguments, .. } => {
            for argument in arguments {
                collect_value_legacy_dollar_brace_deprecations(argument, deprecations);
            }
        }
        Instruction::Return { value, .. } => {
            if let Some(value) = value {
                collect_value_legacy_dollar_brace_deprecations(value, deprecations);
            }
        }
        Instruction::Throw { value, .. } => {
            collect_value_legacy_dollar_brace_deprecations(value, deprecations);
        }
        Instruction::Try { body, catches } => {
            collect_instructions_legacy_dollar_brace_deprecations(body, deprecations);
            for catch in catches {
                collect_instructions_legacy_dollar_brace_deprecations(&catch.body, deprecations);
            }
        }
        Instruction::Branch {
            condition,
            then_body,
            else_body,
        } => {
            collect_value_legacy_dollar_brace_deprecations(condition, deprecations);
            collect_instructions_legacy_dollar_brace_deprecations(then_body, deprecations);
            collect_instructions_legacy_dollar_brace_deprecations(else_body, deprecations);
        }
        Instruction::While { condition, body } | Instruction::DoWhile { body, condition } => {
            collect_value_legacy_dollar_brace_deprecations(condition, deprecations);
            collect_instructions_legacy_dollar_brace_deprecations(body, deprecations);
        }
        Instruction::For {
            initializers,
            condition,
            updates,
            body,
        } => {
            collect_instructions_legacy_dollar_brace_deprecations(initializers, deprecations);
            if let Some(condition) = condition {
                collect_value_legacy_dollar_brace_deprecations(condition, deprecations);
            }
            collect_instructions_legacy_dollar_brace_deprecations(updates, deprecations);
            collect_instructions_legacy_dollar_brace_deprecations(body, deprecations);
        }
        Instruction::Foreach {
            iterable,
            key,
            value,
            body,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(iterable, deprecations);
            if let Some(key) = key {
                collect_assignment_target_legacy_dollar_brace_deprecations(key, deprecations);
            }
            collect_assignment_target_legacy_dollar_brace_deprecations(value, deprecations);
            collect_instructions_legacy_dollar_brace_deprecations(body, deprecations);
        }
        Instruction::Switch { expression, cases } => {
            collect_value_legacy_dollar_brace_deprecations(expression, deprecations);
            for case in cases {
                if let Some(condition) = &case.condition {
                    collect_value_legacy_dollar_brace_deprecations(condition, deprecations);
                }
                collect_instructions_legacy_dollar_brace_deprecations(&case.body, deprecations);
            }
        }
        Instruction::Increment { target, .. } => {
            collect_inc_dec_target_legacy_dollar_brace_deprecations(target, deprecations);
        }
        Instruction::UnsetVariable { .. }
        | Instruction::BindGlobal { .. }
        | Instruction::DeclareFunction { .. }
        | Instruction::Break { .. }
        | Instruction::Continue { .. }
        | Instruction::Label { .. }
        | Instruction::Goto { .. } => {}
    }
}

fn collect_array_dim_target_legacy_dollar_brace_deprecations(
    target: &crate::ir::ArrayDimTarget,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    for dimension in &target.dimensions {
        if let Some(dimension) = dimension {
            collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
        }
    }
}

fn collect_inc_dec_target_legacy_dollar_brace_deprecations(
    target: &IncDecTarget,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    match target {
        IncDecTarget::Variable { .. } => {}
        IncDecTarget::DynamicVariable { name, .. } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
        }
        IncDecTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
                }
            }
        }
        IncDecTarget::ArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
                }
            }
        }
        IncDecTarget::Property { receiver, .. } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
        }
        IncDecTarget::StaticProperty { .. } => {}
    }
}

fn collect_reference_target_legacy_dollar_brace_deprecations(
    target: &ReferenceTarget,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    match target {
        ReferenceTarget::Variable { .. } => {}
        ReferenceTarget::ArrayDim(target) => {
            collect_array_dim_target_legacy_dollar_brace_deprecations(target, deprecations);
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
            for dimension in dimensions.iter().flatten() {
                collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
            }
        }
        ReferenceTarget::Property { receiver, .. } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
        }
    }
}

fn collect_assignment_target_legacy_dollar_brace_deprecations(
    target: &AssignmentTarget,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    match target {
        AssignmentTarget::Variable { .. } | AssignmentTarget::StaticProperty { .. } => {}
        AssignmentTarget::DynamicVariable { name, .. } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
        }
        AssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
                }
            }
        }
        AssignmentTarget::ArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
                }
            }
        }
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_legacy_dollar_brace_deprecations(dimension, deprecations);
                }
            }
        }
        AssignmentTarget::Property { receiver, .. } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
        }
        AssignmentTarget::List(target) => {
            collect_list_assignment_legacy_dollar_brace_deprecations(target, deprecations);
        }
    }
}

fn collect_list_assignment_legacy_dollar_brace_deprecations(
    target: &ListAssignmentTarget,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    for element in &target.elements {
        if let Some(key) = &element.key {
            collect_value_legacy_dollar_brace_deprecations(key, deprecations);
        }
        match &element.target {
            ListAssignmentElementTarget::Value(target) => {
                collect_assignment_target_legacy_dollar_brace_deprecations(target, deprecations);
            }
            ListAssignmentElementTarget::Reference(target) => {
                collect_reference_target_legacy_dollar_brace_deprecations(target, deprecations);
            }
        }
    }
}

fn collect_value_legacy_dollar_brace_deprecations(
    value: &ValueExpr,
    deprecations: &mut Vec<LegacyDollarBraceDeprecation>,
) {
    match value {
        ValueExpr::LegacyDollarBraceStringVariable { line, .. } => {
            deprecations.push(LegacyDollarBraceDeprecation { line: *line });
        }
        ValueExpr::DynamicVariable { name, .. } => {
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
        }
        ValueExpr::Assign { target, value, .. } => {
            collect_assignment_target_legacy_dollar_brace_deprecations(target, deprecations);
            collect_value_legacy_dollar_brace_deprecations(value, deprecations);
        }
        ValueExpr::AssignRef { target, source } => {
            collect_assignment_target_legacy_dollar_brace_deprecations(target, deprecations);
            collect_value_legacy_dollar_brace_deprecations(source, deprecations);
        }
        ValueExpr::Array(elements) => {
            for element in elements {
                if let Some(key) = &element.key {
                    collect_value_legacy_dollar_brace_deprecations(key, deprecations);
                }
                match &element.value {
                    IrArrayElementValue::Value(value)
                    | IrArrayElementValue::Unpack { value, .. } => {
                        collect_value_legacy_dollar_brace_deprecations(value, deprecations);
                    }
                    IrArrayElementValue::Reference(target) => {
                        collect_reference_target_legacy_dollar_brace_deprecations(
                            target,
                            deprecations,
                        );
                    }
                }
            }
        }
        ValueExpr::ArrayAccess { array, index, .. } => {
            collect_value_legacy_dollar_brace_deprecations(array, deprecations);
            collect_value_legacy_dollar_brace_deprecations(index, deprecations);
        }
        ValueExpr::ArrayAppendAccess { array, .. } => {
            collect_value_legacy_dollar_brace_deprecations(array, deprecations);
        }
        ValueExpr::Isset { targets } => {
            for target in targets {
                collect_value_legacy_dollar_brace_deprecations(target, deprecations);
            }
        }
        ValueExpr::Empty { target } => {
            collect_value_legacy_dollar_brace_deprecations(target, deprecations);
        }
        ValueExpr::Print { expression } => {
            collect_value_legacy_dollar_brace_deprecations(expression, deprecations);
        }
        ValueExpr::Include { path, .. } => {
            collect_value_legacy_dollar_brace_deprecations(path, deprecations);
        }
        ValueExpr::Throw { value, .. } => {
            collect_value_legacy_dollar_brace_deprecations(value, deprecations);
        }
        ValueExpr::InternalCall { arguments, .. } | ValueExpr::NewObject { arguments, .. } => {
            for argument in arguments {
                collect_value_legacy_dollar_brace_deprecations(argument, deprecations);
            }
        }
        ValueExpr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(class_name, deprecations);
            for argument in arguments {
                collect_value_legacy_dollar_brace_deprecations(argument, deprecations);
            }
        }
        ValueExpr::FirstClassCallable { callable, .. } => {
            collect_value_legacy_dollar_brace_deprecations(callable, deprecations);
        }
        ValueExpr::Clone { expr, .. } => {
            collect_value_legacy_dollar_brace_deprecations(expr, deprecations);
        }
        ValueExpr::DynamicCall {
            callee, arguments, ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(callee, deprecations);
            for argument in arguments {
                collect_value_legacy_dollar_brace_deprecations(argument, deprecations);
            }
        }
        ValueExpr::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
            for argument in arguments {
                collect_value_legacy_dollar_brace_deprecations(argument, deprecations);
            }
        }
        ValueExpr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
            collect_value_legacy_dollar_brace_deprecations(name, deprecations);
            for argument in arguments {
                collect_value_legacy_dollar_brace_deprecations(argument, deprecations);
            }
        }
        ValueExpr::PropertyFetch { receiver, .. } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
        }
        ValueExpr::Unary { expr, .. } | ValueExpr::Cast { expr, .. } => {
            collect_value_legacy_dollar_brace_deprecations(expr, deprecations);
        }
        ValueExpr::Binary { left, right, .. } => {
            collect_value_legacy_dollar_brace_deprecations(left, deprecations);
            collect_value_legacy_dollar_brace_deprecations(right, deprecations);
        }
        ValueExpr::IncDec { target, .. } => {
            collect_inc_dec_target_legacy_dollar_brace_deprecations(target, deprecations);
        }
        ValueExpr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            collect_value_legacy_dollar_brace_deprecations(condition, deprecations);
            if let Some(if_true) = if_true {
                collect_value_legacy_dollar_brace_deprecations(if_true, deprecations);
            }
            collect_value_legacy_dollar_brace_deprecations(if_false, deprecations);
        }
        ValueExpr::DynamicClassNameFetch { receiver, .. } => {
            collect_value_legacy_dollar_brace_deprecations(receiver, deprecations);
        }
        ValueExpr::InstanceOf { expr, .. } => {
            collect_value_legacy_dollar_brace_deprecations(expr, deprecations);
        }
        ValueExpr::String(_)
        | ValueExpr::Int(_)
        | ValueExpr::Float(_)
        | ValueExpr::Bool(_)
        | ValueExpr::Null
        | ValueExpr::Closure { .. }
        | ValueExpr::Load { .. }
        | ValueExpr::Constant(_)
        | ValueExpr::MagicConstant { .. }
        | ValueExpr::StaticPropertyFetch { .. }
        | ValueExpr::ClassConstantFetch { .. } => {}
    }
}

fn module_runtime_requirements(module: &Module) -> RuntimeRequirements {
    let mut requirements = RuntimeRequirements::default();
    collect_instructions_runtime_requirements(
        &module.instructions,
        &module.functions,
        &mut requirements,
    );
    for class in &module.classes {
        for property in &class.properties {
            if let Some(value) = &property.value {
                collect_value_runtime_requirements(value, &module.functions, &mut requirements);
            }
        }
        for property in &class.static_properties {
            if let Some(value) = &property.value {
                collect_value_runtime_requirements(value, &module.functions, &mut requirements);
            }
        }
        for constant in &class.constants {
            collect_value_runtime_requirements(
                &constant.value,
                &module.functions,
                &mut requirements,
            );
        }
    }
    for include in &module.includes {
        collect_instructions_runtime_requirements(
            &include.instructions,
            &module.functions,
            &mut requirements,
        );
    }
    for function in &module.functions {
        collect_instructions_runtime_requirements(
            &function.body,
            &module.functions,
            &mut requirements,
        );
    }
    requirements
}

fn collect_instructions_runtime_requirements(
    instructions: &[Instruction],
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    for instruction in instructions {
        collect_instruction_runtime_requirements(instruction, functions, requirements);
    }
}

fn collect_instruction_runtime_requirements(
    instruction: &Instruction,
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    match instruction {
        Instruction::Store { value, .. }
        | Instruction::DefineConstant { value, .. }
        | Instruction::Expression(value)
        | Instruction::Echo(value) => {
            collect_value_runtime_requirements(value, functions, requirements);
        }
        Instruction::StoreArrayDim {
            dimensions, value, ..
        } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
            collect_value_runtime_requirements(value, functions, requirements);
        }
        Instruction::StoreRef { source, .. } => {
            collect_value_runtime_requirements(source, functions, requirements);
        }
        Instruction::StoreArrayDimRef { target, source } => {
            for dimension in &target.dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
            collect_value_runtime_requirements(source, functions, requirements);
        }
        Instruction::Increment { target, .. } => {
            collect_inc_dec_target_runtime_requirements(target, functions, requirements);
        }
        Instruction::UnsetVariable { .. }
        | Instruction::BindGlobal { .. }
        | Instruction::DeclareFunction { .. } => {}
        Instruction::UnsetDynamicVariable { name, .. } => {
            collect_value_runtime_requirements(name, functions, requirements);
        }
        Instruction::UnsetArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                collect_value_runtime_requirements(dimension, functions, requirements);
            }
        }
        Instruction::UnsetDynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_value_runtime_requirements(name, functions, requirements);
            for dimension in dimensions {
                collect_value_runtime_requirements(dimension, functions, requirements);
            }
        }
        Instruction::UnsetPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
            for dimension in dimensions {
                collect_value_runtime_requirements(dimension, functions, requirements);
            }
        }
        Instruction::UnsetProperty { receiver, .. } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
        }
        Instruction::InternalCall {
            name,
            arguments,
            argument_names,
            ..
        } => {
            collect_call_runtime_requirements(
                name,
                arguments,
                argument_names,
                functions,
                requirements,
            );
        }
        Instruction::Return { value, .. } => {
            if let Some(value) = value {
                collect_value_runtime_requirements(value, functions, requirements);
            }
        }
        Instruction::Throw { value, .. } => {
            collect_value_runtime_requirements(value, functions, requirements);
        }
        Instruction::Try { body, catches } => {
            collect_instructions_runtime_requirements(body, functions, requirements);
            for catch in catches {
                collect_instructions_runtime_requirements(&catch.body, functions, requirements);
            }
        }
        Instruction::Branch {
            condition,
            then_body,
            else_body,
        } => {
            collect_value_runtime_requirements(condition, functions, requirements);
            collect_instructions_runtime_requirements(then_body, functions, requirements);
            collect_instructions_runtime_requirements(else_body, functions, requirements);
        }
        Instruction::While { condition, body } | Instruction::DoWhile { body, condition } => {
            collect_value_runtime_requirements(condition, functions, requirements);
            collect_instructions_runtime_requirements(body, functions, requirements);
        }
        Instruction::For {
            initializers,
            condition,
            updates,
            body,
        } => {
            collect_instructions_runtime_requirements(initializers, functions, requirements);
            if let Some(condition) = condition {
                collect_value_runtime_requirements(condition, functions, requirements);
            }
            collect_instructions_runtime_requirements(updates, functions, requirements);
            collect_instructions_runtime_requirements(body, functions, requirements);
        }
        Instruction::Foreach {
            iterable,
            key,
            value,
            body,
            ..
        } => {
            collect_value_runtime_requirements(iterable, functions, requirements);
            if let Some(key) = key {
                collect_assignment_target_runtime_requirements(key, functions, requirements);
            }
            collect_assignment_target_runtime_requirements(value, functions, requirements);
            collect_instructions_runtime_requirements(body, functions, requirements);
        }
        Instruction::Switch { expression, cases } => {
            collect_value_runtime_requirements(expression, functions, requirements);
            for case in cases {
                if let Some(condition) = &case.condition {
                    collect_value_runtime_requirements(condition, functions, requirements);
                }
                collect_instructions_runtime_requirements(&case.body, functions, requirements);
            }
        }
        Instruction::Break { .. }
        | Instruction::Continue { .. }
        | Instruction::Label { .. }
        | Instruction::Goto { .. } => {}
    }
}

fn collect_reference_target_runtime_requirements(
    target: &ReferenceTarget,
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    match target {
        ReferenceTarget::Variable { .. } => {}
        ReferenceTarget::ArrayDim(target) => {
            for dimension in &target.dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
            for dimension in dimensions.iter().flatten() {
                collect_value_runtime_requirements(dimension, functions, requirements);
            }
        }
        ReferenceTarget::Property { receiver, .. } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
        }
    }
}

fn collect_assignment_target_runtime_requirements(
    target: &AssignmentTarget,
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    match target {
        AssignmentTarget::Variable { .. } => {}
        AssignmentTarget::DynamicVariable { name, .. } => {
            collect_value_runtime_requirements(name, functions, requirements);
        }
        AssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_value_runtime_requirements(name, functions, requirements);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
        }
        AssignmentTarget::ArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
        }
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
        }
        AssignmentTarget::Property { receiver, .. } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
        }
        AssignmentTarget::StaticProperty { .. } => {}
        AssignmentTarget::List(target) => {
            for element in &target.elements {
                if let Some(key) = &element.key {
                    collect_value_runtime_requirements(key, functions, requirements);
                }
                match &element.target {
                    ListAssignmentElementTarget::Value(target) => {
                        collect_assignment_target_runtime_requirements(
                            target,
                            functions,
                            requirements,
                        );
                    }
                    ListAssignmentElementTarget::Reference(target) => {
                        collect_reference_target_runtime_requirements(
                            target,
                            functions,
                            requirements,
                        );
                    }
                }
            }
        }
    }
}

fn collect_inc_dec_target_runtime_requirements(
    target: &IncDecTarget,
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    match target {
        IncDecTarget::Variable { .. } => {}
        IncDecTarget::DynamicVariable { name, .. } => {
            collect_value_runtime_requirements(name, functions, requirements);
        }
        IncDecTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_value_runtime_requirements(name, functions, requirements);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
        }
        IncDecTarget::ArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_value_runtime_requirements(dimension, functions, requirements);
                }
            }
        }
        IncDecTarget::Property { receiver, .. } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
        }
        IncDecTarget::StaticProperty { .. } => {}
    }
}

fn collect_value_runtime_requirements(
    value: &ValueExpr,
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    match value {
        ValueExpr::String(_)
        | ValueExpr::Int(_)
        | ValueExpr::Float(_)
        | ValueExpr::Bool(_)
        | ValueExpr::Null
        | ValueExpr::Closure { .. }
        | ValueExpr::Load { .. }
        | ValueExpr::LegacyDollarBraceStringVariable { .. }
        | ValueExpr::Constant(_)
        | ValueExpr::MagicConstant { .. } => {}
        ValueExpr::IncDec { target, .. } => {
            collect_inc_dec_target_runtime_requirements(target, functions, requirements);
        }
        ValueExpr::DynamicVariable { name, .. } => {
            collect_value_runtime_requirements(name, functions, requirements);
        }
        ValueExpr::Assign { target, value, .. } => {
            collect_assignment_target_runtime_requirements(target, functions, requirements);
            collect_value_runtime_requirements(value, functions, requirements);
        }
        ValueExpr::AssignRef { target, source } => {
            collect_assignment_target_runtime_requirements(target, functions, requirements);
            collect_value_runtime_requirements(source, functions, requirements);
        }
        ValueExpr::Array(elements) => {
            for element in elements {
                if let Some(key) = &element.key {
                    collect_value_runtime_requirements(key, functions, requirements);
                }
                match &element.value {
                    IrArrayElementValue::Value(value)
                    | IrArrayElementValue::Unpack { value, .. } => {
                        collect_value_runtime_requirements(value, functions, requirements);
                    }
                    IrArrayElementValue::Reference(target) => {
                        collect_reference_target_runtime_requirements(
                            target,
                            functions,
                            requirements,
                        );
                    }
                }
            }
        }
        ValueExpr::ArrayAccess { array, index, .. } => {
            collect_value_runtime_requirements(array, functions, requirements);
            collect_value_runtime_requirements(index, functions, requirements);
        }
        ValueExpr::ArrayAppendAccess { array, .. } => {
            collect_value_runtime_requirements(array, functions, requirements);
        }
        ValueExpr::Isset { targets } => {
            for target in targets {
                collect_value_runtime_requirements(target, functions, requirements);
            }
        }
        ValueExpr::Empty { target } => {
            collect_value_runtime_requirements(target, functions, requirements);
        }
        ValueExpr::Print { expression } => {
            collect_value_runtime_requirements(expression, functions, requirements);
        }
        ValueExpr::Include { path, .. } => {
            collect_value_runtime_requirements(path, functions, requirements);
        }
        ValueExpr::Throw { value, .. } => {
            collect_value_runtime_requirements(value, functions, requirements);
        }
        ValueExpr::InternalCall {
            name,
            arguments,
            argument_names,
            ..
        } => {
            collect_call_runtime_requirements(
                name,
                arguments,
                argument_names,
                functions,
                requirements,
            );
        }
        ValueExpr::FirstClassCallable { callable, .. } => {
            collect_value_runtime_requirements(callable, functions, requirements);
            requirements.internal_function_dispatch = true;
        }
        ValueExpr::DynamicCall {
            callee, arguments, ..
        } => {
            collect_value_runtime_requirements(callee, functions, requirements);
            for argument in arguments {
                collect_value_runtime_requirements(argument, functions, requirements);
            }
            requirements.internal_function_dispatch = true;
            requirements.dynamic_function_dispatch = true;
            requirements.method_dispatch = true;
        }
        ValueExpr::MethodCall {
            receiver,
            name,
            arguments,
            ..
        } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
            for argument in arguments {
                collect_value_runtime_requirements(argument, functions, requirements);
            }
            requirements.method_dispatch = true;
            if name.eq_ignore_ascii_case("__invoke") {
                requirements.closure_invoke_method_dispatch = true;
                requirements.internal_function_dispatch = true;
                requirements.dynamic_function_dispatch = true;
            }
        }
        ValueExpr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
            collect_value_runtime_requirements(name, functions, requirements);
            for argument in arguments {
                collect_value_runtime_requirements(argument, functions, requirements);
            }
            requirements.method_dispatch = true;
        }
        ValueExpr::NewObject {
            class_name,
            arguments,
            ..
        } => {
            for argument in arguments {
                collect_value_runtime_requirements(argument, functions, requirements);
            }
            if class_name.eq_ignore_ascii_case("ReflectionFunction") {
                requirements.internal_function_dispatch = true;
                requirements.method_dispatch = true;
            }
        }
        ValueExpr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => {
            collect_value_runtime_requirements(class_name, functions, requirements);
            for argument in arguments {
                collect_value_runtime_requirements(argument, functions, requirements);
            }
            requirements.internal_function_dispatch = true;
            requirements.method_dispatch = true;
        }
        ValueExpr::Clone { expr, .. } => {
            collect_value_runtime_requirements(expr, functions, requirements);
            requirements.method_dispatch = true;
        }
        ValueExpr::PropertyFetch { receiver, .. } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
        }
        ValueExpr::DynamicClassNameFetch { receiver, .. } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
        }
        ValueExpr::InstanceOf { expr, .. } => {
            collect_value_runtime_requirements(expr, functions, requirements);
        }
        ValueExpr::StaticPropertyFetch { .. } | ValueExpr::ClassConstantFetch { .. } => {}
        ValueExpr::Unary { expr, .. } | ValueExpr::Cast { expr, .. } => {
            collect_value_runtime_requirements(expr, functions, requirements);
        }
        ValueExpr::Binary { left, right, .. } => {
            collect_value_runtime_requirements(left, functions, requirements);
            collect_value_runtime_requirements(right, functions, requirements);
        }
        ValueExpr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            collect_value_runtime_requirements(condition, functions, requirements);
            if let Some(if_true) = if_true {
                collect_value_runtime_requirements(if_true, functions, requirements);
            }
            collect_value_runtime_requirements(if_false, functions, requirements);
        }
    }
}

fn collect_call_runtime_requirements(
    name: &str,
    arguments: &[ValueExpr],
    argument_names: &[Option<String>],
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    for argument in arguments {
        collect_value_runtime_requirements(argument, functions, requirements);
    }
    if is_generated_user_function_call(name, functions) {
        return;
    }
    if argument_names.iter().all(Option::is_none)
        && is_direct_internal_helper_call(name, arguments.len())
    {
        requirements.direct_internal_helpers = true;
        return;
    }
    requirements.internal_function_dispatch = true;
    if internal_call_may_invoke_callable(name) {
        requirements.method_dispatch = true;
    }
}

fn is_generated_user_function_call(name: &str, functions: &[FunctionDecl]) -> bool {
    functions.iter().any(|function| {
        !function.is_anonymous
            && (function.class_name.is_none() || function.is_static)
            && function.name.eq_ignore_ascii_case(name)
    })
}

fn is_direct_internal_helper_call(name: &str, argument_count: usize) -> bool {
    (name.eq_ignore_ascii_case("count") && argument_count == 1)
        || (name.eq_ignore_ascii_case("array_key_exists") && argument_count == 2)
}

enum NamedArgumentBindingError {
    Unknown(String),
    Duplicate(String),
}

impl NamedArgumentBindingError {
    fn message(&self) -> String {
        match self {
            NamedArgumentBindingError::Unknown(name) => {
                format!("Unknown named parameter ${name}")
            }
            NamedArgumentBindingError::Duplicate(name) => {
                format!("Named parameter ${name} overwrites previous argument")
            }
        }
    }
}

fn bind_named_call_arguments(
    parameters: &[crate::ir::FunctionParameter],
    argument_names: &[Option<String>],
) -> std::result::Result<Vec<usize>, NamedArgumentBindingError> {
    let mut occupied_parameters = vec![false; parameters.len()];
    let mut slots = Vec::with_capacity(argument_names.len());
    for (argument_index, argument_name) in argument_names.iter().enumerate() {
        let slot = if let Some(argument_name) = argument_name {
            let Some(parameter_index) = parameters
                .iter()
                .position(|parameter| parameter.name == *argument_name)
            else {
                return Err(NamedArgumentBindingError::Unknown(argument_name.clone()));
            };
            if occupied_parameters[parameter_index] {
                return Err(NamedArgumentBindingError::Duplicate(argument_name.clone()));
            }
            occupied_parameters[parameter_index] = true;
            parameter_index
        } else {
            if argument_index < occupied_parameters.len() {
                if occupied_parameters[argument_index] {
                    return Err(NamedArgumentBindingError::Duplicate(
                        parameters[argument_index].name.clone(),
                    ));
                }
                occupied_parameters[argument_index] = true;
            }
            argument_index
        };
        slots.push(slot);
    }
    Ok(slots)
}

#[derive(Clone, Copy)]
enum InternalParameterDefault {
    Null,
    Int(i64),
}

#[derive(Clone, Copy)]
struct InternalParameterSpec {
    name: &'static str,
    default: Option<InternalParameterDefault>,
}

fn internal_named_call_parameters(name: &str) -> Option<&'static [InternalParameterSpec]> {
    static ARRAY_FILTER_PARAMETERS: [InternalParameterSpec; 3] = [
        InternalParameterSpec {
            name: "array",
            default: None,
        },
        InternalParameterSpec {
            name: "callback",
            default: Some(InternalParameterDefault::Null),
        },
        InternalParameterSpec {
            name: "mode",
            default: Some(InternalParameterDefault::Int(0)),
        },
    ];

    if name.eq_ignore_ascii_case("array_filter") {
        Some(&ARRAY_FILTER_PARAMETERS)
    } else {
        None
    }
}

fn internal_parameter_default_expr(default: InternalParameterDefault) -> ValueExpr {
    match default {
        InternalParameterDefault::Null => ValueExpr::Null,
        InternalParameterDefault::Int(value) => ValueExpr::Int(value),
    }
}

fn bind_named_internal_call_arguments(
    name: &str,
    arguments: &[ValueExpr],
    argument_names: &[Option<String>],
) -> Option<std::result::Result<Vec<ValueExpr>, NamedArgumentBindingError>> {
    let parameters = internal_named_call_parameters(name)?;
    let mut slots = vec![None; parameters.len()];
    for (argument_index, (argument, argument_name)) in
        arguments.iter().zip(argument_names.iter()).enumerate()
    {
        let slot = if let Some(argument_name) = argument_name {
            let Some(parameter_index) = parameters
                .iter()
                .position(|parameter| parameter.name == argument_name)
            else {
                return Some(Err(NamedArgumentBindingError::Unknown(
                    argument_name.clone(),
                )));
            };
            parameter_index
        } else {
            argument_index
        };

        if slot >= slots.len() {
            return None;
        }
        if slots[slot].is_some() {
            return Some(Err(NamedArgumentBindingError::Duplicate(
                parameters[slot].name.to_string(),
            )));
        }
        slots[slot] = Some(argument.clone());
    }

    let Some(last_slot) = slots.iter().rposition(Option::is_some) else {
        return Some(Ok(Vec::new()));
    };
    let mut normalized = Vec::with_capacity(last_slot + 1);
    for index in 0..=last_slot {
        if let Some(argument) = &slots[index] {
            normalized.push(argument.clone());
        } else if let Some(default) = parameters[index].default {
            normalized.push(internal_parameter_default_expr(default));
        } else {
            return None;
        }
    }
    Some(Ok(normalized))
}

fn internal_call_may_invoke_callable(name: &str) -> bool {
    name.eq_ignore_ascii_case("array_all")
        || name.eq_ignore_ascii_case("array_any")
        || name.eq_ignore_ascii_case("array_filter")
        || name.eq_ignore_ascii_case("array_find")
        || name.eq_ignore_ascii_case("array_find_key")
        || name.eq_ignore_ascii_case("array_diff_uassoc")
        || name.eq_ignore_ascii_case("array_diff_ukey")
        || name.eq_ignore_ascii_case("array_intersect_uassoc")
        || name.eq_ignore_ascii_case("array_intersect_ukey")
        || name.eq_ignore_ascii_case("array_map")
        || name.eq_ignore_ascii_case("array_reduce")
        || name.eq_ignore_ascii_case("array_udiff")
        || name.eq_ignore_ascii_case("array_udiff_assoc")
        || name.eq_ignore_ascii_case("array_udiff_uassoc")
        || name.eq_ignore_ascii_case("array_uintersect")
        || name.eq_ignore_ascii_case("array_uintersect_assoc")
        || name.eq_ignore_ascii_case("array_uintersect_uassoc")
        || name.eq_ignore_ascii_case("array_walk")
        || name.eq_ignore_ascii_case("array_walk_recursive")
        || name.eq_ignore_ascii_case("call_user_func")
        || name.eq_ignore_ascii_case("call_user_func_array")
}

fn collect_control_warnings_in(
    instructions: &[Instruction],
    contexts: &mut Vec<ControlTargetKind>,
    warnings: &mut Vec<ControlWarning>,
) {
    for instruction in instructions {
        match instruction {
            Instruction::Branch {
                then_body,
                else_body,
                ..
            } => {
                collect_control_warnings_in(then_body, contexts, warnings);
                collect_control_warnings_in(else_body, contexts, warnings);
            }
            Instruction::Try { body, catches } => {
                collect_control_warnings_in(body, contexts, warnings);
                for catch in catches {
                    collect_control_warnings_in(&catch.body, contexts, warnings);
                }
            }
            Instruction::While { body, .. } | Instruction::DoWhile { body, .. } => {
                contexts.push(ControlTargetKind::Loop);
                collect_control_warnings_in(body, contexts, warnings);
                contexts.pop();
            }
            Instruction::For {
                initializers,
                updates,
                body,
                ..
            } => {
                collect_control_warnings_in(initializers, contexts, warnings);
                contexts.push(ControlTargetKind::Loop);
                collect_control_warnings_in(body, contexts, warnings);
                contexts.pop();
                collect_control_warnings_in(updates, contexts, warnings);
            }
            Instruction::Foreach { body, .. } => {
                contexts.push(ControlTargetKind::Loop);
                collect_control_warnings_in(body, contexts, warnings);
                contexts.pop();
            }
            Instruction::Switch { cases, .. } => {
                contexts.push(ControlTargetKind::Switch);
                for case in cases {
                    collect_control_warnings_in(&case.body, contexts, warnings);
                }
                contexts.pop();
            }
            Instruction::Continue { level, line } if *level > 0 && *level <= contexts.len() => {
                let target_index = contexts.len() - *level;
                if contexts[target_index] == ControlTargetKind::Switch {
                    warnings.push(ControlWarning {
                        message: continue_targeting_switch_warning(*level, contexts, target_index),
                        line: *line,
                    });
                }
            }
            _ => {}
        }
    }
}

fn continue_targeting_switch_warning(
    level: usize,
    contexts: &[ControlTargetKind],
    target_index: usize,
) -> String {
    let mut message = format!(
        "{} targeting switch is equivalent to {}",
        quoted_control_operator("continue", level),
        quoted_control_operator("break", level)
    );
    if let Some(suggested_level) = nearest_outer_loop_level(contexts, target_index) {
        message.push_str(". Did you mean to use ");
        message.push_str(&quoted_control_operator("continue", suggested_level));
        message.push('?');
    }
    message
}

fn nearest_outer_loop_level(contexts: &[ControlTargetKind], target_index: usize) -> Option<usize> {
    (0..target_index)
        .rev()
        .find(|index| contexts[*index] == ControlTargetKind::Loop)
        .map(|index| contexts.len() - index)
}

fn quoted_control_operator(operator: &str, level: usize) -> String {
    if level == 1 {
        format!("\"{operator}\"")
    } else {
        format!("\"{operator} {level}\"")
    }
}

fn emit_control_warning(out: &mut String, message: &str, source_path: &str, line: usize) {
    out.push_str("    ptn_emit_control_warning(\"");
    out.push_str(&c_string(message));
    out.push_str("\", \"");
    out.push_str(&c_string(source_path));
    out.push_str("\", ");
    out.push_str(&line.to_string());
    out.push_str(");\n");
}

fn emit_only_variables_assigned_by_reference_notice(out: &mut String, indent: &str, line: usize) {
    out.push_str(indent);
    out.push_str("ptn_emit_only_variables_assigned_by_reference_notice(&runtime.diagnostics, ");
    out.push_str(&line.to_string());
    out.push_str(");\n");
}

fn emit_only_variables_passed_by_reference_notice(out: &mut String, indent: &str, line: usize) {
    out.push_str(indent);
    out.push_str("ptn_emit_only_variables_passed_by_reference_notice(&runtime.diagnostics, ");
    out.push_str(&line.to_string());
    out.push_str(");\n");
}

fn emit_unwrap_append_reference_call_argument(out: &mut String, indent: &str, temp: &str) {
    out.push_str(indent);
    out.push_str("ptn_runtime_unwrap_reference_slots_if_unaliased(&runtime, ");
    out.push_str(temp);
    out.push_str(", 3);\n");
}

fn reference_target_from_value(value: &ValueExpr) -> Option<ReferenceTarget> {
    match value {
        ValueExpr::Load { name, line } => Some(ReferenceTarget::Variable {
            name: name.clone(),
            line: *line,
        }),
        ValueExpr::ArrayAccess { .. } | ValueExpr::ArrayAppendAccess { .. } => {
            reference_array_dim_target_from_value(value)
        }
        ValueExpr::PropertyFetch {
            receiver,
            name,
            line,
        } => Some(ReferenceTarget::Property {
            receiver: receiver.clone(),
            name: name.clone(),
            line: *line,
        }),
        _ => None,
    }
}

fn by_ref_temporary_argument_allowed(value: &ValueExpr) -> bool {
    matches!(
        value,
        ValueExpr::InternalCall { .. }
            | ValueExpr::DynamicCall { .. }
            | ValueExpr::MethodCall { .. }
    )
}

fn cursor_temporary_helper_name(name: &str) -> Option<&'static str> {
    if name.eq_ignore_ascii_case("next") {
        Some("ptn_runtime_array_next_temporary")
    } else if name.eq_ignore_ascii_case("end") {
        Some("ptn_runtime_array_end_temporary")
    } else if name.eq_ignore_ascii_case("prev") {
        Some("ptn_runtime_array_prev_temporary")
    } else if name.eq_ignore_ascii_case("reset") {
        Some("ptn_runtime_array_reset_temporary")
    } else {
        None
    }
}

fn value_is_append_reference_target(value: &ValueExpr) -> bool {
    matches!(
        reference_target_from_value(value),
        Some(ReferenceTarget::ArrayDim(target))
            if target.dimensions.iter().any(Option::is_none)
    ) || matches!(
        reference_target_from_value(value),
        Some(ReferenceTarget::PropertyArrayDim { dimensions, .. })
            if dimensions.iter().any(Option::is_none)
    )
}

fn reference_array_dim_target_from_value(value: &ValueExpr) -> Option<ReferenceTarget> {
    let mut dimensions = Vec::new();
    let mut current = value;
    let mut line = None;
    loop {
        match current {
            ValueExpr::ArrayAccess {
                array,
                index,
                line: access_line,
            } => {
                if line.is_none() {
                    line = Some(*access_line);
                }
                dimensions.push(Some((**index).clone()));
                current = array.as_ref();
            }
            ValueExpr::ArrayAppendAccess {
                array,
                line: access_line,
            } => {
                if line.is_none() {
                    line = Some(*access_line);
                }
                dimensions.push(None);
                current = array.as_ref();
            }
            ValueExpr::Load {
                name,
                line: load_line,
            } => {
                dimensions.reverse();
                return Some(ReferenceTarget::ArrayDim(crate::ir::ArrayDimTarget {
                    array: name.clone(),
                    dimensions,
                    line: line.unwrap_or(*load_line),
                }));
            }
            ValueExpr::PropertyFetch {
                receiver,
                name,
                line: property_line,
            } => {
                dimensions.reverse();
                return Some(ReferenceTarget::PropertyArrayDim {
                    receiver: receiver.clone(),
                    name: name.clone(),
                    dimensions,
                    line: line.unwrap_or(*property_line),
                });
            }
            _ => return None,
        }
    }
}

fn emit_switch(
    out: &mut String,
    values: &mut ValueEmitter,
    expression: &ValueExpr,
    cases: &[crate::ir::SwitchCase],
    control_targets: &mut Vec<ControlTarget>,
    source_path: &str,
    return_target: Option<&str>,
) {
    let end_label = values.next_label("ptn_switch_end");
    emit_label_reference(out, &end_label);
    let labels: Vec<String> = cases
        .iter()
        .map(|_| values.next_label("ptn_switch_case"))
        .collect();
    let switch_temp = values.emit_materialized_value(out, expression);
    if cases.iter().all(|case| case.condition.is_none()) {
        out.push_str("    (void)");
        out.push_str(&switch_temp);
        out.push_str(";\n");
    }
    let mut default_label = None;

    for (case, label) in cases.iter().zip(labels.iter()) {
        if let Some(condition) = &case.condition {
            out.push_str("    {\n");
            let condition_temp = values.emit_materialized_value(out, condition);
            let matched_temp = values.next_temp();
            out.push_str("        int ");
            out.push_str(&matched_temp);
            out.push_str(" = ptn_compare_equal(");
            out.push_str(&switch_temp);
            out.push_str(", ");
            out.push_str(&condition_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "        ", &condition_temp);
            out.push_str("        if (");
            out.push_str(&matched_temp);
            out.push_str(") {\n");
            emit_value_cleanup(out, "            ", &switch_temp);
            out.push_str("            goto ");
            out.push_str(label);
            out.push_str(";\n");
            out.push_str("        }\n");
            out.push_str("    }\n");
        } else {
            default_label = Some(label.as_str());
        }
    }

    emit_value_cleanup(out, "    ", &switch_temp);
    out.push_str("    goto ");
    out.push_str(default_label.unwrap_or(&end_label));
    out.push_str(";\n");

    for (case, label) in cases.iter().zip(labels.iter()) {
        out.push_str("    ");
        out.push_str(label);
        out.push_str(":\n");
        out.push_str("    ;\n");
        control_targets.push(ControlTarget::switch_target(end_label.clone()));
        for body_instruction in &case.body {
            emit_instruction(
                out,
                values,
                body_instruction,
                control_targets,
                source_path,
                return_target,
            );
        }
        control_targets.pop();
    }

    out.push_str("    ");
    out.push_str(&end_label);
    out.push_str(":\n");
    out.push_str("    ;\n");
}

fn emit_label_reference(out: &mut String, label: &str) {
    out.push_str("    if (0) { goto ");
    out.push_str(label);
    out.push_str("; }\n");
}

fn inc_dec_runtime_function(op: IncDecOp) -> &'static str {
    match op {
        IncDecOp::Increment => "ptn_increment_value",
        IncDecOp::Decrement => "ptn_decrement_value",
    }
}

fn emit_increment_statement(
    out: &mut String,
    values: &mut ValueEmitter,
    target: &IncDecTarget,
    op: IncDecOp,
    line: usize,
    source_path: &str,
) {
    match target {
        IncDecTarget::Variable { name, .. } => {
            let current_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&current_temp);
            out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", \"");
            out.push_str(&c_string(source_path));
            out.push_str("\", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            let result_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str(inc_dec_runtime_function(op));
            out.push_str("(&runtime, ");
            out.push_str(&current_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&result_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &current_temp);
            emit_value_cleanup(out, "    ", &result_temp);
        }
        IncDecTarget::DynamicVariable { .. } | IncDecTarget::DynamicArrayDim { .. } => {
            unreachable!("parser restricts statement inc/dec targets")
        }
        IncDecTarget::ArrayDim {
            array, dimensions, ..
        } => {
            out.push_str("    ptn_runtime_array_warn_missing_base_for_assign_op(&runtime, \"");
            out.push_str(&c_string(array));
            out.push_str("\", \"");
            out.push_str(&c_string(source_path));
            out.push_str("\", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            let path = emit_array_path_segments(out, values, dimensions);
            let current_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&current_temp);
            out.push_str(" = ptn_runtime_array_path_read_for_assign_op(&runtime, \"");
            out.push_str(&c_string(array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            let result_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str(inc_dec_runtime_function(op));
            out.push_str("(&runtime, ");
            out.push_str(&current_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_runtime_array_path_set_from_assign_op(&runtime, \"");
            out.push_str(&c_string(array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&result_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &current_temp);
            emit_value_cleanup(out, "    ", &result_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        IncDecTarget::Property { .. } | IncDecTarget::StaticProperty { .. } => {
            let result_temp =
                values.emit_inc_dec_expression(out, target, op, IncDecResult::Pre, line);
            emit_value_cleanup(out, "    ", &result_temp);
        }
    }
}

fn emit_value_cleanup(out: &mut String, indent: &str, value: &str) {
    out.push_str(indent);
    out.push_str("ptn_value_drop(&");
    out.push_str(value);
    out.push_str(");\n");
}

fn is_array_mutating_internal_call(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array_pop"
            | "array_push"
            | "array_shift"
            | "array_splice"
            | "array_unshift"
            | "array_walk"
            | "array_walk_recursive"
            | "array_multisort"
            | "arsort"
            | "asort"
            | "end"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "next"
            | "prev"
            | "reset"
            | "rsort"
            | "shuffle"
            | "sort"
            | "uasort"
            | "uksort"
            | "usort"
    )
}

fn binary_runtime_function(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "ptn_add",
        BinaryOp::Subtract => "ptn_subtract",
        BinaryOp::Multiply => "ptn_multiply",
        BinaryOp::Power => "ptn_power",
        BinaryOp::Divide => "ptn_divide",
        BinaryOp::Modulo => "ptn_modulo",
        BinaryOp::BitwiseAnd => "ptn_bitwise_and",
        BinaryOp::BitwiseXor => "ptn_bitwise_xor",
        BinaryOp::BitwiseOr => "ptn_bitwise_or",
        BinaryOp::ShiftLeft => "ptn_shift_left",
        BinaryOp::ShiftRight => "ptn_shift_right",
        BinaryOp::Concat
        | BinaryOp::Coalesce
        | BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::Spaceship
        | BinaryOp::Identical
        | BinaryOp::NotIdentical
        | BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual
        | BinaryOp::And
        | BinaryOp::Xor
        | BinaryOp::Or => unreachable!("not a direct binary runtime helper"),
    }
}

fn binary_runtime_function_uses_context(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Power
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseXor
            | BinaryOp::BitwiseOr
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight
    )
}

fn assignment_compound_binary_op(op: AssignmentOp) -> Option<BinaryOp> {
    match op {
        AssignmentOp::Assign | AssignmentOp::CoalesceAssign => None,
        AssignmentOp::AddAssign => Some(BinaryOp::Add),
        AssignmentOp::SubtractAssign => Some(BinaryOp::Subtract),
        AssignmentOp::MultiplyAssign => Some(BinaryOp::Multiply),
        AssignmentOp::PowerAssign => Some(BinaryOp::Power),
        AssignmentOp::DivideAssign => Some(BinaryOp::Divide),
        AssignmentOp::ModuloAssign => Some(BinaryOp::Modulo),
        AssignmentOp::ConcatAssign => Some(BinaryOp::Concat),
        AssignmentOp::BitwiseAndAssign => Some(BinaryOp::BitwiseAnd),
        AssignmentOp::BitwiseOrAssign => Some(BinaryOp::BitwiseOr),
        AssignmentOp::BitwiseXorAssign => Some(BinaryOp::BitwiseXor),
        AssignmentOp::ShiftLeftAssign => Some(BinaryOp::ShiftLeft),
        AssignmentOp::ShiftRightAssign => Some(BinaryOp::ShiftRight),
    }
}

pub fn compile_c(c_source: &str, output: &Path) -> Result<()> {
    let c_path = output.with_extension("c");
    fs::write(&c_path, c_source).map_err(|error| {
        Diagnostic::new(
            format!(
                "failed to write generated C source {}: {error}",
                c_path.display()
            ),
            None,
        )
    })?;
    let optimization_args = cc_optimization_args()?;
    let mut command = Command::new("cc");
    command.arg("-std=c11").arg("-Wall").arg("-Wextra");
    for arg in optimization_args {
        command.arg(arg);
    }
    let status = command
        .arg(&c_path)
        .arg("-o")
        .arg(output)
        .arg("-lm")
        .status()
        .map_err(|error| Diagnostic::new(format!("failed to launch cc: {error}"), None))?;
    if status.success() {
        Ok(())
    } else {
        Err(Diagnostic::new(
            format!(
                "cc failed compiling {} to {}",
                display_os(c_path.as_os_str()),
                display_os(output.as_os_str())
            ),
            None,
        ))
    }
}

const CC_OPT_LEVEL_ENV: &str = "PTN_CC_OPT_LEVEL";

fn cc_optimization_args() -> Result<Vec<&'static str>> {
    let value = match env::var(CC_OPT_LEVEL_ENV) {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            return Err(Diagnostic::new(
                format!("{CC_OPT_LEVEL_ENV} must be valid Unicode"),
                None,
            ))
        }
    };
    cc_optimization_args_for(value.as_deref())
}

fn cc_optimization_args_for(value: Option<&str>) -> Result<Vec<&'static str>> {
    let Some(raw) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(vec!["-O2"]);
    };
    match raw.to_ascii_lowercase().as_str() {
        "0" | "o0" | "-o0" | "debug" => Ok(vec!["-O0", "-g"]),
        "1" | "o1" | "-o1" => Ok(vec!["-O1"]),
        "2" | "o2" | "-o2" => Ok(vec!["-O2"]),
        "3" | "o3" | "-o3" => Ok(vec!["-O3"]),
        "s" | "os" | "-os" => Ok(vec!["-Os"]),
        "z" | "oz" | "-oz" => Ok(vec!["-Oz"]),
        _ => Err(Diagnostic::new(
            format!(
                "invalid {CC_OPT_LEVEL_ENV} value `{raw}`; expected 0, 1, 2, 3, s, z, or debug"
            ),
            None,
        )),
    }
}

struct ValueEmitter {
    next_temp: usize,
    next_label: usize,
    source_file: String,
    source_dir: String,
    in_const_declaration: bool,
    current_function_name: Option<String>,
    current_method_name: Option<String>,
    current_class_name: Option<String>,
    current_function_return_by_ref: bool,
    user_functions: Vec<FunctionDecl>,
    classes: Vec<ClassDecl>,
    includes: Vec<IncludeFile>,
}

enum CalledClassOverride {
    Literal(String),
    CurrentCalledOr(String),
}

#[derive(Clone)]
struct StaticMethodVisibilityCheck {
    target_class_name: String,
    declaring_class_name: String,
    method_name: String,
    visibility: PropertyVisibility,
}

struct ConcatOperand<'a> {
    value: &'a ValueExpr,
    line: usize,
}

trait AssignmentTargetLine {
    fn line(&self) -> usize;
}

impl AssignmentTargetLine for AssignmentTarget {
    fn line(&self) -> usize {
        match self {
            AssignmentTarget::Variable { line, .. }
            | AssignmentTarget::DynamicVariable { line, .. }
            | AssignmentTarget::DynamicArrayDim { line, .. }
            | AssignmentTarget::ArrayDim { line, .. }
            | AssignmentTarget::PropertyArrayDim { line, .. } => *line,
            AssignmentTarget::Property { line, .. }
            | AssignmentTarget::StaticProperty { line, .. } => *line,
            AssignmentTarget::List(target) => target.line,
        }
    }
}

impl AssignmentTargetLine for ReferenceTarget {
    fn line(&self) -> usize {
        match self {
            ReferenceTarget::Variable { line, .. } => *line,
            ReferenceTarget::ArrayDim(target) => target.line,
            ReferenceTarget::PropertyArrayDim { line, .. } => *line,
            ReferenceTarget::Property { line, .. } => *line,
        }
    }
}

fn list_assignment_has_reference(target: &ListAssignmentTarget) -> bool {
    target.elements.iter().any(|element| match &element.target {
        ListAssignmentElementTarget::Reference(_) => true,
        ListAssignmentElementTarget::Value(target) => match target.as_ref() {
            AssignmentTarget::List(target) => list_assignment_has_reference(target),
            AssignmentTarget::Variable { .. }
            | AssignmentTarget::DynamicVariable { .. }
            | AssignmentTarget::DynamicArrayDim { .. }
            | AssignmentTarget::ArrayDim { .. }
            | AssignmentTarget::PropertyArrayDim { .. }
            | AssignmentTarget::Property { .. }
            | AssignmentTarget::StaticProperty { .. } => false,
        },
    })
}

fn list_assignment_references_variable(target: &ListAssignmentTarget, name: &str) -> bool {
    target.elements.iter().any(|element| match &element.target {
        ListAssignmentElementTarget::Reference(target) => {
            reference_target_mentions_variable(target, name)
        }
        ListAssignmentElementTarget::Value(target) => {
            assignment_target_mentions_variable(target, name)
        }
    })
}

fn assignment_target_mentions_variable(target: &AssignmentTarget, name: &str) -> bool {
    match target {
        AssignmentTarget::Variable { name: target, .. } => target == name,
        AssignmentTarget::DynamicVariable { name: target, .. } => {
            value_mentions_variable(target, name)
        }
        AssignmentTarget::DynamicArrayDim {
            name: target,
            dimensions,
            ..
        } => {
            value_mentions_variable(target, name)
                || dimensions
                    .iter()
                    .flatten()
                    .any(|dimension| value_mentions_variable(dimension, name))
        }
        AssignmentTarget::ArrayDim { array, .. } => array == name,
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            value_mentions_variable(receiver, name)
                || dimensions
                    .iter()
                    .flatten()
                    .any(|dimension| value_mentions_variable(dimension, name))
        }
        AssignmentTarget::Property { receiver, .. } => value_mentions_variable(receiver, name),
        AssignmentTarget::StaticProperty { .. } => false,
        AssignmentTarget::List(target) => list_assignment_references_variable(target, name),
    }
}

fn reference_target_mentions_variable(target: &ReferenceTarget, name: &str) -> bool {
    match target {
        ReferenceTarget::Variable { name: target, .. } => target == name,
        ReferenceTarget::ArrayDim(target) => target.array == name,
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            value_mentions_variable(receiver, name)
                || dimensions
                    .iter()
                    .flatten()
                    .any(|dimension| value_mentions_variable(dimension, name))
        }
        ReferenceTarget::Property { receiver, .. } => value_mentions_variable(receiver, name),
    }
}

fn inc_dec_target_mentions_variable(target: &IncDecTarget, name: &str) -> bool {
    match target {
        IncDecTarget::Variable { name: target, .. } => target == name,
        IncDecTarget::DynamicVariable { name: target, .. } => value_mentions_variable(target, name),
        IncDecTarget::DynamicArrayDim {
            name: target,
            dimensions,
            ..
        } => {
            value_mentions_variable(target, name)
                || dimensions.iter().any(|dimension| {
                    dimension
                        .as_ref()
                        .is_some_and(|dimension| value_mentions_variable(dimension, name))
                })
        }
        IncDecTarget::ArrayDim {
            array, dimensions, ..
        } => {
            array == name
                || dimensions.iter().any(|dimension| {
                    dimension
                        .as_ref()
                        .is_some_and(|dimension| value_mentions_variable(dimension, name))
                })
        }
        IncDecTarget::Property {
            receiver: target, ..
        } => value_mentions_variable(target, name),
        IncDecTarget::StaticProperty { .. } => false,
    }
}

fn value_mentions_variable(value: &ValueExpr, name: &str) -> bool {
    match value {
        ValueExpr::Load { name: target, .. } => target == name,
        ValueExpr::LegacyDollarBraceStringVariable { name: target, .. } => target == name,
        ValueExpr::DynamicVariable { name: target, .. } => value_mentions_variable(target, name),
        ValueExpr::IncDec { target, .. } => inc_dec_target_mentions_variable(target, name),
        ValueExpr::Assign { target, value, .. } => {
            assignment_target_mentions_variable(target, name)
                || value_mentions_variable(value, name)
        }
        ValueExpr::AssignRef { target, source } => {
            assignment_target_mentions_variable(target, name)
                || value_mentions_variable(source, name)
        }
        ValueExpr::Array(elements) => elements.iter().any(|element| {
            element
                .key
                .as_ref()
                .is_some_and(|key| value_mentions_variable(key, name))
                || match &element.value {
                    IrArrayElementValue::Value(value)
                    | IrArrayElementValue::Unpack { value, .. } => {
                        value_mentions_variable(value, name)
                    }
                    IrArrayElementValue::Reference(target) => {
                        reference_target_mentions_variable(target, name)
                    }
                }
        }),
        ValueExpr::ArrayAccess { array, index, .. } => {
            value_mentions_variable(array, name) || value_mentions_variable(index, name)
        }
        ValueExpr::ArrayAppendAccess { array, .. } => value_mentions_variable(array, name),
        ValueExpr::Isset { targets } => targets
            .iter()
            .any(|target| value_mentions_variable(target, name)),
        ValueExpr::Empty { target } => value_mentions_variable(target, name),
        ValueExpr::Print { expression } => value_mentions_variable(expression, name),
        ValueExpr::Include { path, .. } => value_mentions_variable(path, name),
        ValueExpr::Throw { value, .. } => value_mentions_variable(value, name),
        ValueExpr::InternalCall { arguments, .. } | ValueExpr::NewObject { arguments, .. } => {
            arguments
                .iter()
                .any(|argument| value_mentions_variable(argument, name))
        }
        ValueExpr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => {
            value_mentions_variable(class_name, name)
                || arguments
                    .iter()
                    .any(|argument| value_mentions_variable(argument, name))
        }
        ValueExpr::FirstClassCallable { callable, .. } => value_mentions_variable(callable, name),
        ValueExpr::Clone { expr, .. } => value_mentions_variable(expr, name),
        ValueExpr::DynamicCall {
            callee, arguments, ..
        } => {
            value_mentions_variable(callee, name)
                || arguments
                    .iter()
                    .any(|argument| value_mentions_variable(argument, name))
        }
        ValueExpr::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            value_mentions_variable(receiver, name)
                || arguments
                    .iter()
                    .any(|argument| value_mentions_variable(argument, name))
        }
        ValueExpr::DynamicMethodCall {
            receiver,
            name: method_name,
            arguments,
            ..
        } => {
            value_mentions_variable(receiver, name)
                || value_mentions_variable(method_name, name)
                || arguments
                    .iter()
                    .any(|argument| value_mentions_variable(argument, name))
        }
        ValueExpr::PropertyFetch { receiver, .. }
        | ValueExpr::DynamicClassNameFetch { receiver, .. } => {
            value_mentions_variable(receiver, name)
        }
        ValueExpr::InstanceOf { expr, .. } => value_mentions_variable(expr, name),
        ValueExpr::StaticPropertyFetch { .. } | ValueExpr::ClassConstantFetch { .. } => false,
        ValueExpr::Unary { expr, .. } | ValueExpr::Cast { expr, .. } => {
            value_mentions_variable(expr, name)
        }
        ValueExpr::Binary { left, right, .. } => {
            value_mentions_variable(left, name) || value_mentions_variable(right, name)
        }
        ValueExpr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            value_mentions_variable(condition, name)
                || if_true
                    .as_ref()
                    .is_some_and(|if_true| value_mentions_variable(if_true, name))
                || value_mentions_variable(if_false, name)
        }
        ValueExpr::String(_)
        | ValueExpr::Int(_)
        | ValueExpr::Float(_)
        | ValueExpr::Bool(_)
        | ValueExpr::Null
        | ValueExpr::Closure { .. }
        | ValueExpr::Constant(_)
        | ValueExpr::MagicConstant { .. } => false,
    }
}

fn unpack_requires_literal_fatal(value: &ValueExpr) -> bool {
    matches!(
        value,
        ValueExpr::String(_)
            | ValueExpr::Int(_)
            | ValueExpr::Float(_)
            | ValueExpr::Bool(_)
            | ValueExpr::Null
    )
}

fn const_array_unpack_operand_short_circuits(value: &ValueExpr) -> bool {
    matches!(
        value,
        ValueExpr::NewObject { .. } | ValueExpr::DynamicNewObject { .. }
    )
}

fn static_call_receiver_class_name(call_name: &str, function: &FunctionDecl) -> Option<String> {
    if !function.is_static {
        return None;
    }
    call_name
        .split_once("::")
        .map(|(class_name, _)| class_name.to_string())
        .or_else(|| function.class_name.clone())
}

fn emit_static_call_receiver(out: &mut String, receiver_class_name: Option<&str>) {
    if let Some(receiver_class_name) = receiver_class_name {
        out.push_str("ptn_string(\"");
        out.push_str(&c_string(receiver_class_name));
        out.push_str("\")");
    } else {
        out.push_str("ptn_null()");
    }
}

impl ValueEmitter {
    fn new(
        source_file: &str,
        source_dir: &str,
        functions: &[FunctionDecl],
        classes: &[ClassDecl],
        includes: &[IncludeFile],
    ) -> Self {
        Self::new_with_scope(
            source_file,
            source_dir,
            functions,
            classes,
            includes,
            None,
            None,
            None,
            false,
        )
    }

    fn new_for_function(
        source_file: &str,
        source_dir: &str,
        functions: &[FunctionDecl],
        classes: &[ClassDecl],
        includes: &[IncludeFile],
        function: &FunctionDecl,
    ) -> Self {
        let function_magic_name = function
            .method_name
            .as_deref()
            .unwrap_or(function.name.as_str());
        Self::new_with_scope(
            source_file,
            source_dir,
            functions,
            classes,
            includes,
            Some(function_magic_name),
            Some(function.name.as_str()),
            function.class_name.as_deref(),
            function.return_by_ref,
        )
    }

    fn new_with_scope(
        source_file: &str,
        source_dir: &str,
        functions: &[FunctionDecl],
        classes: &[ClassDecl],
        includes: &[IncludeFile],
        current_function_name: Option<&str>,
        current_method_name: Option<&str>,
        current_class_name: Option<&str>,
        current_function_return_by_ref: bool,
    ) -> Self {
        Self {
            next_temp: 0,
            next_label: 0,
            source_file: source_file.to_string(),
            source_dir: source_dir.to_string(),
            in_const_declaration: false,
            current_function_name: current_function_name.map(str::to_string),
            current_method_name: current_method_name.map(str::to_string),
            current_class_name: current_class_name.map(str::to_string),
            current_function_return_by_ref,
            user_functions: functions.to_vec(),
            classes: classes.to_vec(),
            includes: includes.to_vec(),
        }
    }

    fn emit_const_materialized_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        let previous = self.in_const_declaration;
        self.in_const_declaration = true;
        let value_temp = self.emit_materialized_value(out, value);
        self.in_const_declaration = previous;
        value_temp
    }

    fn declared_class_name(&self, class_name: &str) -> Option<&str> {
        self.classes
            .iter()
            .find(|class| class.name.eq_ignore_ascii_case(class_name))
            .map(|class| class.name.as_str())
    }

    fn declared_parent_class_name(&self, class_name: &str) -> Option<String> {
        self.classes
            .iter()
            .find(|class| class.name.eq_ignore_ascii_case(class_name))
            .and_then(|class| class.parent_name.clone())
    }

    fn static_property_class_name(&self, class_name: &str) -> String {
        if class_name.eq_ignore_ascii_case("parent") {
            if let Some(current_class_name) = &self.current_class_name {
                if let Some(parent_name) = self.declared_parent_class_name(current_class_name) {
                    return parent_name;
                }
            }
        }
        if class_name.eq_ignore_ascii_case("self") || class_name.eq_ignore_ascii_case("static") {
            if let Some(current_class_name) = &self.current_class_name {
                return current_class_name.clone();
            }
        }
        self.declared_class_name(class_name)
            .unwrap_or(class_name)
            .to_string()
    }

    fn static_member_class_name(&self, class_name: &str) -> String {
        self.static_property_class_name(class_name)
    }

    fn class_name_fetch_name(&self, class_name: &str) -> String {
        if class_name.eq_ignore_ascii_case("self") || class_name.eq_ignore_ascii_case("static") {
            if let Some(current_class_name) = &self.current_class_name {
                return current_class_name.clone();
            }
        }
        if class_name.eq_ignore_ascii_case("parent") {
            if let Some(current_class_name) = &self.current_class_name {
                if let Some(parent_name) = self
                    .classes
                    .iter()
                    .find(|class| class.name.eq_ignore_ascii_case(current_class_name))
                    .and_then(|class| class.parent_name.as_deref())
                {
                    return parent_name.to_string();
                }
            }
        }
        class_name.to_string()
    }

    fn class_name_fetch_error_message(&self, class_name: &str) -> Option<String> {
        if class_name.eq_ignore_ascii_case("self")
            || class_name.eq_ignore_ascii_case("static")
            || class_name.eq_ignore_ascii_case("parent")
        {
            let Some(current_class_name) = &self.current_class_name else {
                return Some(format!(
                    "Cannot use \"{}\" in the global scope",
                    class_name.to_ascii_lowercase()
                ));
            };
            if class_name.eq_ignore_ascii_case("parent")
                && self
                    .classes
                    .iter()
                    .find(|class| class.name.eq_ignore_ascii_case(current_class_name))
                    .and_then(|class| class.parent_name.as_deref())
                    .is_none()
            {
                return Some(
                    "Cannot use \"parent\" when current class scope has no parent".to_string(),
                );
            }
        }
        None
    }

    fn static_call_target_class_name(&self, class_name: &str) -> String {
        if class_name.eq_ignore_ascii_case("parent") {
            if let Some(current_class_name) = &self.current_class_name {
                if let Some(parent_name) = self.declared_parent_class_name(current_class_name) {
                    return parent_name;
                }
            }
        }
        if class_name.eq_ignore_ascii_case("self") || class_name.eq_ignore_ascii_case("static") {
            if let Some(current_class_name) = &self.current_class_name {
                return current_class_name.clone();
            }
        }
        self.declared_class_name(class_name)
            .unwrap_or(class_name)
            .to_string()
    }

    fn split_static_call_name<'a>(&self, name: &'a str) -> Option<(&'a str, &'a str)> {
        name.split_once("::")
            .filter(|(class_name, method_name)| !class_name.is_empty() && !method_name.is_empty())
    }

    fn resolved_function_call_name(&self, name: &str) -> String {
        if let Some((class_name, method_name)) = self.split_static_call_name(name) {
            let target_class_name = self.static_call_target_class_name(class_name);
            return format!("{target_class_name}::{method_name}");
        }
        name.to_string()
    }

    fn called_class_override_for_function_call(&self, name: &str) -> Option<CalledClassOverride> {
        let (class_name, _) = self.split_static_call_name(name)?;
        let target_class_name = self.static_call_target_class_name(class_name);
        if class_name.eq_ignore_ascii_case("parent")
            || class_name.eq_ignore_ascii_case("self")
            || class_name.eq_ignore_ascii_case("static")
        {
            let fallback = self
                .current_class_name
                .clone()
                .unwrap_or_else(|| target_class_name.clone());
            return Some(CalledClassOverride::CurrentCalledOr(fallback));
        }
        Some(CalledClassOverride::Literal(target_class_name))
    }

    fn relative_scoped_call_parts(&self, name: &str) -> Option<(String, String)> {
        let (class_name, method_name) = self.split_static_call_name(name)?;
        if !(class_name.eq_ignore_ascii_case("parent")
            || class_name.eq_ignore_ascii_case("self")
            || class_name.eq_ignore_ascii_case("static"))
        {
            return None;
        }
        Some((
            self.static_call_target_class_name(class_name),
            method_name.to_string(),
        ))
    }

    fn emit_called_class_override_expr(out: &mut String, override_: &CalledClassOverride) {
        match override_ {
            CalledClassOverride::Literal(class_name) => {
                out.push('"');
                out.push_str(&c_string(class_name));
                out.push('"');
            }
            CalledClassOverride::CurrentCalledOr(class_name) => {
                out.push_str("(runtime.current_called_class_name != NULL ? runtime.current_called_class_name : \"");
                out.push_str(&c_string(class_name));
                out.push_str("\")");
            }
        }
    }

    fn direct_user_function_by_resolved_name(&self, name: &str) -> Option<(usize, &FunctionDecl)> {
        self.user_functions
            .iter()
            .enumerate()
            .find(|(_, function)| {
                !function.is_anonymous
                    && (function.class_name.is_none() || function.is_static)
                    && function.name.eq_ignore_ascii_case(name)
                    && function
                        .class_name
                        .as_deref()
                        .zip(function.method_name.as_deref())
                        .is_none_or(|(class_name, method_name)| {
                            self.static_method_visibility_error(&format!(
                                "{class_name}::{method_name}"
                            ))
                            .is_none()
                        })
            })
    }

    fn static_method_visibility_check(
        &self,
        resolved_name: &str,
        function: &FunctionDecl,
    ) -> Option<StaticMethodVisibilityCheck> {
        if !function.is_static {
            return None;
        }
        let (target_class_name, method_name) =
            self.split_static_call_name(resolved_name).or_else(|| {
                Some((
                    function.class_name.as_deref()?,
                    function.method_name.as_deref()?,
                ))
            })?;
        let class = class_by_name(&self.classes, target_class_name)?;
        let method = class_method_lookup_chain(class, &self.classes)
            .into_iter()
            .find(|method| method.is_static && method.name.eq_ignore_ascii_case(method_name))?;
        Some(StaticMethodVisibilityCheck {
            target_class_name: target_class_name.to_string(),
            declaring_class_name: method.declaring_class.to_string(),
            method_name: method.name.clone(),
            visibility: method.visibility,
        })
    }

    fn direct_user_function(&self, name: &str) -> Option<(usize, &FunctionDecl)> {
        let resolved_name = self.resolved_function_call_name(name);
        self.direct_user_function_by_resolved_name(&resolved_name)
    }

    fn class_is_same_or_descendant(&self, class_name: &str, ancestor_name: &str) -> bool {
        let mut current = Some(class_name.to_string());
        while let Some(name) = current {
            if name.eq_ignore_ascii_case(ancestor_name) {
                return true;
            }
            current = self.declared_parent_class_name(&name);
        }
        false
    }

    fn class_scope_allows(&self, access_scope: &str, declaring_class: &str) -> bool {
        self.class_is_same_or_descendant(access_scope, declaring_class)
            || self.class_is_same_or_descendant(declaring_class, access_scope)
    }

    fn method_visibility_allows(
        &self,
        visibility: PropertyVisibility,
        declaring_class: &str,
    ) -> bool {
        match visibility {
            PropertyVisibility::Public => true,
            PropertyVisibility::Private => self
                .current_class_name
                .as_ref()
                .is_some_and(|scope| scope.eq_ignore_ascii_case(declaring_class)),
            PropertyVisibility::Protected => self
                .current_class_name
                .as_ref()
                .is_some_and(|scope| self.class_scope_allows(scope, declaring_class)),
        }
    }

    fn protected_static_method_root_allows(&self, target_class: &str, method_name: &str) -> bool {
        let Some(access_scope) = self.current_class_name.as_deref() else {
            return false;
        };
        self.classes.iter().any(|class| {
            self.class_is_same_or_descendant(access_scope, &class.name)
                && self.class_is_same_or_descendant(target_class, &class.name)
                && class.methods.iter().any(|method| {
                    method.is_static
                        && method.visibility == PropertyVisibility::Protected
                        && method.name.eq_ignore_ascii_case(method_name)
                })
        })
    }

    fn static_method_visibility_error(
        &self,
        resolved_name: &str,
    ) -> Option<(PropertyVisibility, String, String)> {
        let (class_name, method_name) = self.split_static_call_name(resolved_name)?;
        let class = self
            .classes
            .iter()
            .find(|class| class.name.eq_ignore_ascii_case(class_name))?;
        for entry in class_method_lookup_chain(class, &self.classes) {
            let method = entry.method;
            if !method.is_static || !method.name.eq_ignore_ascii_case(method_name) {
                continue;
            }
            if self.method_visibility_allows(method.visibility, entry.declaring_class)
                || (method.visibility == PropertyVisibility::Protected
                    && self.protected_static_method_root_allows(class_name, method_name))
            {
                return None;
            }
            return Some((
                method.visibility,
                entry.declaring_class.to_string(),
                method.name.clone(),
            ));
        }
        None
    }

    fn emit_static_method_visibility_error(
        &mut self,
        out: &mut String,
        result_temp: &str,
        visibility: PropertyVisibility,
        declaring_class: &str,
        method_name: &str,
        line: usize,
    ) {
        out.push_str("    PtnValue ");
        out.push_str(result_temp);
        out.push_str(" = ptn_null();\n");
        out.push_str("    ptn_throw_declared_method_visibility_error(&runtime, \"");
        out.push_str(method_visibility_name(visibility));
        out.push_str("\", \"");
        out.push_str(&c_string(declaring_class));
        out.push_str("\", \"");
        out.push_str(&c_string(method_name));
        out.push_str("\", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
    }

    fn declared_class_has_descendant(&self, class_name: &str) -> bool {
        self.classes.iter().any(|class| {
            !class.name.eq_ignore_ascii_case(class_name)
                && self.class_is_same_or_descendant(&class.name, class_name)
        })
    }

    fn declared_instance_method_signature_for_receiver(
        &self,
        receiver: &ValueExpr,
        name: &str,
    ) -> Option<(String, Vec<FunctionParameter>)> {
        let class_name = match receiver {
            ValueExpr::Load { name, .. } if name == "this" => {
                let class_name = self.current_class_name.clone()?;
                if self.declared_class_has_descendant(&class_name) {
                    return None;
                }
                class_name
            }
            ValueExpr::NewObject { class_name, .. } => self.class_name_fetch_name(class_name),
            _ => return None,
        };
        let class = class_by_name(&self.classes, &class_name)?;
        let method = class_method_lookup_chain(class, &self.classes)
            .into_iter()
            .find(|method| !method.is_static && method.name.eq_ignore_ascii_case(name))?;
        let function = self.user_functions.get(method.function_index)?;
        Some((function.display_name.clone(), function.parameters.clone()))
    }

    fn source_is_declared_by_ref_call(&self, source: &ValueExpr) -> bool {
        match source {
            ValueExpr::InternalCall { name, .. } => self
                .direct_user_function(name)
                .map(|(_, function)| function.return_by_ref)
                .unwrap_or(false),
            _ => false,
        }
    }

    fn emit_dynamic_variable_name(
        &mut self,
        out: &mut String,
        name: &ValueExpr,
        line: usize,
    ) -> String {
        let value_temp = self.emit_materialized_value(out, name);
        let name_temp = self.next_temp();
        out.push_str("    char *");
        out.push_str(&name_temp);
        out.push_str(" = ptn_dynamic_variable_name(&runtime, ");
        out.push_str(&value_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &value_temp);
        name_temp
    }

    fn emit_dynamic_variable_read(
        &mut self,
        out: &mut String,
        name: &ValueExpr,
        line: usize,
    ) -> String {
        let name_temp = self.emit_dynamic_variable_name(out, name, line);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_runtime_read_variable(&runtime, ");
        out.push_str(&name_temp);
        out.push_str(", \"");
        out.push_str(&c_string(&self.source_file));
        out.push_str("\", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        out.push_str("    free(");
        out.push_str(&name_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_assignment(
        &mut self,
        out: &mut String,
        target: &AssignmentTarget,
        op: AssignmentOp,
        value: &ValueExpr,
    ) -> String {
        if matches!(op, AssignmentOp::CoalesceAssign) {
            match target {
                AssignmentTarget::Variable { name, .. } => {
                    return self.emit_coalesce_assignment(out, name, value);
                }
                AssignmentTarget::DynamicVariable { name, line } => {
                    return self.emit_dynamic_coalesce_assignment(out, name, *line, value);
                }
                AssignmentTarget::DynamicArrayDim {
                    name,
                    dimensions,
                    line,
                } => {
                    return self.emit_dynamic_offset_coalesce_assignment(
                        out, name, dimensions, *line, value,
                    );
                }
                AssignmentTarget::ArrayDim {
                    array,
                    dimensions,
                    line,
                } => {
                    return self
                        .emit_offset_coalesce_assignment(out, array, dimensions, *line, value);
                }
                AssignmentTarget::PropertyArrayDim { .. } => {
                    unreachable!(
                        "parser rejects null coalescing assignment for property array offsets"
                    );
                }
                AssignmentTarget::List(_) => {
                    unreachable!("parser rejects null coalescing assignment for list targets");
                }
                AssignmentTarget::Property {
                    receiver,
                    name,
                    line,
                } => {
                    return self
                        .emit_property_coalesce_assignment(out, receiver, name, *line, value);
                }
                AssignmentTarget::StaticProperty {
                    class_name,
                    name,
                    line,
                } => {
                    return self.emit_static_property_coalesce_assignment(
                        out, class_name, name, *line, value,
                    );
                }
            }
        }

        if let AssignmentTarget::List(target) = target {
            return self.emit_list_assignment(out, target, value);
        }

        if let AssignmentTarget::DynamicArrayDim {
            name,
            dimensions,
            line,
        } = target
        {
            let name_temp = self.emit_dynamic_variable_name(out, name, *line);
            let path = emit_array_path_segments(out, self, dimensions);
            let value_temp = self.emit_materialized_value(out, value);
            let mut current_cleanup_temp = None;
            let stored_temp = if let Some(op) = assignment_compound_binary_op(op) {
                out.push_str("    ptn_runtime_array_warn_missing_base_for_assign_op(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_array_path_read_for_assign_op(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                if matches!(op, BinaryOp::Concat) {
                    out.push_str("ptn_concat(&runtime, ");
                    out.push_str(&current_temp);
                    out.push_str(", ");
                    out.push_str(&value_temp);
                    out.push_str(", ");
                    out.push_str(&line.to_string());
                    out.push_str(")");
                } else {
                    out.push_str(binary_runtime_function(op));
                    out.push('(');
                    if binary_runtime_function_uses_context(op) {
                        out.push_str("&runtime, ");
                    }
                    out.push_str(&current_temp);
                    out.push_str(", ");
                    out.push_str(&value_temp);
                    if binary_runtime_function_uses_context(op) {
                        out.push_str(", ");
                        out.push_str(&line.to_string());
                    }
                    out.push(')');
                }
                out.push_str(";\n");
                current_cleanup_temp = Some(current_temp);
                result_temp
            } else {
                let snapshot_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&snapshot_temp);
                out.push_str(" = ptn_value_snapshot_for_array_path_write(");
                out.push_str(&value_temp);
                out.push_str(");\n");
                snapshot_temp
            };
            out.push_str("    ");
            out.push_str(if assignment_compound_binary_op(op).is_some() {
                "ptn_runtime_array_path_set_from_assign_op"
            } else {
                "ptn_runtime_array_path_set"
            });
            out.push_str("(&runtime, ");
            out.push_str(&name_temp);
            out.push_str(", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&stored_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_value_clone(");
            out.push_str(&stored_temp);
            out.push_str(");\n");
            out.push_str("    free(");
            out.push_str(&name_temp);
            out.push_str(");\n");
            if let Some(current_temp) = current_cleanup_temp {
                emit_value_cleanup(out, "    ", &current_temp);
            }
            emit_value_cleanup(out, "    ", &stored_temp);
            emit_value_cleanup(out, "    ", &value_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
            return result_temp;
        }

        if let AssignmentTarget::ArrayDim {
            array,
            dimensions,
            line,
        } = target
        {
            if let Some(compound_op) = assignment_compound_binary_op(op) {
                return self.emit_array_dim_compound_assignment(
                    out,
                    array,
                    dimensions,
                    *line,
                    compound_op,
                    value,
                );
            }
            let path = emit_array_path_segments(out, self, dimensions);
            let value_temp = self.emit_materialized_value(out, value);
            let snapshot_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&snapshot_temp);
            out.push_str(" = ptn_value_snapshot_for_array_path_write(");
            out.push_str(&value_temp);
            out.push_str(");\n");
            out.push_str("    ptn_runtime_array_path_set(&runtime, \"");
            out.push_str(&c_string(array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&snapshot_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_value_clone(");
            out.push_str(&snapshot_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &snapshot_temp);
            emit_value_cleanup(out, "    ", &value_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
            return result_temp;
        }

        if let AssignmentTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            line,
        } = target
        {
            if let Some(compound_op) = assignment_compound_binary_op(op) {
                return self.emit_property_array_dim_compound_assignment(
                    out,
                    receiver,
                    name,
                    dimensions,
                    *line,
                    compound_op,
                    value,
                );
            }
            let value_temp = self.emit_materialized_value(out, value);
            let result_temp = self.emit_property_array_dim_assignment_from_temp(
                out,
                receiver,
                name,
                dimensions,
                *line,
                &value_temp,
            );
            emit_value_cleanup(out, "    ", &value_temp);
            return result_temp;
        }

        if let AssignmentTarget::DynamicVariable { name, line } = target {
            let name_temp = self.emit_dynamic_variable_name(out, name, *line);
            let value_temp = self.emit_materialized_value(out, value);
            out.push_str("    ptn_runtime_write_variable(&runtime, ");
            out.push_str(&name_temp);
            out.push_str(", ");
            out.push_str(&value_temp);
            out.push_str(");\n");
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_value_clone(ptn_value_deref(");
            out.push_str(&value_temp);
            out.push_str("));\n");
            out.push_str("    free(");
            out.push_str(&name_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &value_temp);
            return result_temp;
        }

        if let AssignmentTarget::Property {
            receiver,
            name,
            line,
        } = target
        {
            if let Some(compound_op) = assignment_compound_binary_op(op) {
                return self.emit_property_compound_assignment(
                    out,
                    receiver,
                    name,
                    *line,
                    compound_op,
                    value,
                );
            }
            let receiver_temp = self.emit_materialized_value(out, receiver);
            let value_temp = self.emit_materialized_value(out, value);
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_object_write_property(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&c_optional_string(self.current_class_name.as_deref()));
            out.push_str(", ");
            out.push_str(&value_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &value_temp);
            emit_value_cleanup(out, "    ", &receiver_temp);
            return result_temp;
        }

        if let AssignmentTarget::StaticProperty {
            class_name,
            name,
            line,
        } = target
        {
            if let Some(compound_op) = assignment_compound_binary_op(op) {
                return self.emit_static_property_compound_assignment(
                    out,
                    class_name,
                    name,
                    *line,
                    compound_op,
                    value,
                );
            }
        }

        let value_temp = self.emit_materialized_value(out, value);
        let result_temp = self.emit_store_assignment_target_from_temp(out, target, &value_temp);
        emit_value_cleanup(out, "    ", &value_temp);
        result_temp
    }

    fn emit_compound_binary_value(
        &mut self,
        out: &mut String,
        current_temp: &str,
        value_temp: &str,
        line: usize,
        op: BinaryOp,
    ) -> String {
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        if matches!(op, BinaryOp::Concat) {
            out.push_str("ptn_concat(&runtime, ");
            out.push_str(current_temp);
            out.push_str(", ");
            out.push_str(value_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(")");
        } else {
            out.push_str(binary_runtime_function(op));
            out.push('(');
            if binary_runtime_function_uses_context(op) {
                out.push_str("&runtime, ");
            }
            out.push_str(current_temp);
            out.push_str(", ");
            out.push_str(value_temp);
            if binary_runtime_function_uses_context(op) {
                out.push_str(", ");
                out.push_str(&line.to_string());
            }
            out.push(')');
        }
        out.push_str(";\n");
        result_temp
    }

    fn emit_array_dim_compound_assignment(
        &mut self,
        out: &mut String,
        array: &str,
        dimensions: &[Option<ValueExpr>],
        line: usize,
        op: BinaryOp,
        value: &ValueExpr,
    ) -> String {
        out.push_str("    ptn_runtime_array_warn_missing_base_for_assign_op(&runtime, \"");
        out.push_str(&c_string(array));
        out.push_str("\", \"");
        out.push_str(&c_string(&self.source_file));
        out.push_str("\", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let path = emit_array_path_segments(out, self, dimensions);
        let value_temp = self.emit_materialized_value(out, value);

        let current_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&current_temp);
        out.push_str(" = ptn_runtime_array_path_read_for_assign_op(&runtime, \"");
        out.push_str(&c_string(array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp =
            self.emit_compound_binary_value(out, &current_temp, &value_temp, line, op);

        out.push_str("    ptn_runtime_array_path_set_from_assign_op(&runtime, \"");
        out.push_str(&c_string(array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&result_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        emit_value_cleanup(out, "    ", &current_temp);
        emit_value_cleanup(out, "    ", &value_temp);
        for segment_temp in path.value_temps {
            emit_value_cleanup(out, "    ", &segment_temp);
        }
        result_temp
    }

    fn emit_property_array_dim_assignment_from_temp(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        dimensions: &[Option<ValueExpr>],
        line: usize,
        value_temp: &str,
    ) -> String {
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let path = emit_array_path_segments(out, self, dimensions);
        let current_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&current_temp);
        out.push_str(" = ptn_object_read_property(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let snapshot_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&snapshot_temp);
        out.push_str(" = ptn_value_snapshot_for_array_path_write(");
        out.push_str(value_temp);
        out.push_str(");\n");
        out.push_str("    ptn_value_array_path_set(&runtime, &");
        out.push_str(&current_temp);
        out.push_str(", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&snapshot_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let assigned_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&assigned_temp);
        out.push_str(" = ptn_object_write_property_indirect(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&current_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone(");
        out.push_str(&snapshot_temp);
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &assigned_temp);
        emit_value_cleanup(out, "    ", &current_temp);
        emit_value_cleanup(out, "    ", &snapshot_temp);
        emit_value_cleanup(out, "    ", &receiver_temp);
        for segment_temp in path.value_temps {
            emit_value_cleanup(out, "    ", &segment_temp);
        }
        result_temp
    }

    fn emit_property_array_dim_compound_assignment(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        dimensions: &[Option<ValueExpr>],
        line: usize,
        op: BinaryOp,
        value: &ValueExpr,
    ) -> String {
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let path = emit_array_path_segments(out, self, dimensions);
        let value_temp = self.emit_materialized_value(out, value);

        let current_value_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&current_value_temp);
        out.push_str(" = ptn_object_read_property(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let current_element_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&current_element_temp);
        out.push_str(" = ptn_value_array_path_read_for_assign_op(&runtime, ");
        out.push_str(&current_value_temp);
        out.push_str(", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp =
            self.emit_compound_binary_value(out, &current_element_temp, &value_temp, line, op);
        out.push_str("    ptn_value_array_path_set_from_assign_op(&runtime, &");
        out.push_str(&current_value_temp);
        out.push_str(", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&result_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let assigned_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&assigned_temp);
        out.push_str(" = ptn_object_write_property_indirect(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&current_value_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        emit_value_cleanup(out, "    ", &assigned_temp);
        emit_value_cleanup(out, "    ", &current_element_temp);
        emit_value_cleanup(out, "    ", &current_value_temp);
        emit_value_cleanup(out, "    ", &value_temp);
        emit_value_cleanup(out, "    ", &receiver_temp);
        for segment_temp in path.value_temps {
            emit_value_cleanup(out, "    ", &segment_temp);
        }
        result_temp
    }

    fn emit_property_compound_assignment(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        line: usize,
        op: BinaryOp,
        value: &ValueExpr,
    ) -> String {
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let value_temp = self.emit_materialized_value(out, value);

        let current_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&current_temp);
        out.push_str(" = ptn_object_read_property(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp =
            self.emit_compound_binary_value(out, &current_temp, &value_temp, line, op);
        let assigned_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&assigned_temp);
        out.push_str(" = ptn_object_write_property(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&result_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        emit_value_cleanup(out, "    ", &assigned_temp);
        emit_value_cleanup(out, "    ", &current_temp);
        emit_value_cleanup(out, "    ", &value_temp);
        emit_value_cleanup(out, "    ", &receiver_temp);
        result_temp
    }

    fn emit_static_property_compound_assignment(
        &mut self,
        out: &mut String,
        class_name: &str,
        name: &str,
        line: usize,
        op: BinaryOp,
        value: &ValueExpr,
    ) -> String {
        let resolved_class_name = self.static_property_class_name(class_name);
        let value_temp = self.emit_materialized_value(out, value);

        let current_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&current_temp);
        out.push_str(" = ptn_runtime_read_static_property(&runtime, \"");
        out.push_str(&c_string(&resolved_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp =
            self.emit_compound_binary_value(out, &current_temp, &value_temp, line, op);
        let assigned_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&assigned_temp);
        out.push_str(" = ptn_runtime_write_static_property(&runtime, \"");
        out.push_str(&c_string(&resolved_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&result_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        emit_value_cleanup(out, "    ", &assigned_temp);
        emit_value_cleanup(out, "    ", &current_temp);
        emit_value_cleanup(out, "    ", &value_temp);
        result_temp
    }

    fn emit_reference_assignment(
        &mut self,
        out: &mut String,
        target: &AssignmentTarget,
        source: &ValueExpr,
    ) -> String {
        if let Some(source_target) = reference_target_from_value(source) {
            let source_temp = self.emit_reference_target(out, &source_target);
            self.emit_bind_assignment_target_reference(out, target, &source_temp);
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_value_clone(ptn_value_deref(");
            out.push_str(&source_temp);
            out.push_str("));\n");
            emit_value_cleanup(out, "    ", &source_temp);
            return result_temp;
        }

        let source_temp = self.emit_materialized_value(out, source);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_null();\n");
        out.push_str("    if (");
        out.push_str(&source_temp);
        out.push_str(".type == PTN_REFERENCE) {\n");
        self.emit_bind_assignment_target_reference(out, target, &source_temp);
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone(ptn_value_deref(");
        out.push_str(&source_temp);
        out.push_str("));\n");
        out.push_str("    } else {\n");
        let stored_temp = self.emit_store_assignment_target_from_temp(out, target, &source_temp);
        if !self.source_is_declared_by_ref_call(source) {
            emit_only_variables_assigned_by_reference_notice(out, "        ", target.line());
        }
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&stored_temp);
        out.push_str(";\n");
        out.push_str("    }\n");
        emit_value_cleanup(out, "    ", &source_temp);
        result_temp
    }

    fn emit_store_reference_source_to_variable(
        &mut self,
        out: &mut String,
        name: &str,
        source: &ValueExpr,
        line: usize,
    ) {
        if let Some(target) = reference_target_from_value(source) {
            let reference_temp = self.emit_reference_target(out, &target);
            out.push_str("    ptn_runtime_bind_variable_reference(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&reference_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &reference_temp);
            return;
        }

        let source_temp = self.emit_materialized_value(out, source);
        out.push_str("    if (");
        out.push_str(&source_temp);
        out.push_str(".type == PTN_REFERENCE) {\n");
        out.push_str("        ptn_runtime_bind_variable_reference(&runtime, \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&source_temp);
        out.push_str(");\n");
        out.push_str("    } else {\n");
        if !self.source_is_declared_by_ref_call(source) {
            emit_only_variables_assigned_by_reference_notice(out, "        ", line);
        }
        out.push_str("        ptn_runtime_write_variable(&runtime, \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&source_temp);
        out.push_str(");\n");
        out.push_str("    }\n");
        emit_value_cleanup(out, "    ", &source_temp);
    }

    fn emit_store_reference_source_to_array_dim(
        &mut self,
        out: &mut String,
        target: &crate::ir::ArrayDimTarget,
        source: &ValueExpr,
        source_path: &str,
    ) {
        if let Some(source_target) = reference_target_from_value(source) {
            let source_temp = self.emit_reference_target(out, &source_target);
            let path = emit_array_path_segments(out, self, &target.dimensions);
            out.push_str("    ptn_runtime_bind_array_path_reference(&runtime, \"");
            out.push_str(&c_string(&target.array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&source_temp);
            out.push_str(", \"");
            out.push_str(&c_string(source_path));
            out.push_str("\", ");
            out.push_str(&target.line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &source_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
            return;
        }

        let source_temp = self.emit_materialized_value(out, source);
        let path = emit_array_path_segments(out, self, &target.dimensions);
        out.push_str("    if (");
        out.push_str(&source_temp);
        out.push_str(".type == PTN_REFERENCE) {\n");
        out.push_str("        ptn_runtime_bind_array_path_reference(&runtime, \"");
        out.push_str(&c_string(&target.array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&source_temp);
        out.push_str(", \"");
        out.push_str(&c_string(source_path));
        out.push_str("\", ");
        out.push_str(&target.line.to_string());
        out.push_str(");\n");
        out.push_str("    } else {\n");
        if !self.source_is_declared_by_ref_call(source) {
            emit_only_variables_assigned_by_reference_notice(out, "        ", target.line);
        }
        let snapshot_temp = self.next_temp();
        out.push_str("        PtnValue ");
        out.push_str(&snapshot_temp);
        out.push_str(" = ptn_value_snapshot_for_array_path_write(");
        out.push_str(&source_temp);
        out.push_str(");\n");
        out.push_str("        ptn_runtime_array_path_set(&runtime, \"");
        out.push_str(&c_string(&target.array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&snapshot_temp);
        out.push_str(", ");
        out.push_str(&target.line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "        ", &snapshot_temp);
        out.push_str("    }\n");
        emit_value_cleanup(out, "    ", &source_temp);
        for segment_temp in path.value_temps {
            emit_value_cleanup(out, "    ", &segment_temp);
        }
    }

    fn emit_bind_assignment_target_reference(
        &mut self,
        out: &mut String,
        target: &AssignmentTarget,
        reference_temp: &str,
    ) {
        match target {
            AssignmentTarget::Variable { name, line } => {
                self.emit_bind_reference_target(
                    out,
                    &ReferenceTarget::Variable {
                        name: name.clone(),
                        line: *line,
                    },
                    reference_temp,
                );
            }
            AssignmentTarget::DynamicVariable { .. } => {
                unreachable!("parser rejects by-reference assignment to dynamic variable targets");
            }
            AssignmentTarget::DynamicArrayDim { .. } => {
                unreachable!("parser rejects by-reference assignment to dynamic array targets");
            }
            AssignmentTarget::ArrayDim {
                array,
                dimensions,
                line,
            } => {
                self.emit_bind_reference_target(
                    out,
                    &ReferenceTarget::ArrayDim(crate::ir::ArrayDimTarget {
                        array: array.clone(),
                        dimensions: dimensions.clone(),
                        line: *line,
                    }),
                    reference_temp,
                );
            }
            AssignmentTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                line,
            } => {
                self.emit_bind_reference_target(
                    out,
                    &ReferenceTarget::PropertyArrayDim {
                        receiver: receiver.clone(),
                        name: name.clone(),
                        dimensions: dimensions.clone(),
                        line: *line,
                    },
                    reference_temp,
                );
            }
            AssignmentTarget::Property {
                receiver,
                name,
                line,
            } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                out.push_str("    ptn_object_bind_property_reference(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(reference_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &receiver_temp);
            }
            AssignmentTarget::StaticProperty { .. } => {
                unreachable!("parser rejects by-reference assignment to static property targets");
            }
            AssignmentTarget::List(_) => {
                unreachable!("parser rejects by-reference assignment to list targets");
            }
        }
    }

    fn emit_store_assignment_target_from_temp(
        &mut self,
        out: &mut String,
        target: &AssignmentTarget,
        value_temp: &str,
    ) -> String {
        match target {
            AssignmentTarget::Variable { name, .. } => {
                out.push_str("    ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(value_temp);
                out.push_str(");\n");
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_value_clone(ptn_value_deref(");
                out.push_str(value_temp);
                out.push_str("));\n");
                result_temp
            }
            AssignmentTarget::DynamicVariable { name, line } => {
                let name_temp = self.emit_dynamic_variable_name(out, name, *line);
                out.push_str("    ptn_runtime_write_variable(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", ");
                out.push_str(value_temp);
                out.push_str(");\n");
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_value_clone(ptn_value_deref(");
                out.push_str(value_temp);
                out.push_str("));\n");
                out.push_str("    free(");
                out.push_str(&name_temp);
                out.push_str(");\n");
                result_temp
            }
            AssignmentTarget::DynamicArrayDim {
                name,
                dimensions,
                line,
            } => {
                let name_temp = self.emit_dynamic_variable_name(out, name, *line);
                let path = emit_array_path_segments(out, self, dimensions);
                let snapshot_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&snapshot_temp);
                out.push_str(" = ptn_value_snapshot_for_array_path_write(");
                out.push_str(value_temp);
                out.push_str(");\n");
                out.push_str("    ptn_runtime_array_path_set(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&snapshot_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_value_clone(");
                out.push_str(&snapshot_temp);
                out.push_str(");\n");
                out.push_str("    free(");
                out.push_str(&name_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &snapshot_temp);
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
                result_temp
            }
            AssignmentTarget::ArrayDim {
                array,
                dimensions,
                line,
            } => {
                let path = emit_array_path_segments(out, self, dimensions);
                let snapshot_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&snapshot_temp);
                out.push_str(" = ptn_value_snapshot_for_array_path_write(");
                out.push_str(value_temp);
                out.push_str(");\n");
                out.push_str("    ptn_runtime_array_path_set(&runtime, \"");
                out.push_str(&c_string(array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&snapshot_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_value_clone(");
                out.push_str(&snapshot_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &snapshot_temp);
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
                result_temp
            }
            AssignmentTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                line,
            } => self.emit_property_array_dim_assignment_from_temp(
                out, receiver, name, dimensions, *line, value_temp,
            ),
            AssignmentTarget::Property {
                receiver,
                name,
                line,
            } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_object_write_property(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(value_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &receiver_temp);
                result_temp
            }
            AssignmentTarget::StaticProperty {
                class_name,
                name,
                line,
            } => {
                let resolved_class_name = self.static_property_class_name(class_name);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_write_static_property(&runtime, \"");
                out.push_str(&c_string(&resolved_class_name));
                out.push_str("\", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(value_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                result_temp
            }
            AssignmentTarget::List(target) => {
                self.emit_list_assignment_from_temp(out, target, value_temp)
            }
        }
    }

    fn emit_list_assignment(
        &mut self,
        out: &mut String,
        target: &ListAssignmentTarget,
        value: &ValueExpr,
    ) -> String {
        if list_assignment_has_reference(target) {
            if let ValueExpr::Load { name, .. } = value {
                return self.emit_reference_list_assignment_from_variable(out, target, name);
            }
        }

        let value_temp = self.emit_materialized_value(out, value);
        let result_temp = self.emit_list_assignment_from_temp(out, target, &value_temp);
        emit_value_cleanup(out, "    ", &value_temp);
        result_temp
    }

    fn emit_list_assignment_from_temp(
        &mut self,
        out: &mut String,
        target: &ListAssignmentTarget,
        value_temp: &str,
    ) -> String {
        for (index, element) in target.elements.iter().enumerate() {
            let key_temp = self.emit_list_key(out, element, index);
            match &element.target {
                ListAssignmentElementTarget::Value(target) => {
                    let element_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&element_temp);
                    out.push_str(" = ptn_array_read_for_list_destructure(&runtime, ");
                    out.push_str(value_temp);
                    out.push_str(", ");
                    out.push_str(&key_temp);
                    out.push_str(", ");
                    out.push_str(&target.line().to_string());
                    out.push_str(");\n");
                    let stored_temp =
                        self.emit_store_assignment_target_from_temp(out, target, &element_temp);
                    emit_value_cleanup(out, "    ", &stored_temp);
                    emit_value_cleanup(out, "    ", &element_temp);
                }
                ListAssignmentElementTarget::Reference(target) => {
                    let source_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&source_temp);
                    out.push_str(" = ptn_runtime_reference_for_array_value_dim(&runtime, &");
                    out.push_str(value_temp);
                    out.push_str(", &");
                    out.push_str(&key_temp);
                    out.push_str(", \"");
                    out.push_str(&c_string(&self.source_file));
                    out.push_str("\", ");
                    out.push_str(&target.line().to_string());
                    out.push_str(");\n");
                    self.emit_bind_reference_target(out, target, &source_temp);
                    emit_value_cleanup(out, "    ", &source_temp);
                }
            }
            emit_value_cleanup(out, "    ", &key_temp);
        }

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone(");
        out.push_str(value_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_reference_list_assignment_from_variable(
        &mut self,
        out: &mut String,
        target: &ListAssignmentTarget,
        source_name: &str,
    ) -> String {
        let self_referential = list_assignment_references_variable(target, source_name);
        let mut entries = Vec::with_capacity(target.elements.len());
        let mut cleanup_temps = Vec::new();

        for (index, element) in target.elements.iter().enumerate() {
            let key_temp = self.emit_list_key(out, element, index);
            cleanup_temps.push(key_temp.clone());
            let value_temp = match &element.target {
                ListAssignmentElementTarget::Value(target) => {
                    let source_value_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&source_value_temp);
                    out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
                    out.push_str(&c_string(source_name));
                    out.push_str("\", \"");
                    out.push_str(&c_string(&self.source_file));
                    out.push_str("\", ");
                    out.push_str(&target.line().to_string());
                    out.push_str(");\n");
                    cleanup_temps.push(source_value_temp.clone());
                    let element_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&element_temp);
                    out.push_str(" = ptn_array_read_for_list_destructure(&runtime, ");
                    out.push_str(&source_value_temp);
                    out.push_str(", ");
                    out.push_str(&key_temp);
                    out.push_str(", ");
                    out.push_str(&target.line().to_string());
                    out.push_str(");\n");
                    let stored_temp =
                        self.emit_store_assignment_target_from_temp(out, target, &element_temp);
                    emit_value_cleanup(out, "    ", &stored_temp);
                    cleanup_temps.push(element_temp.clone());
                    element_temp
                }
                ListAssignmentElementTarget::Reference(target) => {
                    let source_temp = if self_referential {
                        self.emit_reference_target(out, target)
                    } else {
                        let temp = self.next_temp();
                        out.push_str("    PtnValue ");
                        out.push_str(&temp);
                        out.push_str(" = ptn_runtime_reference_for_array_dim(&runtime, \"");
                        out.push_str(&c_string(source_name));
                        out.push_str("\", &");
                        out.push_str(&key_temp);
                        out.push_str(", \"");
                        out.push_str(&c_string(&self.source_file));
                        out.push_str("\", ");
                        out.push_str(&target.line().to_string());
                        out.push_str(");\n");
                        temp
                    };
                    self.emit_bind_reference_target(out, target, &source_temp);
                    cleanup_temps.push(source_temp.clone());
                    source_temp
                }
            };
            entries.push(format!("{{ 0, {key_temp}, {value_temp} }}"));
        }

        let result_temp = self.next_temp();
        if self_referential {
            if entries.is_empty() {
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_array_from_literal_entries(0, NULL);\n");
            } else {
                let entries_temp = self.next_temp();
                out.push_str("    PtnArrayLiteralEntry ");
                out.push_str(&entries_temp);
                out.push_str("[] = { ");
                out.push_str(&entries.join(", "));
                out.push_str(" };\n");
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_array_from_literal_entries(");
                out.push_str(&entries.len().to_string());
                out.push_str(", ");
                out.push_str(&entries_temp);
                out.push_str(");\n");
            }
        } else {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
            out.push_str(&c_string(source_name));
            out.push_str("\", \"");
            out.push_str(&c_string(&self.source_file));
            out.push_str("\", ");
            out.push_str(&target.line.to_string());
            out.push_str(");\n");
        }

        for temp in cleanup_temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        result_temp
    }

    fn emit_list_key(
        &mut self,
        out: &mut String,
        element: &ListAssignmentElement,
        index: usize,
    ) -> String {
        match &element.key {
            Some(key) => self.emit_materialized_value(out, key),
            None => {
                let key_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&key_temp);
                out.push_str(" = ptn_int(");
                out.push_str(&index.to_string());
                out.push_str(");\n");
                key_temp
            }
        }
    }

    fn emit_bind_reference_target(
        &mut self,
        out: &mut String,
        target: &ReferenceTarget,
        reference_temp: &str,
    ) {
        match target {
            ReferenceTarget::Variable { name, .. } => {
                out.push_str("    ptn_runtime_bind_variable_reference(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(reference_temp);
                out.push_str(");\n");
            }
            ReferenceTarget::ArrayDim(target) => {
                let path = emit_array_path_segments(out, self, &target.dimensions);
                out.push_str("    ptn_runtime_bind_array_path_reference(&runtime, \"");
                out.push_str(&c_string(&target.array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(reference_temp);
                out.push_str(", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&target.line.to_string());
                out.push_str(");\n");
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
            }
            ReferenceTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                line,
            } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                let property_reference_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&property_reference_temp);
                out.push_str(" = ptn_object_reference_for_property(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let path = emit_array_path_segments(out, self, dimensions);
                out.push_str("    ptn_value_bind_array_path_reference(&runtime, &");
                out.push_str(&property_reference_temp);
                out.push_str(", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(reference_temp);
                out.push_str(", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
                emit_value_cleanup(out, "    ", &property_reference_temp);
                emit_value_cleanup(out, "    ", &receiver_temp);
            }
            ReferenceTarget::Property {
                receiver,
                name,
                line,
            } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                out.push_str("    ptn_object_bind_property_reference(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(reference_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &receiver_temp);
            }
        }
    }

    fn emit_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Binary {
                op,
                left,
                right,
                line,
            } => self.emit_binary(out, *op, left, right, *line),
            ValueExpr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => self.emit_ternary(out, condition, if_true.as_deref(), if_false),
            ValueExpr::InstanceOf {
                expr, class_name, ..
            } => self.emit_instanceof(out, expr, class_name),
            ValueExpr::Unary { op, expr, line } => {
                if matches!(op, UnaryOp::ErrorSuppress) {
                    let saved_temp = self.next_temp();
                    out.push_str("    int ");
                    out.push_str(&saved_temp);
                    out.push_str(" = runtime.diagnostics.suppressed;\n");
                    out.push_str("    runtime.diagnostics.suppressed++;\n");
                    let expr_temp = self.emit_materialized_value(out, expr);
                    out.push_str("    runtime.diagnostics.suppressed = ");
                    out.push_str(&saved_temp);
                    out.push_str(";\n");
                    return expr_temp;
                }
                let expr_temp = self.emit_materialized_value(out, expr);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(match op {
                    UnaryOp::Positive => "ptn_positive",
                    UnaryOp::Negate => "ptn_negate",
                    UnaryOp::Not => "ptn_not",
                    UnaryOp::BitwiseNot => "ptn_bitwise_not",
                    UnaryOp::ErrorSuppress => unreachable!(),
                });
                out.push('(');
                if matches!(op, UnaryOp::BitwiseNot) {
                    out.push_str("&runtime, ");
                }
                out.push_str(&expr_temp);
                if matches!(op, UnaryOp::BitwiseNot) {
                    out.push_str(", \"");
                    out.push_str(&c_string(&self.source_file));
                    out.push_str("\", ");
                    out.push_str(&line.to_string());
                }
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &expr_temp);
                result_temp
            }
            ValueExpr::Assign { target, op, value } => {
                self.emit_assignment(out, target, *op, value)
            }
            ValueExpr::AssignRef { target, source } => {
                self.emit_reference_assignment(out, target, source)
            }
            ValueExpr::Cast { kind, expr, line } => {
                let expr_temp = self.emit_materialized_value(out, expr);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                match kind {
                    CastKind::Int
                    | CastKind::Float
                    | CastKind::String
                    | CastKind::Bool
                    | CastKind::Array
                    | CastKind::Object => match kind {
                        CastKind::String => {
                            out.push_str("ptn_cast_string_with_runtime(&runtime, ");
                            out.push_str(&expr_temp);
                            out.push_str(", ");
                            out.push_str(&line.to_string());
                            out.push_str(");\n");
                        }
                        CastKind::Object => {
                            out.push_str("ptn_cast_object(&runtime, ");
                            out.push_str(&expr_temp);
                            out.push_str(");\n");
                        }
                        CastKind::Array => {
                            out.push_str("ptn_cast_array(");
                            out.push_str(&expr_temp);
                            out.push_str(");\n");
                        }
                        CastKind::Int | CastKind::Float | CastKind::Bool => {
                            out.push_str(match kind {
                                CastKind::Int => "ptn_cast_int",
                                CastKind::Float => "ptn_cast_float",
                                CastKind::Bool => "ptn_cast_bool",
                                CastKind::String
                                | CastKind::Array
                                | CastKind::Object
                                | CastKind::Integer
                                | CastKind::Double
                                | CastKind::Binary
                                | CastKind::Boolean => {
                                    unreachable!(
                                        "only int/float/bool canonical casts use this branch"
                                    )
                                }
                            });
                            out.push('(');
                            out.push_str(&expr_temp);
                            out.push_str(");\n");
                        }
                        CastKind::Integer
                        | CastKind::Double
                        | CastKind::Binary
                        | CastKind::Boolean => {
                            unreachable!("non-canonical casts are handled separately")
                        }
                    },
                    CastKind::Integer | CastKind::Double | CastKind::Binary | CastKind::Boolean => {
                        let (spelling, canonical, target) = match kind {
                            CastKind::Integer => ("integer", "int", "PTN_CAST_TARGET_INT"),
                            CastKind::Double => ("double", "float", "PTN_CAST_TARGET_FLOAT"),
                            CastKind::Binary => ("binary", "string", "PTN_CAST_TARGET_STRING"),
                            CastKind::Boolean => ("boolean", "bool", "PTN_CAST_TARGET_BOOL"),
                            CastKind::Int
                            | CastKind::Float
                            | CastKind::String
                            | CastKind::Bool
                            | CastKind::Array
                            | CastKind::Object => {
                                unreachable!("canonical casts are handled separately")
                            }
                        };
                        out.push_str("ptn_cast_noncanonical(&runtime, ");
                        out.push_str(&expr_temp);
                        out.push_str(", \"");
                        out.push_str(spelling);
                        out.push_str("\", \"");
                        out.push_str(canonical);
                        out.push_str("\", ");
                        out.push_str(target);
                        out.push_str(", ");
                        out.push_str(&line.to_string());
                        out.push_str(");\n");
                    }
                }
                emit_value_cleanup(out, "    ", &expr_temp);
                result_temp
            }
            ValueExpr::String(value) => {
                format!(
                    "ptn_string_literal(\"{}\", {})",
                    c_string(value),
                    php_string_byte_len(value)
                )
            }
            ValueExpr::Int(value) => format!("ptn_int({value})"),
            ValueExpr::Float(value) => format!("ptn_float({value:?})"),
            ValueExpr::Bool(true) => "ptn_bool(1)".to_string(),
            ValueExpr::Bool(false) => "ptn_bool(0)".to_string(),
            ValueExpr::Null => "ptn_null()".to_string(),
            ValueExpr::Closure {
                function_index,
                captures,
                ..
            } => self.emit_closure(out, *function_index, captures),
            ValueExpr::Array(elements) => self.emit_array(out, elements),
            ValueExpr::ArrayAccess { array, index, line } => {
                let array_temp = self.emit_materialized_value(out, array);
                let index_temp = self.emit_materialized_value(out, index);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_array_read(&runtime, ");
                out.push_str(&array_temp);
                out.push_str(", ");
                out.push_str(&index_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &array_temp);
                emit_value_cleanup(out, "    ", &index_temp);
                result_temp
            }
            ValueExpr::ArrayAppendAccess { line, .. } => {
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_null();\n");
                out.push_str("    ptn_abort_type_error_at(\"Cannot use [] for reading\", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                result_temp
            }
            ValueExpr::NewObject {
                class_name,
                arguments,
                argument_names,
                argument_unpacks,
                line,
            } => self.emit_new_object(
                out,
                class_name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            ),
            ValueExpr::DynamicNewObject {
                class_name,
                arguments,
                argument_names,
                argument_unpacks,
                line,
            } => self.emit_dynamic_new_object(
                out,
                class_name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            ),
            ValueExpr::Clone { expr, line } => {
                let expr_temp = self.emit_materialized_value(out, expr);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_clone_value(&runtime, ");
                out.push_str(&expr_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &expr_temp);
                result_temp
            }
            ValueExpr::PropertyFetch {
                receiver,
                name,
                line,
            } => self.emit_property_fetch(out, receiver, name, *line),
            ValueExpr::StaticPropertyFetch {
                class_name,
                name,
                line,
            } => self.emit_static_property_fetch(out, class_name, name, *line),
            ValueExpr::ClassConstantFetch {
                class_name,
                name,
                line,
            } => self.emit_class_constant_fetch(out, class_name, name, *line),
            ValueExpr::DynamicClassNameFetch { receiver, line } => {
                self.emit_dynamic_class_name_fetch(out, receiver, *line)
            }
            ValueExpr::Isset { targets } => self.emit_isset(out, targets),
            ValueExpr::Empty { target } => self.emit_empty(out, target),
            ValueExpr::Print { expression } => self.emit_print(out, expression),
            ValueExpr::Include {
                kind,
                path,
                candidates,
                line,
            } => self.emit_include(out, *kind, path, candidates, *line),
            ValueExpr::Throw { value, line } => self.emit_throw_value(out, value, *line),
            ValueExpr::Load { name, line } => format!(
                "ptn_runtime_read_variable(&runtime, \"{}\", \"{}\", {})",
                c_string(name),
                c_string(&self.source_file),
                line
            ),
            ValueExpr::LegacyDollarBraceStringVariable { name, line } => {
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                result_temp
            }
            ValueExpr::DynamicVariable { name, line } => {
                self.emit_dynamic_variable_read(out, name, *line)
            }
            ValueExpr::IncDec {
                target,
                op,
                result,
                line,
            } => self.emit_inc_dec_expression(out, target, *op, *result, *line),
            ValueExpr::Constant(name) => {
                format!("ptn_read_constant(&runtime, \"{}\")", c_string(name))
            }
            ValueExpr::MagicConstant { kind, line } => match kind {
                MagicConstantKind::Line => format!("ptn_int({line})"),
                MagicConstantKind::File => {
                    format!(
                        "ptn_string_literal(\"{}\", {})",
                        c_string(&self.source_file),
                        self.source_file.len()
                    )
                }
                MagicConstantKind::Dir => {
                    format!(
                        "ptn_string_literal(\"{}\", {})",
                        c_string(&self.source_dir),
                        self.source_dir.len()
                    )
                }
                MagicConstantKind::Function => {
                    format!(
                        "ptn_string(\"{}\")",
                        c_string(self.current_function_name.as_deref().unwrap_or(""))
                    )
                }
                MagicConstantKind::Method => {
                    format!(
                        "ptn_string(\"{}\")",
                        c_string(self.current_method_name.as_deref().unwrap_or(""))
                    )
                }
                MagicConstantKind::Class => {
                    format!(
                        "ptn_string(\"{}\")",
                        c_string(self.current_class_name.as_deref().unwrap_or(""))
                    )
                }
                MagicConstantKind::Trait | MagicConstantKind::Namespace => {
                    "ptn_string(\"\")".to_string()
                }
            },
            ValueExpr::InternalCall {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                line,
            } => self.emit_internal_call(
                out,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            ),
            ValueExpr::FirstClassCallable { callable, line } => {
                self.emit_first_class_callable(out, callable, *line)
            }
            ValueExpr::DynamicCall {
                callee,
                arguments,
                argument_names,
                argument_unpacks,
                line,
            } => self.emit_dynamic_call(
                out,
                callee,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            ),
            ValueExpr::MethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                line,
            } => self.emit_method_call(
                out,
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            ),
            ValueExpr::DynamicMethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                line,
            } => self.emit_dynamic_method_call(
                out,
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            ),
        }
    }

    fn emit_include(
        &mut self,
        out: &mut String,
        kind: IncludeKind,
        path: &ValueExpr,
        candidates: &[usize],
        line: usize,
    ) -> String {
        let path_temp = self.emit_materialized_value(out, path);
        let operand_temp = self.next_temp();
        out.push_str("    PtnStringOperand ");
        out.push_str(&operand_temp);
        out.push_str(" = ptn_value_to_string_operand_with_runtime(&runtime, ");
        out.push_str(&path_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        let resolved_temp = self.next_temp();
        out.push_str("    char *");
        out.push_str(&resolved_temp);
        out.push_str(" = ptn_include_resolve_path(\"");
        out.push_str(&c_string(&self.source_dir));
        out.push_str("\", ");
        out.push_str(&operand_temp);
        out.push_str(");\n");
        out.push_str("    ptn_string_operand_free(");
        out.push_str(&operand_temp);
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &path_temp);

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(0);\n");
        out.push_str("    if (");
        out.push_str(&resolved_temp);
        out.push_str(" == NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_compiled_include_path_error(&runtime, \"");
        out.push_str(include_kind_text(kind));
        out.push_str("\", ");
        out.push_str(&line.to_string());
        out.push_str(", ");
        out.push_str(if include_kind_is_required(kind) {
            "1"
        } else {
            "0"
        });
        out.push_str(");\n");
        for candidate in candidates {
            out.push_str("    } else if (");
            self.emit_include_candidate_condition(out, &resolved_temp, *candidate);
            out.push_str(") {\n");
            if include_kind_is_once(kind) {
                out.push_str("        if (ptn_include_seen[");
                out.push_str(&candidate.to_string());
                out.push_str("]) {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(1);\n");
                out.push_str("        } else {\n");
                out.push_str("            ptn_include_seen[");
                out.push_str(&candidate.to_string());
                out.push_str("] = 1;\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(&include_c_name(*candidate));
                out.push_str("(&runtime);\n");
                out.push_str("        }\n");
            } else {
                out.push_str("        ptn_include_seen[");
                out.push_str(&candidate.to_string());
                out.push_str("] = 1;\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(&include_c_name(*candidate));
                out.push_str("(&runtime);\n");
            }
        }
        out.push_str("    } else {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_compiled_include_failure(&runtime, \"");
        out.push_str(include_kind_text(kind));
        out.push_str("\", ");
        out.push_str(&resolved_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(", ");
        out.push_str(if include_kind_is_required(kind) {
            "1"
        } else {
            "0"
        });
        out.push_str(");\n");
        out.push_str("    }\n");
        out.push_str("    if (");
        out.push_str(&resolved_temp);
        out.push_str(" != NULL) {\n");
        out.push_str("        free(");
        out.push_str(&resolved_temp);
        out.push_str(");\n");
        out.push_str("    }\n");
        result_temp
    }

    fn emit_throw_value(&mut self, out: &mut String, value: &ValueExpr, line: usize) -> String {
        let value_temp = self.emit_materialized_value(out, value);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_throw_value(&runtime, ");
        out.push_str(&value_temp);
        out.push_str(", \"");
        out.push_str(&c_string(&self.source_file));
        out.push_str("\", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        result_temp
    }

    fn emit_include_candidate_condition(
        &self,
        out: &mut String,
        resolved_temp: &str,
        candidate: usize,
    ) {
        let include = &self.includes[candidate];
        if include.path_aliases.is_empty() {
            out.push('0');
            return;
        }
        for (index, alias) in include.path_aliases.iter().enumerate() {
            if index > 0 {
                out.push_str(" || ");
            }
            out.push_str("strcmp(");
            out.push_str(resolved_temp);
            out.push_str(", \"");
            out.push_str(&c_string(alias));
            out.push_str("\") == 0");
        }
    }

    fn emit_inc_dec_expression(
        &mut self,
        out: &mut String,
        target: &IncDecTarget,
        op: IncDecOp,
        result: IncDecResult,
        line: usize,
    ) -> String {
        match target {
            IncDecTarget::Variable { name, .. } => {
                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let old_temp = if matches!(result, IncDecResult::Post) {
                    let old_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&old_temp);
                    out.push_str(" = ptn_value_clone(ptn_value_deref(");
                    out.push_str(&current_temp);
                    out.push_str("));\n");
                    Some(old_temp)
                } else {
                    None
                };

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(inc_dec_runtime_function(op));
                out.push_str("(&runtime, ");
                out.push_str(&current_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&result_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &current_temp);

                if let Some(old_temp) = old_temp {
                    emit_value_cleanup(out, "    ", &result_temp);
                    old_temp
                } else {
                    result_temp
                }
            }
            IncDecTarget::DynamicVariable {
                name,
                line: target_line,
            } => {
                let name_temp = self.emit_dynamic_variable_name(out, name, *target_line);
                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_read_variable(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let old_temp = if matches!(result, IncDecResult::Post) {
                    let old_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&old_temp);
                    out.push_str(" = ptn_value_clone(ptn_value_deref(");
                    out.push_str(&current_temp);
                    out.push_str("));\n");
                    Some(old_temp)
                } else {
                    None
                };

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(inc_dec_runtime_function(op));
                out.push_str("(&runtime, ");
                out.push_str(&current_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    ptn_runtime_write_variable(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", ");
                out.push_str(&result_temp);
                out.push_str(");\n");
                out.push_str("    free(");
                out.push_str(&name_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &current_temp);

                if let Some(old_temp) = old_temp {
                    emit_value_cleanup(out, "    ", &result_temp);
                    old_temp
                } else {
                    result_temp
                }
            }
            IncDecTarget::DynamicArrayDim {
                name,
                dimensions,
                line: target_line,
            } => {
                let name_temp = self.emit_dynamic_variable_name(out, name, *target_line);
                out.push_str("    ptn_runtime_array_warn_missing_base_for_assign_op(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let path = emit_array_path_segments(out, self, dimensions);
                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_array_path_read_for_assign_op(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let old_temp = if matches!(result, IncDecResult::Post) {
                    let old_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&old_temp);
                    out.push_str(" = ptn_value_clone(ptn_value_deref(");
                    out.push_str(&current_temp);
                    out.push_str("));\n");
                    Some(old_temp)
                } else {
                    None
                };

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(inc_dec_runtime_function(op));
                out.push_str("(&runtime, ");
                out.push_str(&current_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    ptn_runtime_array_path_set_from_assign_op(&runtime, ");
                out.push_str(&name_temp);
                out.push_str(", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&result_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    free(");
                out.push_str(&name_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &current_temp);
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }

                if let Some(old_temp) = old_temp {
                    emit_value_cleanup(out, "    ", &result_temp);
                    old_temp
                } else {
                    result_temp
                }
            }
            IncDecTarget::ArrayDim {
                array, dimensions, ..
            } => {
                out.push_str("    ptn_runtime_array_warn_missing_base_for_assign_op(&runtime, \"");
                out.push_str(&c_string(array));
                out.push_str("\", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let path = emit_array_path_segments(out, self, dimensions);
                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_array_path_read_for_assign_op(&runtime, \"");
                out.push_str(&c_string(array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let old_temp = if matches!(result, IncDecResult::Post) {
                    let old_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&old_temp);
                    out.push_str(" = ptn_value_clone(ptn_value_deref(");
                    out.push_str(&current_temp);
                    out.push_str("));\n");
                    Some(old_temp)
                } else {
                    None
                };

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(inc_dec_runtime_function(op));
                out.push_str("(&runtime, ");
                out.push_str(&current_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    ptn_runtime_array_path_set_from_assign_op(&runtime, \"");
                out.push_str(&c_string(array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&result_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &current_temp);
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }

                if let Some(old_temp) = old_temp {
                    emit_value_cleanup(out, "    ", &result_temp);
                    old_temp
                } else {
                    result_temp
                }
            }
            IncDecTarget::Property { receiver, name, .. } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_object_read_property(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let old_temp = if matches!(result, IncDecResult::Post) {
                    let old_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&old_temp);
                    out.push_str(" = ptn_value_clone(ptn_value_deref(");
                    out.push_str(&current_temp);
                    out.push_str("));\n");
                    Some(old_temp)
                } else {
                    None
                };

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(inc_dec_runtime_function(op));
                out.push_str("(&runtime, ");
                out.push_str(&current_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let assigned_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&assigned_temp);
                out.push_str(" = ptn_object_write_property(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&result_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &assigned_temp);
                emit_value_cleanup(out, "    ", &current_temp);
                emit_value_cleanup(out, "    ", &receiver_temp);

                if let Some(old_temp) = old_temp {
                    emit_value_cleanup(out, "    ", &result_temp);
                    old_temp
                } else {
                    result_temp
                }
            }
            IncDecTarget::StaticProperty {
                class_name, name, ..
            } => {
                let resolved_class_name = self.static_property_class_name(class_name);
                let current_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&current_temp);
                out.push_str(" = ptn_runtime_read_static_property(&runtime, \"");
                out.push_str(&c_string(&resolved_class_name));
                out.push_str("\", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");

                let old_temp = if matches!(result, IncDecResult::Post) {
                    let old_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&old_temp);
                    out.push_str(" = ptn_value_clone(ptn_value_deref(");
                    out.push_str(&current_temp);
                    out.push_str("));\n");
                    Some(old_temp)
                } else {
                    None
                };

                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(inc_dec_runtime_function(op));
                out.push_str("(&runtime, ");
                out.push_str(&current_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let assigned_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&assigned_temp);
                out.push_str(" = ptn_runtime_write_static_property(&runtime, \"");
                out.push_str(&c_string(&resolved_class_name));
                out.push_str("\", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&result_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &assigned_temp);
                emit_value_cleanup(out, "    ", &current_temp);

                if let Some(old_temp) = old_temp {
                    emit_value_cleanup(out, "    ", &result_temp);
                    old_temp
                } else {
                    result_temp
                }
            }
        }
    }

    fn emit_closure(
        &mut self,
        out: &mut String,
        function_index: usize,
        captures: &[ClosureCapture],
    ) -> String {
        let closure_temp = self.next_temp();
        let function = &self.user_functions[function_index];
        let required_parameter_count = function
            .parameters
            .iter()
            .filter(|parameter| !parameter.is_variadic && parameter.default_value.is_none())
            .count();
        let is_variadic = function
            .parameters
            .iter()
            .any(|parameter| parameter.is_variadic);
        let parameter_names = emit_function_metadata_parameter_names(
            out,
            "    ",
            &format!("ptn_closure_{function_index}_parameter_names"),
            &function.parameters,
        );
        out.push_str("    PtnValue ");
        out.push_str(&closure_temp);
        out.push_str(" = ptn_closure(&runtime, ");
        out.push_str(&function_index.to_string());
        out.push_str(", \"");
        out.push_str(&c_string(&function.display_name));
        out.push_str("\", ptn_function_metadata_found(\"{closure}\", 0, ");
        out.push_str(&function.parameters.len().to_string());
        out.push_str(", ");
        out.push_str(&required_parameter_count.to_string());
        out.push_str(", ");
        out.push_str(if is_variadic { "1" } else { "0" });
        out.push_str(", ");
        out.push_str(&parameter_names);
        out.push_str("));\n");

        for capture in captures {
            let capture_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&capture_temp);
            if capture.by_ref {
                out.push_str(" = ptn_runtime_reference_for_variable(&runtime, \"");
                out.push_str(&c_string(&capture.name));
                out.push_str("\");\n");
                out.push_str("    ptn_closure_bind_capture_reference(");
            } else {
                if capture.warn_if_missing {
                    out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
                    out.push_str(&c_string(&capture.name));
                    out.push_str("\", \"");
                    out.push_str(&c_string(&self.source_file));
                    out.push_str("\", ");
                    out.push_str(&capture.line.to_string());
                    out.push_str(");\n");
                    out.push_str("    ptn_closure_set_capture(");
                } else {
                    out.push_str(";\n");
                    out.push_str("    PtnLookupResult ");
                    out.push_str(&capture_temp);
                    out.push_str("_lookup = ptn_runtime_read_variable_quiet(&runtime, \"");
                    out.push_str(&c_string(&capture.name));
                    out.push_str("\");\n");
                    out.push_str("    if (");
                    out.push_str(&capture_temp);
                    out.push_str("_lookup.exists) {\n");
                    out.push_str("        ");
                    out.push_str(&capture_temp);
                    out.push_str(" = ");
                    out.push_str(&capture_temp);
                    out.push_str("_lookup.value;\n");
                    out.push_str("        ptn_closure_set_capture(");
                }
            }
            out.push_str(&closure_temp);
            out.push_str(", \"");
            out.push_str(&c_string(&capture.name));
            out.push_str("\", ");
            out.push_str(&capture_temp);
            out.push_str(");\n");
            if capture.by_ref || capture.warn_if_missing {
                emit_value_cleanup(out, "    ", &capture_temp);
            } else {
                out.push_str("    }\n");
            }
        }

        closure_temp
    }

    fn emit_new_object(
        &mut self,
        out: &mut String,
        class_name: &str,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        argument_unpacks: &[bool],
        line: usize,
    ) -> String {
        if argument_names.iter().any(Option::is_some) {
            let result_temp = self.next_temp();
            self.emit_fatal_value(
                out,
                &result_temp,
                "named arguments currently support user-defined functions",
            );
            return result_temp;
        }
        let result_temp = self.next_temp();
        if let Some(declared_class) = self
            .classes
            .iter()
            .find(|class| class.name.eq_ignore_ascii_case(class_name))
            .cloned()
        {
            self.emit_declared_new_object(
                out,
                &result_temp,
                &declared_class,
                arguments,
                argument_unpacks,
                line,
                true,
            );
        } else {
            self.emit_runtime_new_object(
                out,
                &result_temp,
                class_name,
                arguments,
                argument_unpacks,
                line,
                true,
            );
        }
        result_temp
    }

    fn emit_dynamic_new_object(
        &mut self,
        out: &mut String,
        class_name: &ValueExpr,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        argument_unpacks: &[bool],
        line: usize,
    ) -> String {
        if argument_names.iter().any(Option::is_some) {
            let result_temp = self.next_temp();
            self.emit_fatal_value(
                out,
                &result_temp,
                "named arguments currently support user-defined functions",
            );
            return result_temp;
        }
        let class_value_temp = self.emit_materialized_value(out, class_name);
        let class_name_temp = self.next_temp();
        out.push_str("    char *");
        out.push_str(&class_name_temp);
        out.push_str(" = ptn_value_to_string(ptn_value_deref(");
        out.push_str(&class_value_temp);
        out.push_str("));\n");
        emit_value_cleanup(out, "    ", &class_value_temp);

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        let declared_classes = self.classes.clone();
        let mut emitted_branch = false;
        for declared_class in declared_classes {
            out.push_str("    ");
            if emitted_branch {
                out.push_str("} else ");
            }
            out.push_str("if (ptn_ascii_case_equal(");
            out.push_str(&class_name_temp);
            out.push_str(", \"");
            out.push_str(&c_string(&declared_class.name));
            out.push_str("\")) {\n");
            self.emit_declared_new_object(
                out,
                &result_temp,
                &declared_class,
                arguments,
                argument_unpacks,
                line,
                false,
            );
            emitted_branch = true;
        }
        if emitted_branch {
            out.push_str("    } else {\n");
            self.emit_runtime_new_object(
                out,
                &result_temp,
                &format!("{class_name_temp}"),
                arguments,
                argument_unpacks,
                line,
                false,
            );
            out.push_str("    }\n");
        } else {
            self.emit_runtime_new_object(
                out,
                &result_temp,
                &class_name_temp,
                arguments,
                argument_unpacks,
                line,
                false,
            );
        }
        out.push_str("    free(");
        out.push_str(&class_name_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_declared_new_object(
        &mut self,
        out: &mut String,
        result_temp: &str,
        declared_class: &ClassDecl,
        arguments: &[ValueExpr],
        argument_unpacks: &[bool],
        line: usize,
        declare_result: bool,
    ) {
        if declared_class.is_interface || declared_class.is_abstract {
            out.push_str("    ");
            if declare_result {
                out.push_str("PtnValue ");
            }
            out.push_str(result_temp);
            out.push_str(" = ptn_null();\n");
            out.push_str("    ptn_throw_exception(&runtime, \"Error\", \"Cannot instantiate ");
            out.push_str(if declared_class.is_interface {
                "interface "
            } else {
                "abstract class "
            });
            out.push_str(&c_string(&declared_class.name));
            out.push_str("\");\n");
            return;
        }
        out.push_str("    ");
        if declare_result {
            out.push_str("PtnValue ");
        }
        out.push_str(result_temp);
        out.push_str(" = ptn_object_new_shell(&runtime, \"");
        out.push_str(&c_string(&declared_class.name));
        out.push_str("\");\n");
        for (declaring_class_name, property) in
            class_property_initialization_chain(declared_class, &self.classes)
        {
            let value_temp = match &property.value {
                Some(value) => self.emit_materialized_value(out, value),
                None => {
                    let temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&temp);
                    out.push_str(" = ptn_null();\n");
                    temp
                }
            };
            let assigned_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&assigned_temp);
            out.push_str(" = ptn_object_declare_property(&runtime, ");
            out.push_str(result_temp);
            out.push_str(", \"");
            out.push_str(&c_string(&property.name));
            out.push_str("\", \"");
            out.push_str(&c_string(&declaring_class_name));
            out.push_str("\", ");
            out.push_str(c_property_visibility(property.visibility));
            out.push_str(", ");
            out.push_str(c_property_visibility(property.set_visibility));
            out.push_str(", ");
            out.push_str(if property.is_readonly { "1" } else { "0" });
            out.push_str(", ");
            out.push_str(if property.is_readonly && property.value.is_none() {
                "0"
            } else {
                "1"
            });
            out.push_str(", ");
            out.push_str(&value_temp);
            out.push_str(", ");
            out.push_str(&property.line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &assigned_temp);
            emit_value_cleanup(out, "    ", &value_temp);
        }
        if let Some(constructor_parameters) =
            class_constructor_method(declared_class, &self.classes).map(|constructor| {
                self.user_functions[constructor.function_index]
                    .parameters
                    .clone()
            })
        {
            let constructor_result = self.next_temp();
            if argument_unpacks.iter().any(|unpack| *unpack) {
                let args_temp = self.emit_call_arguments_builder(
                    out,
                    "__construct",
                    arguments,
                    argument_unpacks,
                    line,
                    true,
                );
                out.push_str("    PtnValue ");
                out.push_str(&constructor_result);
                out.push_str(" = ptn_call_declared_method(&runtime, ");
                out.push_str(result_temp);
                out.push_str(", \"__construct\", ");
                out.push_str(&args_temp);
                out.push_str(".len, ");
                out.push_str(&args_temp);
                out.push_str(".values, ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    ptn_call_arguments_destroy(&");
                out.push_str(&args_temp);
                out.push_str(");\n");
            } else if arguments.is_empty() {
                out.push_str("    PtnValue ");
                out.push_str(&constructor_result);
                out.push_str(" = ptn_call_declared_method(&runtime, ");
                out.push_str(result_temp);
                out.push_str(", \"__construct\", 0, NULL, ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            } else {
                let mut constructor_argument_temps = Vec::with_capacity(arguments.len());
                let mut unwrap_append_reference_temps = Vec::new();
                for (argument_index, argument) in arguments.iter().enumerate() {
                    let by_ref_parameter =
                        by_ref_parameter_for_argument(&constructor_parameters, argument_index);
                    if let Some(parameter) = by_ref_parameter {
                        let temp = self.emit_by_ref_call_argument(
                            out,
                            argument,
                            "__construct",
                            argument_index,
                            &parameter.name,
                            line,
                            true,
                            false,
                        );
                        if value_is_append_reference_target(argument) {
                            unwrap_append_reference_temps.push(temp.clone());
                        }
                        constructor_argument_temps.push(temp);
                    } else {
                        constructor_argument_temps.push(self.emit_call_argument(
                            out,
                            "__construct",
                            argument_index,
                            argument,
                        ));
                    }
                }
                let args_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&args_temp);
                out.push_str("[] = { ");
                for (index, temp) in constructor_argument_temps.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    out.push_str("ptn_value_share(");
                    out.push_str(temp);
                    out.push(')');
                }
                out.push_str(" };\n");
                out.push_str("    PtnValue ");
                out.push_str(&constructor_result);
                out.push_str(" = ptn_call_declared_method(&runtime, ");
                out.push_str(result_temp);
                out.push_str(", \"__construct\", ");
                out.push_str(&constructor_argument_temps.len().to_string());
                out.push_str(", ");
                out.push_str(&args_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                for temp in &unwrap_append_reference_temps {
                    emit_unwrap_append_reference_call_argument(out, "    ", temp);
                }
                for index in 0..constructor_argument_temps.len() {
                    emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
                }
                for temp in constructor_argument_temps {
                    emit_value_cleanup(out, "    ", &temp);
                }
            }
            emit_value_cleanup(out, "    ", &constructor_result);
        } else {
            if argument_unpacks.iter().any(|unpack| *unpack) {
                let args_temp = self.emit_call_arguments_builder(
                    out,
                    "__construct",
                    arguments,
                    argument_unpacks,
                    line,
                    true,
                );
                out.push_str("    ptn_call_arguments_destroy(&");
                out.push_str(&args_temp);
                out.push_str(");\n");
            } else {
                for argument in arguments {
                    let argument_temp = self.emit_materialized_value(out, argument);
                    emit_value_cleanup(out, "    ", &argument_temp);
                }
            }
        }
    }

    fn emit_runtime_new_object(
        &mut self,
        out: &mut String,
        result_temp: &str,
        class_name: &str,
        arguments: &[ValueExpr],
        argument_unpacks: &[bool],
        line: usize,
        declare_result: bool,
    ) {
        if argument_unpacks.iter().any(|unpack| *unpack) {
            let args_temp = self.emit_call_arguments_builder(
                out,
                "__construct",
                arguments,
                argument_unpacks,
                line,
                true,
            );
            out.push_str("    ");
            if declare_result {
                out.push_str("PtnValue ");
            }
            out.push_str(result_temp);
            out.push_str(" = ptn_new_object(&runtime, ");
            if declare_result {
                out.push('"');
                out.push_str(&c_string(class_name));
                out.push('"');
            } else {
                out.push_str(class_name);
            }
            out.push_str(", ");
            out.push_str(&args_temp);
            out.push_str(".len, ");
            out.push_str(&args_temp);
            out.push_str(".values, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_call_arguments_destroy(&");
            out.push_str(&args_temp);
            out.push_str(");\n");
            return;
        }

        let mut argument_temps = Vec::with_capacity(arguments.len());
        for argument in arguments {
            argument_temps.push(self.emit_materialized_value(out, argument));
        }
        if argument_temps.is_empty() {
            out.push_str("    ");
            if declare_result {
                out.push_str("PtnValue ");
            }
            out.push_str(result_temp);
            out.push_str(" = ptn_new_object(&runtime, ");
            if declare_result {
                out.push('"');
                out.push_str(&c_string(class_name));
                out.push('"');
            } else {
                out.push_str(class_name);
            }
            out.push_str(", 0, NULL, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
        } else {
            let args_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&args_temp);
            out.push_str("[] = { ");
            out.push_str(&argument_temps.join(", "));
            out.push_str(" };\n");
            out.push_str("    ");
            if declare_result {
                out.push_str("PtnValue ");
            }
            out.push_str(result_temp);
            out.push_str(" = ptn_new_object(&runtime, ");
            if declare_result {
                out.push('"');
                out.push_str(&c_string(class_name));
                out.push('"');
            } else {
                out.push_str(class_name);
            }
            out.push_str(", ");
            out.push_str(&argument_temps.len().to_string());
            out.push_str(", ");
            out.push_str(&args_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
        }
        for argument_temp in argument_temps {
            emit_value_cleanup(out, "    ", &argument_temp);
        }
    }

    fn emit_instanceof(&mut self, out: &mut String, expr: &ValueExpr, class_name: &str) -> String {
        let expr_temp = self.emit_materialized_value(out, expr);
        let resolved_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&resolved_temp);
        out.push_str(" = ptn_value_deref(");
        out.push_str(&expr_temp);
        out.push_str(");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(0);\n");
        out.push_str("    if (");
        out.push_str(&resolved_temp);
        out.push_str(".type == PTN_OBJECT) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(ptn_declared_class_is_same_or_descendant(");
        out.push_str(&resolved_temp);
        out.push_str(".as.object->class_name, \"");
        out.push_str(&c_string(class_name));
        out.push_str("\") || ptn_declared_class_implements_interface(");
        out.push_str(&resolved_temp);
        out.push_str(".as.object->class_name, \"");
        out.push_str(&c_string(class_name));
        out.push_str("\"));\n");
        out.push_str("    } else if (");
        out.push_str(&resolved_temp);
        out.push_str(".type == PTN_EXCEPTION) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(ptn_exception_type_matches_name(");
        out.push_str(&resolved_temp);
        out.push_str(".as.exception->class_name, \"");
        out.push_str(&c_string(class_name));
        out.push_str("\"));\n");
        out.push_str("    }\n");
        emit_value_cleanup(out, "    ", &expr_temp);
        result_temp
    }

    fn emit_print(&mut self, out: &mut String, expression: &ValueExpr) -> String {
        let expression_temp = self.emit_materialized_value(out, expression);
        out.push_str("    ptn_echo(&runtime, ");
        out.push_str(&expression_temp);
        out.push_str(", 0);\n");
        emit_value_cleanup(out, "    ", &expression_temp);
        "ptn_int(1)".to_string()
    }

    fn emit_property_fetch(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        line: usize,
    ) -> String {
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_object_read_property(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &receiver_temp);
        result_temp
    }

    fn emit_static_property_fetch(
        &mut self,
        out: &mut String,
        class_name: &str,
        name: &str,
        line: usize,
    ) -> String {
        let resolved_class_name = self.static_property_class_name(class_name);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_runtime_read_static_property(&runtime, \"");
        out.push_str(&c_string(&resolved_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        result_temp
    }

    fn emit_class_constant_fetch(
        &mut self,
        out: &mut String,
        class_name: &str,
        name: &str,
        line: usize,
    ) -> String {
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        if name.eq_ignore_ascii_case("class") {
            if let Some(message) = self.class_name_fetch_error_message(class_name) {
                out.push_str(" = ptn_null();\n");
                out.push_str("    ptn_throw_exception_at(&runtime, \"Error\", \"");
                out.push_str(&c_string(&message));
                out.push_str("\", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            } else if class_name.eq_ignore_ascii_case("static") {
                let fallback_class_name = self.class_name_fetch_name(class_name);
                out.push_str(";\n");
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str("_receiver = ptn_value_deref(receiver);\n");
                out.push_str("    if (");
                out.push_str(&result_temp);
                out.push_str("_receiver.type == PTN_STRING) {\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_value_clone_deref(");
                out.push_str(&result_temp);
                out.push_str("_receiver);\n");
                out.push_str("    } else if (");
                out.push_str(&result_temp);
                out.push_str("_receiver.type == PTN_OBJECT || ");
                out.push_str(&result_temp);
                out.push_str("_receiver.type == PTN_EXCEPTION || ");
                out.push_str(&result_temp);
                out.push_str("_receiver.type == PTN_CLOSURE) {\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_fetch_dynamic_class_name(&runtime, receiver, ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("    } else {\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_string(\"");
                out.push_str(&c_string(&fallback_class_name));
                out.push_str("\");\n");
                out.push_str("    }\n");
            } else {
                let resolved_class_name = self.class_name_fetch_name(class_name);
                out.push_str(" = ptn_string(\"");
                out.push_str(&c_string(&resolved_class_name));
                out.push_str("\");\n");
            }
        } else {
            let resolved_class_name = self.static_member_class_name(class_name);
            out.push_str(" = ptn_runtime_read_class_constant(&runtime, \"");
            out.push_str(&c_string(&resolved_class_name));
            out.push_str("\", \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
        }
        result_temp
    }

    fn emit_dynamic_class_name_fetch(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        line: usize,
    ) -> String {
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_runtime_fetch_dynamic_class_name(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &receiver_temp);
        result_temp
    }

    fn emit_static_property_lookup_quiet(
        &mut self,
        out: &mut String,
        class_name: &str,
        name: &str,
        line: usize,
    ) -> String {
        let resolved_class_name = self.static_property_class_name(class_name);
        let result_temp = self.next_temp();
        out.push_str("        PtnLookupResult ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_runtime_read_static_property_quiet(&runtime, \"");
        out.push_str(&c_string(&resolved_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        result_temp
    }

    fn emit_condition(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Bool(true) => "1".to_string(),
            ValueExpr::Bool(false) | ValueExpr::Null => "0".to_string(),
            ValueExpr::Int(value) => {
                if *value == 0 {
                    "0".to_string()
                } else {
                    "1".to_string()
                }
            }
            ValueExpr::Float(value) => {
                if *value == 0.0 {
                    "0".to_string()
                } else {
                    "1".to_string()
                }
            }
            ValueExpr::Unary {
                op: UnaryOp::Not,
                expr,
                ..
            } => {
                let predicate = self.emit_condition(out, expr);
                format!("!({predicate})")
            }
            ValueExpr::Binary {
                op, left, right, ..
            } => match op {
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Identical
                | BinaryOp::NotIdentical
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => self.emit_comparison_predicate(out, *op, left, right),
                BinaryOp::And | BinaryOp::Or => {
                    self.emit_short_circuit_condition(out, *op, left, right)
                }
                BinaryOp::Xor => self.emit_boolean_xor_condition(out, left, right),
                _ => {
                    let emitted_value = self.emit_materialized_value(out, value);
                    let result_temp = self.next_temp();
                    out.push_str("    int ");
                    out.push_str(&result_temp);
                    out.push_str(" = ptn_is_truthy(");
                    out.push_str(&emitted_value);
                    out.push_str(");\n");
                    emit_value_cleanup(out, "    ", &emitted_value);
                    result_temp
                }
            },
            _ => {
                let emitted_value = self.emit_materialized_value(out, value);
                let result_temp = self.next_temp();
                out.push_str("    int ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_is_truthy(");
                out.push_str(&emitted_value);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &emitted_value);
                result_temp
            }
        }
    }

    fn emit_binary(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
        line: usize,
    ) -> String {
        match op {
            BinaryOp::Concat => self.emit_concat(out, left, right, line),
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Power
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseXor
            | BinaryOp::BitwiseOr
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => self.emit_runtime_binary(out, op, left, right, line),
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Identical
            | BinaryOp::NotIdentical
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => self.emit_comparison(out, op, left, right),
            BinaryOp::Spaceship => self.emit_spaceship(out, left, right),
            BinaryOp::Xor => self.emit_boolean_xor(out, left, right),
            BinaryOp::And | BinaryOp::Or => self.emit_short_circuit(out, op, left, right),
            BinaryOp::Coalesce => self.emit_coalesce(out, left, right),
        }
    }

    fn emit_ternary(
        &mut self,
        out: &mut String,
        condition: &ValueExpr,
        if_true: Option<&ValueExpr>,
        if_false: &ValueExpr,
    ) -> String {
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");

        if let Some(if_true) = if_true {
            let predicate = self.emit_condition(out, condition);
            out.push_str("    if (");
            out.push_str(&predicate);
            out.push_str(") {\n");
            let true_temp = self.emit_materialized_value(out, if_true);
            out.push_str("        ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str(&true_temp);
            out.push_str(";\n");
            out.push_str("    } else {\n");
            let false_temp = self.emit_materialized_value(out, if_false);
            out.push_str("        ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str(&false_temp);
            out.push_str(";\n");
            out.push_str("    }\n");
            return result_temp;
        }

        let condition_temp = self.emit_materialized_value(out, condition);
        let predicate_temp = self.next_temp();
        out.push_str("    int ");
        out.push_str(&predicate_temp);
        out.push_str(" = ptn_is_truthy(");
        out.push_str(&condition_temp);
        out.push_str(");\n");
        out.push_str("    if (");
        out.push_str(&predicate_temp);
        out.push_str(") {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&condition_temp);
        out.push_str(";\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &condition_temp);
        let false_temp = self.emit_materialized_value(out, if_false);
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&false_temp);
        out.push_str(";\n");
        out.push_str("    }\n");
        result_temp
    }

    fn emit_runtime_binary(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
        line: usize,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(binary_runtime_function(op));
        out.push('(');
        if binary_runtime_function_uses_context(op) {
            out.push_str("&runtime, ");
        }
        out.push_str(&left_temp);
        out.push_str(", ");
        out.push_str(&right_temp);
        if binary_runtime_function_uses_context(op) {
            out.push_str(", ");
            out.push_str(&line.to_string());
        }
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &left_temp);
        emit_value_cleanup(out, "    ", &right_temp);
        result_temp
    }

    fn emit_concat(
        &mut self,
        out: &mut String,
        left: &ValueExpr,
        right: &ValueExpr,
        line: usize,
    ) -> String {
        let mut operands = Vec::new();
        collect_concat_operands(left, line, &mut operands);
        collect_concat_operands(right, line, &mut operands);

        let mut emitted_operands = Vec::with_capacity(operands.len());
        for operand in operands {
            let value_temp = self.emit_materialized_value(out, operand.value);
            emitted_operands.push((value_temp, operand.line));
        }

        let operands_temp = self.next_temp();
        out.push_str("    PtnConcatOperand ");
        out.push_str(&operands_temp);
        out.push_str("[] = { ");
        for (index, (value_temp, operand_line)) in emitted_operands.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("{ ");
            out.push_str(value_temp);
            out.push_str(", ");
            out.push_str(&operand_line.to_string());
            out.push_str(" }");
        }
        out.push_str(" };\n");

        let strings_temp = self.next_temp();
        out.push_str("    PtnStringOperand ");
        out.push_str(&strings_temp);
        out.push('[');
        out.push_str(&emitted_operands.len().to_string());
        out.push_str("];\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_concat_many(&runtime, ");
        out.push_str(&operands_temp);
        out.push_str(", ");
        out.push_str(&emitted_operands.len().to_string());
        out.push_str(", ");
        out.push_str(&strings_temp);
        out.push_str(");\n");
        for (value_temp, _) in emitted_operands {
            emit_value_cleanup(out, "    ", &value_temp);
        }
        result_temp
    }

    fn emit_spaceship(&mut self, out: &mut String, left: &ValueExpr, right: &ValueExpr) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_int(ptn_compare_spaceship(");
        out.push_str(&left_temp);
        out.push_str(", ");
        out.push_str(&right_temp);
        out.push_str("));\n");
        emit_value_cleanup(out, "    ", &left_temp);
        emit_value_cleanup(out, "    ", &right_temp);
        result_temp
    }

    fn emit_coalesce(&mut self, out: &mut String, left: &ValueExpr, right: &ValueExpr) -> String {
        let left_lookup = self.emit_quiet_lookup(out, left);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&left_lookup);
        out.push_str(".exists && ");
        out.push_str(&left_lookup);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&left_lookup);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{left_lookup}.value"));
        let right_temp = self.emit_materialized_value(out, right);
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&right_temp);
        out.push_str(";\n");
        out.push_str("    }\n");
        result_temp
    }

    fn emit_coalesce_assignment(
        &mut self,
        out: &mut String,
        name: &str,
        value: &ValueExpr,
    ) -> String {
        let lookup_temp = self.next_temp();
        out.push_str("    PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_runtime_read_variable_quiet(&runtime, \"");
        out.push_str(&c_string(name));
        out.push_str("\");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&lookup_temp);
        out.push_str(".exists && ");
        out.push_str(&lookup_temp);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&lookup_temp);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
        let value_temp = self.emit_materialized_value(out, value);
        out.push_str("        ptn_runtime_write_variable(&runtime, \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&value_temp);
        out.push_str(");\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone_deref(");
        out.push_str(&value_temp);
        out.push_str(");\n");
        emit_value_cleanup(out, "        ", &value_temp);
        out.push_str("    }\n");
        result_temp
    }

    fn emit_dynamic_coalesce_assignment(
        &mut self,
        out: &mut String,
        name: &ValueExpr,
        line: usize,
        value: &ValueExpr,
    ) -> String {
        let name_temp = self.emit_dynamic_variable_name(out, name, line);
        let lookup_temp = self.next_temp();
        out.push_str("    PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_runtime_read_variable_quiet(&runtime, ");
        out.push_str(&name_temp);
        out.push_str(");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&lookup_temp);
        out.push_str(".exists && ");
        out.push_str(&lookup_temp);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&lookup_temp);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
        let value_temp = self.emit_materialized_value(out, value);
        out.push_str("        ptn_runtime_write_variable(&runtime, ");
        out.push_str(&name_temp);
        out.push_str(", ");
        out.push_str(&value_temp);
        out.push_str(");\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone_deref(");
        out.push_str(&value_temp);
        out.push_str(");\n");
        emit_value_cleanup(out, "        ", &value_temp);
        out.push_str("    }\n");
        out.push_str("    free(");
        out.push_str(&name_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_offset_coalesce_assignment(
        &mut self,
        out: &mut String,
        array: &str,
        dimensions: &[Option<ValueExpr>],
        line: usize,
        value: &ValueExpr,
    ) -> String {
        let path = emit_array_path_segments(out, self, dimensions);
        let lookup_temp = self.next_temp();
        out.push_str("    PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_runtime_array_path_lookup_quiet(&runtime, \"");
        out.push_str(&c_string(array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&lookup_temp);
        out.push_str(".exists && ");
        out.push_str(&lookup_temp);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&lookup_temp);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
        let value_temp = self.emit_materialized_value(out, value);
        let snapshot_temp = self.next_temp();
        out.push_str("        PtnValue ");
        out.push_str(&snapshot_temp);
        out.push_str(" = ptn_value_snapshot_for_array_path_write(");
        out.push_str(&value_temp);
        out.push_str(");\n");
        out.push_str("        ptn_runtime_array_path_set(&runtime, \"");
        out.push_str(&c_string(array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&snapshot_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        let stored_lookup_temp = self.next_temp();
        out.push_str("        PtnLookupResult ");
        out.push_str(&stored_lookup_temp);
        out.push_str(" = ptn_runtime_array_path_lookup_quiet(&runtime, \"");
        out.push_str(&c_string(array));
        out.push_str("\", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        out.push_str("        if (");
        out.push_str(&stored_lookup_temp);
        out.push_str(".exists) {\n");
        out.push_str("            ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&stored_lookup_temp);
        out.push_str(".value;\n");
        out.push_str("        } else {\n");
        emit_value_cleanup(out, "            ", &format!("{stored_lookup_temp}.value"));
        out.push_str("            ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone(");
        out.push_str(&snapshot_temp);
        out.push_str(");\n");
        out.push_str("        }\n");
        emit_value_cleanup(out, "        ", &snapshot_temp);
        emit_value_cleanup(out, "        ", &value_temp);
        out.push_str("    }\n");
        for segment_temp in path.value_temps {
            emit_value_cleanup(out, "    ", &segment_temp);
        }
        result_temp
    }

    fn emit_dynamic_offset_coalesce_assignment(
        &mut self,
        out: &mut String,
        name: &ValueExpr,
        dimensions: &[Option<ValueExpr>],
        line: usize,
        value: &ValueExpr,
    ) -> String {
        let name_temp = self.emit_dynamic_variable_name(out, name, line);
        let path = emit_array_path_segments(out, self, dimensions);
        let lookup_temp = self.next_temp();
        out.push_str("    PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_runtime_array_path_lookup_quiet(&runtime, ");
        out.push_str(&name_temp);
        out.push_str(", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&lookup_temp);
        out.push_str(".exists && ");
        out.push_str(&lookup_temp);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&lookup_temp);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
        let value_temp = self.emit_materialized_value(out, value);
        let snapshot_temp = self.next_temp();
        out.push_str("        PtnValue ");
        out.push_str(&snapshot_temp);
        out.push_str(" = ptn_value_snapshot_for_array_path_write(");
        out.push_str(&value_temp);
        out.push_str(");\n");
        out.push_str("        ptn_runtime_array_path_set(&runtime, ");
        out.push_str(&name_temp);
        out.push_str(", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&snapshot_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        let stored_lookup_temp = self.next_temp();
        out.push_str("        PtnLookupResult ");
        out.push_str(&stored_lookup_temp);
        out.push_str(" = ptn_runtime_array_path_lookup_quiet(&runtime, ");
        out.push_str(&name_temp);
        out.push_str(", ");
        out.push_str(&path.name);
        out.push_str(", ");
        out.push_str(&path.len.to_string());
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        out.push_str("        if (");
        out.push_str(&stored_lookup_temp);
        out.push_str(".exists) {\n");
        out.push_str("            ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&stored_lookup_temp);
        out.push_str(".value;\n");
        out.push_str("        } else {\n");
        emit_value_cleanup(out, "            ", &format!("{stored_lookup_temp}.value"));
        out.push_str("            ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_value_clone(");
        out.push_str(&snapshot_temp);
        out.push_str(");\n");
        out.push_str("        }\n");
        emit_value_cleanup(out, "        ", &snapshot_temp);
        emit_value_cleanup(out, "        ", &value_temp);
        out.push_str("    }\n");
        out.push_str("    free(");
        out.push_str(&name_temp);
        out.push_str(");\n");
        for segment_temp in path.value_temps {
            emit_value_cleanup(out, "    ", &segment_temp);
        }
        result_temp
    }

    fn emit_property_coalesce_assignment(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        line: usize,
        value: &ValueExpr,
    ) -> String {
        let lookup_receiver_temp = self.emit_materialized_value(out, receiver);
        let lookup_temp = self.next_temp();
        out.push_str("    PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_object_property_lookup_quiet(&runtime, ");
        out.push_str(&lookup_receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &lookup_receiver_temp);

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&lookup_temp);
        out.push_str(".exists && ");
        out.push_str(&lookup_temp);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&lookup_temp);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
        let value_temp = self.emit_materialized_value(out, value);
        let write_receiver_temp = self.emit_materialized_value(out, receiver);
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_object_write_property(&runtime, ");
        out.push_str(&write_receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&value_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "        ", &write_receiver_temp);
        emit_value_cleanup(out, "        ", &value_temp);
        out.push_str("    }\n");
        result_temp
    }

    fn emit_static_property_coalesce_assignment(
        &mut self,
        out: &mut String,
        class_name: &str,
        name: &str,
        line: usize,
        value: &ValueExpr,
    ) -> String {
        let resolved_class_name = self.static_property_class_name(class_name);
        let lookup_temp = self.next_temp();
        out.push_str("    PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_runtime_read_static_property_quiet(&runtime, \"");
        out.push_str(&c_string(&resolved_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&lookup_temp);
        out.push_str(".exists && ");
        out.push_str(&lookup_temp);
        out.push_str(".value.type != PTN_NULL) {\n");
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&lookup_temp);
        out.push_str(".value;\n");
        out.push_str("    } else {\n");
        emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
        let value_temp = self.emit_materialized_value(out, value);
        out.push_str("        ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_runtime_write_static_property(&runtime, \"");
        out.push_str(&c_string(&resolved_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&value_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        emit_value_cleanup(out, "        ", &value_temp);
        out.push_str("    }\n");
        result_temp
    }

    fn emit_property_probe_quiet(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        line: usize,
    ) -> String {
        let receiver_lookup_temp = self.emit_quiet_lookup(out, receiver);
        let lookup_temp = self.next_temp();
        out.push_str("        PtnLookupResult ");
        out.push_str(&lookup_temp);
        out.push_str(";\n");
        out.push_str("        if (");
        out.push_str(&receiver_lookup_temp);
        out.push_str(".exists) {\n");
        out.push_str("            ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_object_property_probe_quiet(&runtime, ");
        out.push_str(&receiver_lookup_temp);
        out.push_str(".value, \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&c_optional_string(self.current_class_name.as_deref()));
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        out.push_str("        } else {\n");
        out.push_str("            ");
        out.push_str(&lookup_temp);
        out.push_str(" = ptn_lookup_missing();\n");
        out.push_str("        }\n");
        emit_value_cleanup(out, "        ", &format!("{receiver_lookup_temp}.value"));
        lookup_temp
    }

    fn emit_comparison(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        let comparison = match op {
            BinaryOp::Equal => format!("ptn_compare_equal({left_temp}, {right_temp})"),
            BinaryOp::NotEqual => format!("!ptn_compare_equal({left_temp}, {right_temp})"),
            BinaryOp::Identical => format!("ptn_compare_identical({left_temp}, {right_temp})"),
            BinaryOp::NotIdentical => {
                format!("ptn_compare_not_identical({left_temp}, {right_temp})")
            }
            BinaryOp::Less => format!("ptn_compare_less({left_temp}, {right_temp})"),
            BinaryOp::LessEqual => format!("ptn_compare_less_equal({left_temp}, {right_temp})"),
            BinaryOp::Greater => format!("ptn_compare_greater({left_temp}, {right_temp})"),
            BinaryOp::GreaterEqual => {
                format!("ptn_compare_greater_equal({left_temp}, {right_temp})")
            }
            _ => unreachable!(),
        };
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(");
        out.push_str(&comparison);
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &left_temp);
        emit_value_cleanup(out, "    ", &right_temp);
        result_temp
    }

    fn emit_comparison_predicate(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let predicate = match op {
            BinaryOp::Equal => format!("ptn_compare_equal({left_temp}, {right_temp})"),
            BinaryOp::NotEqual => format!("!ptn_compare_equal({left_temp}, {right_temp})"),
            BinaryOp::Identical => format!("ptn_compare_identical({left_temp}, {right_temp})"),
            BinaryOp::NotIdentical => {
                format!("ptn_compare_not_identical({left_temp}, {right_temp})")
            }
            BinaryOp::Less => format!("ptn_compare_less({left_temp}, {right_temp})"),
            BinaryOp::LessEqual => format!("ptn_compare_less_equal({left_temp}, {right_temp})"),
            BinaryOp::Greater => format!("ptn_compare_greater({left_temp}, {right_temp})"),
            BinaryOp::GreaterEqual => {
                format!("ptn_compare_greater_equal({left_temp}, {right_temp})")
            }
            _ => unreachable!(),
        };
        let result_temp = self.next_temp();
        out.push_str("    int ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(&predicate);
        out.push_str(";\n");
        emit_value_cleanup(out, "    ", &left_temp);
        emit_value_cleanup(out, "    ", &right_temp);
        result_temp
    }

    fn emit_isset(&mut self, out: &mut String, targets: &[ValueExpr]) -> String {
        let state_temp = self.next_temp();
        out.push_str("    int ");
        out.push_str(&state_temp);
        out.push_str(" = 1;\n");
        for target in targets {
            out.push_str("    if (");
            out.push_str(&state_temp);
            out.push_str(") {\n");
            let check_temp = self.emit_isset_check(out, target);
            out.push_str("        ");
            out.push_str(&state_temp);
            out.push_str(" = ");
            out.push_str(&check_temp);
            out.push_str(";\n");
            out.push_str("    }\n");
        }
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(");
        out.push_str(&state_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_empty(&mut self, out: &mut String, target: &ValueExpr) -> String {
        let check_temp = self.emit_empty_check(out, target);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(");
        out.push_str(&check_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_isset_check(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Load { name, .. } => {
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_variable_is_set(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\");\n");
                result_temp
            }
            ValueExpr::ArrayAccess { array, index, line } => {
                let container_temp = self.emit_quiet_lookup(out, array);
                let index_temp = self.emit_materialized_value(out, index);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = 0;\n");
                out.push_str("        if (");
                out.push_str(&container_temp);
                out.push_str(".exists) {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_offset_is_set(&runtime, ");
                out.push_str(&container_temp);
                out.push_str(".value, ");
                out.push_str(&index_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("        }\n");
                emit_value_cleanup(out, "        ", &format!("{container_temp}.value"));
                emit_value_cleanup(out, "        ", &index_temp);
                result_temp
            }
            ValueExpr::PropertyFetch {
                receiver,
                name,
                line,
            } => {
                let lookup_temp = self.emit_property_probe_quiet(out, receiver, name, *line);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(&lookup_temp);
                out.push_str(".exists && ");
                out.push_str(&lookup_temp);
                out.push_str(".value.type != PTN_NULL;\n");
                emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
                result_temp
            }
            ValueExpr::StaticPropertyFetch {
                class_name,
                name,
                line,
            } => {
                let lookup_temp =
                    self.emit_static_property_lookup_quiet(out, class_name, name, *line);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(&lookup_temp);
                out.push_str(".exists && ");
                out.push_str(&lookup_temp);
                out.push_str(".value.type != PTN_NULL;\n");
                emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
                result_temp
            }
            _ => {
                let value_temp = self.emit_materialized_value(out, value);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(&value_temp);
                out.push_str(".type != PTN_NULL;\n");
                emit_value_cleanup(out, "        ", &value_temp);
                result_temp
            }
        }
    }

    fn emit_empty_check(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Load { name, .. } => {
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_variable_is_empty(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\");\n");
                result_temp
            }
            ValueExpr::ArrayAccess { array, index, line } => {
                let container_temp = self.emit_quiet_lookup(out, array);
                let index_temp = self.emit_materialized_value(out, index);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = 1;\n");
                out.push_str("        if (");
                out.push_str(&container_temp);
                out.push_str(".exists) {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_offset_is_empty(&runtime, ");
                out.push_str(&container_temp);
                out.push_str(".value, ");
                out.push_str(&index_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                out.push_str("        }\n");
                emit_value_cleanup(out, "        ", &format!("{container_temp}.value"));
                emit_value_cleanup(out, "        ", &index_temp);
                result_temp
            }
            ValueExpr::PropertyFetch {
                receiver,
                name,
                line,
            } => {
                let lookup_temp = self.emit_property_probe_quiet(out, receiver, name, *line);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = 1;\n");
                out.push_str("        if (");
                out.push_str(&lookup_temp);
                out.push_str(".exists) {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = !ptn_is_truthy(");
                out.push_str(&lookup_temp);
                out.push_str(".value);\n");
                out.push_str("        }\n");
                emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
                result_temp
            }
            ValueExpr::StaticPropertyFetch {
                class_name,
                name,
                line,
            } => {
                let lookup_temp =
                    self.emit_static_property_lookup_quiet(out, class_name, name, *line);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = 1;\n");
                out.push_str("        if (");
                out.push_str(&lookup_temp);
                out.push_str(".exists) {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = !ptn_is_truthy(");
                out.push_str(&lookup_temp);
                out.push_str(".value);\n");
                out.push_str("        }\n");
                emit_value_cleanup(out, "        ", &format!("{lookup_temp}.value"));
                result_temp
            }
            _ => {
                let value_temp = self.emit_materialized_value(out, value);
                let result_temp = self.next_temp();
                out.push_str("        int ");
                out.push_str(&result_temp);
                out.push_str(" = !ptn_is_truthy(");
                out.push_str(&value_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "        ", &value_temp);
                result_temp
            }
        }
    }

    fn emit_quiet_lookup(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Load { name, .. } => {
                let result_temp = self.next_temp();
                out.push_str("        PtnLookupResult ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_read_variable_quiet(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\");\n");
                result_temp
            }
            ValueExpr::ArrayAccess { array, index, line } => {
                let container_temp = self.emit_quiet_lookup(out, array);
                let index_temp = self.emit_materialized_value(out, index);
                let result_temp = self.next_temp();
                out.push_str("        PtnLookupResult ");
                out.push_str(&result_temp);
                out.push_str(";\n");
                out.push_str("        if (");
                out.push_str(&container_temp);
                out.push_str(".exists) {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_offset_lookup(&runtime, ");
                out.push_str(&container_temp);
                out.push_str(".value, ");
                out.push_str(&index_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(", 1);\n");
                out.push_str("        } else {\n");
                out.push_str("            ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_lookup_missing();\n");
                out.push_str("        }\n");
                emit_value_cleanup(out, "        ", &format!("{container_temp}.value"));
                emit_value_cleanup(out, "        ", &index_temp);
                result_temp
            }
            ValueExpr::PropertyFetch {
                receiver,
                name,
                line,
            } => self.emit_property_probe_quiet(out, receiver, name, *line),
            ValueExpr::StaticPropertyFetch {
                class_name,
                name,
                line,
            } => self.emit_static_property_lookup_quiet(out, class_name, name, *line),
            _ => {
                let value_temp = self.emit_materialized_value(out, value);
                let result_temp = self.next_temp();
                out.push_str("        PtnLookupResult ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_lookup_found(");
                out.push_str(&value_temp);
                out.push_str(");\n");
                result_temp
            }
        }
    }

    fn emit_array(&mut self, out: &mut String, elements: &[IrArrayElement]) -> String {
        let result_temp = self.next_temp();
        if elements.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_array_from_literal_entries(0, NULL);\n");
            return result_temp;
        }

        if elements
            .iter()
            .any(|element| matches!(element.value, IrArrayElementValue::Unpack { .. }))
        {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_array_from_literal_entries(0, NULL);\n");
            for element in elements {
                match &element.value {
                    IrArrayElementValue::Value(_) | IrArrayElementValue::Reference(_) => {
                        let (has_key, key_temp) = if let Some(key) = &element.key {
                            let key_temp = self.emit_materialized_value(out, key);
                            ("1", key_temp)
                        } else {
                            ("0", "ptn_null()".to_string())
                        };
                        let value_temp = match &element.value {
                            IrArrayElementValue::Value(value) => {
                                self.emit_materialized_value(out, value)
                            }
                            IrArrayElementValue::Reference(target) => {
                                self.emit_reference_target(out, target)
                            }
                            IrArrayElementValue::Unpack { .. } => unreachable!(),
                        };
                        out.push_str("    ptn_array_literal_append_entry(&runtime, ");
                        out.push_str(&result_temp);
                        out.push_str(".as.array, runtime.call_site_line, ");
                        out.push_str(has_key);
                        out.push_str(", ");
                        out.push_str(&key_temp);
                        out.push_str(", ");
                        out.push_str(&value_temp);
                        out.push_str(");\n");
                        if element.key.is_some() {
                            emit_value_cleanup(out, "    ", &key_temp);
                        }
                        emit_value_cleanup(out, "    ", &value_temp);
                    }
                    IrArrayElementValue::Unpack { value, line } => {
                        if self.in_const_declaration
                            && const_array_unpack_operand_short_circuits(value)
                        {
                            out.push_str("    ptn_array_unpack_const_invalid(&runtime, ");
                            out.push_str(&line.to_string());
                            out.push_str(");\n");
                            continue;
                        }
                        let value_temp = self.emit_materialized_value(out, value);
                        let unpack_fn = if self.in_const_declaration {
                            "ptn_array_unpack_const_into"
                        } else if unpack_requires_literal_fatal(value) {
                            "ptn_array_unpack_into_or_fatal"
                        } else {
                            "ptn_array_unpack_into"
                        };
                        out.push_str("    ");
                        out.push_str(unpack_fn);
                        out.push_str("(&runtime, ");
                        out.push_str(&result_temp);
                        out.push_str(".as.array, ");
                        out.push_str(&value_temp);
                        out.push_str(", ");
                        out.push_str(&line.to_string());
                        out.push_str(");\n");
                        emit_value_cleanup(out, "    ", &value_temp);
                    }
                }
            }
            return result_temp;
        }

        let mut entries = Vec::with_capacity(elements.len());
        let mut entry_temps = Vec::with_capacity(elements.len() * 2);
        for element in elements {
            let (has_key, key_temp) = if let Some(key) = &element.key {
                let key_temp = self.emit_materialized_value(out, key);
                entry_temps.push(key_temp.clone());
                ("1", key_temp)
            } else {
                ("0", "ptn_null()".to_string())
            };
            let value_temp = match &element.value {
                IrArrayElementValue::Value(value) => self.emit_materialized_value(out, value),
                IrArrayElementValue::Reference(target) => self.emit_reference_target(out, target),
                IrArrayElementValue::Unpack { .. } => unreachable!(),
            };
            entry_temps.push(value_temp.clone());
            entries.push(format!("{{ {has_key}, {key_temp}, {value_temp} }}"));
        }

        let entries_temp = self.next_temp();
        out.push_str("    PtnArrayLiteralEntry ");
        out.push_str(&entries_temp);
        out.push_str("[] = { ");
        out.push_str(&entries.join(", "));
        out.push_str(" };\n");
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_array_from_literal_entries_at(&runtime, runtime.call_site_line, ");
        out.push_str(&elements.len().to_string());
        out.push_str(", ");
        out.push_str(&entries_temp);
        out.push_str(");\n");
        for temp in entry_temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        result_temp
    }

    fn emit_short_circuit(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_predicate = self.emit_condition(out, left);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (");
        out.push_str(&left_predicate);
        out.push_str(") {\n");
        match op {
            BinaryOp::And => {
                let right_predicate = self.emit_condition(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(");
                out.push_str(&right_predicate);
                out.push_str(");\n");
                out.push_str("    } else {\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(0);\n");
            }
            BinaryOp::Or => {
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(1);\n");
                out.push_str("    } else {\n");
                let right_predicate = self.emit_condition(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(");
                out.push_str(&right_predicate);
                out.push_str(");\n");
            }
            _ => unreachable!(),
        }
        out.push_str("    }\n");
        result_temp
    }

    fn emit_short_circuit_condition(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_predicate = self.emit_condition(out, left);
        let result_temp = self.next_temp();
        out.push_str("    int ");
        out.push_str(&result_temp);
        out.push_str(" = 0;\n");
        out.push_str("    if (");
        out.push_str(&left_predicate);
        out.push_str(") {\n");
        match op {
            BinaryOp::And => {
                let right_predicate = self.emit_condition(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = (");
                out.push_str(&right_predicate);
                out.push_str(") != 0;\n");
                out.push_str("    } else {\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = 0;\n");
            }
            BinaryOp::Or => {
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = 1;\n");
                out.push_str("    } else {\n");
                let right_predicate = self.emit_condition(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = (");
                out.push_str(&right_predicate);
                out.push_str(") != 0;\n");
            }
            _ => unreachable!(),
        }
        out.push_str("    }\n");
        result_temp
    }

    fn emit_boolean_xor_condition(
        &mut self,
        out: &mut String,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_predicate = self.emit_condition(out, left);
        let left_temp = self.next_temp();
        out.push_str("    int ");
        out.push_str(&left_temp);
        out.push_str(" = (");
        out.push_str(&left_predicate);
        out.push_str(") != 0;\n");
        let right_predicate = self.emit_condition(out, right);
        format!("{left_temp} != (({right_predicate}) != 0)")
    }

    fn emit_boolean_xor(
        &mut self,
        out: &mut String,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_predicate = self.emit_condition(out, left);
        let right_predicate = self.emit_condition(out, right);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool((");
        out.push_str(&left_predicate);
        out.push_str(") != (");
        out.push_str(&right_predicate);
        out.push_str("));\n");
        result_temp
    }

    fn emit_materialized_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        if matches!(
            value,
            ValueExpr::Binary { .. }
                | ValueExpr::Assign { .. }
                | ValueExpr::AssignRef { .. }
                | ValueExpr::IncDec { .. }
                | ValueExpr::InternalCall { .. }
                | ValueExpr::FirstClassCallable { .. }
                | ValueExpr::DynamicCall { .. }
                | ValueExpr::Unary { .. }
                | ValueExpr::Cast { .. }
                | ValueExpr::Array(_)
                | ValueExpr::Closure { .. }
                | ValueExpr::ArrayAccess { .. }
                | ValueExpr::ArrayAppendAccess { .. }
                | ValueExpr::LegacyDollarBraceStringVariable { .. }
                | ValueExpr::DynamicVariable { .. }
                | ValueExpr::Isset { .. }
                | ValueExpr::Empty { .. }
                | ValueExpr::Ternary { .. }
                | ValueExpr::MethodCall { .. }
                | ValueExpr::DynamicMethodCall { .. }
                | ValueExpr::Clone { .. }
                | ValueExpr::StaticPropertyFetch { .. }
                | ValueExpr::ClassConstantFetch { .. }
                | ValueExpr::DynamicClassNameFetch { .. }
                | ValueExpr::Include { .. }
                | ValueExpr::Throw { .. }
        ) {
            return self.emit_value(out, value);
        }

        let temp = self.next_temp();
        let emitted_value = self.emit_value(out, value);
        out.push_str("    PtnValue ");
        out.push_str(&temp);
        out.push_str(" = ");
        out.push_str(&emitted_value);
        out.push_str(";\n");
        temp
    }

    fn emit_call_argument(
        &mut self,
        out: &mut String,
        call_name: &str,
        argument_index: usize,
        argument: &ValueExpr,
    ) -> String {
        if argument_index == 0 && is_array_mutating_internal_call(call_name) {
            if let ValueExpr::Load { name, line } = argument {
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_runtime_read_variable_for_array_mutation(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", \"");
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                return temp;
            }
        }

        self.emit_materialized_value(out, argument)
    }

    fn emit_dynamic_call_argument(&mut self, out: &mut String, argument: &ValueExpr) -> String {
        match reference_target_from_value(argument) {
            Some(target) => self.emit_reference_target(out, &target),
            None => self.emit_materialized_value(out, argument),
        }
    }

    fn emit_reference_source(&mut self, out: &mut String, source: &ValueExpr) -> String {
        if let Some(target) = reference_target_from_value(source) {
            return self.emit_reference_target(out, &target);
        }

        if let ValueExpr::InternalCall {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            line,
        } = source
        {
            let result_temp = self.emit_internal_call(
                out,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                *line,
            );
            let reference_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&reference_temp);
            out.push_str(" = ptn_reference_source_or_value(&runtime, ");
            out.push_str(&result_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &result_temp);
            return reference_temp;
        }

        let temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&temp);
        out.push_str(" = ptn_null();\n");
        out.push_str("    ptn_abort_by_reference_return_error();\n");
        temp
    }

    fn emit_reference_target(&mut self, out: &mut String, target: &ReferenceTarget) -> String {
        match target {
            ReferenceTarget::Variable { name, .. } => {
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_runtime_reference_for_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\");\n");
                temp
            }
            ReferenceTarget::ArrayDim(target) => {
                let path = emit_array_path_segments(out, self, &target.dimensions);
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_runtime_reference_for_array_path(&runtime, \"");
                out.push_str(&c_string(&target.array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push('"');
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&target.line.to_string());
                out.push_str(");\n");
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
                temp
            }
            ReferenceTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                line,
            } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                let property_reference_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&property_reference_temp);
                out.push_str(" = ptn_object_reference_for_property(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                let path = emit_array_path_segments(out, self, dimensions);
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_value_reference_for_array_path(&runtime, &");
                out.push_str(&property_reference_temp);
                out.push_str(", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push('"');
                out.push_str(&c_string(&self.source_file));
                out.push_str("\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
                emit_value_cleanup(out, "    ", &property_reference_temp);
                emit_value_cleanup(out, "    ", &receiver_temp);
                temp
            }
            ReferenceTarget::Property {
                receiver,
                name,
                line,
            } => {
                let receiver_temp = self.emit_materialized_value(out, receiver);
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_object_reference_for_property(&runtime, ");
                out.push_str(&receiver_temp);
                out.push_str(", \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&c_optional_string(self.current_class_name.as_deref()));
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &receiver_temp);
                temp
            }
        }
    }

    fn emit_call_arguments_builder(
        &mut self,
        out: &mut String,
        function_name: &str,
        arguments: &[ValueExpr],
        argument_unpacks: &[bool],
        line: usize,
        dynamic_argument_materialization: bool,
    ) -> String {
        let args_temp = self.next_temp();
        out.push_str("    PtnCallArguments ");
        out.push_str(&args_temp);
        out.push_str(";\n");
        out.push_str("    ptn_call_arguments_init(&");
        out.push_str(&args_temp);
        out.push_str(");\n");
        for (argument_index, argument) in arguments.iter().enumerate() {
            if argument_unpacks
                .get(argument_index)
                .copied()
                .unwrap_or(false)
            {
                let value_temp = self.emit_materialized_value(out, argument);
                out.push_str("    ptn_call_arguments_unpack(&runtime, &");
                out.push_str(&args_temp);
                out.push_str(", ");
                out.push_str(&value_temp);
                out.push_str(", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &value_temp);
            } else {
                let value_temp = if dynamic_argument_materialization {
                    self.emit_dynamic_call_argument(out, argument)
                } else {
                    self.emit_call_argument(out, function_name, argument_index, argument)
                };
                out.push_str("    ptn_call_arguments_append_owned(&");
                out.push_str(&args_temp);
                out.push_str(", ptn_value_share(");
                out.push_str(&value_temp);
                out.push_str("));\n");
                emit_value_cleanup(out, "    ", &value_temp);
            }
        }
        args_temp
    }

    fn emit_internal_call(
        &mut self,
        out: &mut String,
        name: &str,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        argument_unpacks: &[bool],
        line: usize,
    ) -> String {
        let has_named_arguments = argument_names.iter().any(Option::is_some);
        let has_unpacked_arguments = argument_unpacks.iter().any(|unpack| *unpack);
        if !has_named_arguments
            && !has_unpacked_arguments
            && name.eq_ignore_ascii_case("count")
            && arguments.len() == 1
        {
            let argument_temp = self.emit_materialized_value(out, &arguments[0]);
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_count_value(&runtime, \"count\", ");
            out.push_str(&argument_temp);
            out.push_str(", PTN_COUNT_NORMAL, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &argument_temp);
            return result_temp;
        }

        if !has_named_arguments
            && !has_unpacked_arguments
            && name.eq_ignore_ascii_case("array_key_exists")
            && arguments.len() == 2
        {
            let key_temp = self.emit_materialized_value(out, &arguments[0]);
            let array_temp = self.emit_materialized_value(out, &arguments[1]);
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_array_key_exists_value(&runtime, ");
            out.push_str(&key_temp);
            out.push_str(", ");
            out.push_str(&array_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &key_temp);
            emit_value_cleanup(out, "    ", &array_temp);
            return result_temp;
        }

        if !has_named_arguments && !has_unpacked_arguments {
            if let Some(result_temp) =
                self.emit_variable_array_mutator_call(out, name, arguments, line)
            {
                return result_temp;
            }
        }

        let result_temp = self.next_temp();
        let resolved_name = self.resolved_function_call_name(name);
        let called_class_override = self.called_class_override_for_function_call(name);
        let direct_user = self
            .direct_user_function_by_resolved_name(&resolved_name)
            .map(|(index, function)| {
                (
                    user_function_c_name(index),
                    function.parameters.clone(),
                    static_call_receiver_class_name(&resolved_name, function),
                    self.static_method_visibility_check(&resolved_name, function),
                )
            });
        if direct_user.is_none() {
            if let Some((visibility, declaring_class, method_name)) =
                self.static_method_visibility_error(&resolved_name)
            {
                self.emit_static_method_visibility_error(
                    out,
                    &result_temp,
                    visibility,
                    &declaring_class,
                    &method_name,
                    line,
                );
                return result_temp;
            }
        }
        if has_unpacked_arguments {
            if has_named_arguments {
                self.emit_fatal_value(
                    out,
                    &result_temp,
                    "named arguments with argument unpacking are unsupported",
                );
                return result_temp;
            }
            let args_temp = self.emit_call_arguments_builder(
                out,
                name,
                arguments,
                argument_unpacks,
                line,
                false,
            );
            if let Some((c_name, _, receiver_class_name, visibility_check)) = &direct_user {
                self.emit_direct_user_function_call(
                    out,
                    &result_temp,
                    c_name,
                    &format!("{args_temp}.len"),
                    &format!("{args_temp}.values"),
                    line,
                    called_class_override.as_ref(),
                    receiver_class_name.as_deref(),
                    visibility_check.as_ref(),
                );
            } else {
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_call_function(&runtime, \"");
                out.push_str(&c_string(&resolved_name));
                out.push_str("\", ");
                out.push_str(&args_temp);
                out.push_str(".len, ");
                out.push_str(&args_temp);
                out.push_str(".values, ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            }
            out.push_str("    ptn_call_arguments_destroy(&");
            out.push_str(&args_temp);
            out.push_str(");\n");
            return result_temp;
        }
        if has_named_arguments {
            if direct_user.is_none() {
                if let Some(binding) =
                    bind_named_internal_call_arguments(name, arguments, argument_names)
                {
                    return match binding {
                        Ok(normalized_arguments) => {
                            let normalized_argument_names = vec![None; normalized_arguments.len()];
                            self.emit_internal_call(
                                out,
                                name,
                                &normalized_arguments,
                                &normalized_argument_names,
                                &vec![false; normalized_arguments.len()],
                                line,
                            )
                        }
                        Err(error) => {
                            self.emit_fatal_value(out, &result_temp, &error.message());
                            result_temp
                        }
                    };
                }
            }
            if let Some((c_name, parameters, receiver_class_name, visibility_check)) = &direct_user
            {
                return self.emit_named_user_call(
                    out,
                    &result_temp,
                    name,
                    c_name,
                    parameters,
                    receiver_class_name.as_deref(),
                    arguments,
                    argument_names,
                    line,
                    called_class_override.as_ref(),
                    visibility_check.as_ref(),
                );
            }
            self.emit_fatal_value(
                out,
                &result_temp,
                "named arguments currently support user-defined functions",
            );
            return result_temp;
        }
        if arguments.is_empty() {
            if let Some((c_name, _, receiver_class_name, visibility_check)) = &direct_user {
                self.emit_direct_user_function_call(
                    out,
                    &result_temp,
                    c_name,
                    "0",
                    "NULL",
                    line,
                    called_class_override.as_ref(),
                    receiver_class_name.as_deref(),
                    visibility_check.as_ref(),
                );
            } else if let Some((target_class_name, method_name)) =
                self.relative_scoped_call_parts(name)
            {
                self.emit_relative_scoped_method_call_or_function_fallback(
                    out,
                    &result_temp,
                    &target_class_name,
                    &method_name,
                    &resolved_name,
                    "0",
                    "NULL",
                    line,
                );
            } else {
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str("ptn_call_function(&runtime, \"");
                out.push_str(&c_string(&resolved_name));
                out.push_str("\", 0, NULL, ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
            }
            return result_temp;
        }

        let mut temps = Vec::with_capacity(arguments.len());
        let mut unwrap_append_reference_temps = Vec::new();
        for (argument_index, argument) in arguments.iter().enumerate() {
            let by_ref_parameter = direct_user.as_ref().and_then(|(_, parameters, _, _)| {
                by_ref_parameter_for_argument(parameters, argument_index)
            });
            if let Some(parameter) = by_ref_parameter {
                let temp = self.emit_by_ref_call_argument(
                    out,
                    argument,
                    name,
                    argument_index,
                    &parameter.name,
                    line,
                    true,
                    false,
                );
                if value_is_append_reference_target(argument) {
                    unwrap_append_reference_temps.push(temp.clone());
                }
                temps.push(temp);
            } else if let Some(parameter_name) =
                internal_by_ref_parameter_name(name, argument_index)
            {
                let allow_temporary =
                    internal_by_ref_temporary_argument_allowed(name, argument_index);
                let temp = self.emit_by_ref_call_argument(
                    out,
                    argument,
                    name,
                    argument_index,
                    parameter_name,
                    line,
                    allow_temporary,
                    true,
                );
                if value_is_append_reference_target(argument) {
                    unwrap_append_reference_temps.push(temp.clone());
                }
                temps.push(temp);
            } else {
                temps.push(self.emit_call_argument(out, name, argument_index, argument));
            }
        }

        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ");
        for (index, temp) in temps.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("ptn_value_share(");
            out.push_str(temp);
            out.push(')');
        }
        out.push_str(" };\n");
        if let Some((c_name, _, receiver_class_name, visibility_check)) = &direct_user {
            self.emit_direct_user_function_call(
                out,
                &result_temp,
                c_name,
                &arguments.len().to_string(),
                &args_temp,
                line,
                called_class_override.as_ref(),
                receiver_class_name.as_deref(),
                visibility_check.as_ref(),
            );
        } else if let Some((target_class_name, method_name)) = self.relative_scoped_call_parts(name)
        {
            self.emit_relative_scoped_method_call_or_function_fallback(
                out,
                &result_temp,
                &target_class_name,
                &method_name,
                &resolved_name,
                &arguments.len().to_string(),
                &args_temp,
                line,
            );
        } else {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str("ptn_call_function(&runtime, \"");
            out.push_str(&c_string(&resolved_name));
            out.push_str("\", ");
            out.push_str(&arguments.len().to_string());
            out.push_str(", ");
            out.push_str(&args_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
        }
        for temp in &unwrap_append_reference_temps {
            emit_unwrap_append_reference_call_argument(out, "    ", temp);
        }
        for index in 0..temps.len() {
            emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
        }
        for temp in temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        result_temp
    }

    fn emit_direct_user_function_call(
        &mut self,
        out: &mut String,
        result_temp: &str,
        c_name: &str,
        argc_expr: &str,
        args_expr: &str,
        line: usize,
        called_class_override: Option<&CalledClassOverride>,
        receiver_class_name: Option<&str>,
        visibility_check: Option<&StaticMethodVisibilityCheck>,
    ) {
        let previous_override_temp = called_class_override.map(|override_| {
            let previous_override_temp = self.next_temp();
            out.push_str("    const char *");
            out.push_str(&previous_override_temp);
            out.push_str(" = runtime.called_class_name_override;\n");
            out.push_str("    runtime.called_class_name_override = ");
            Self::emit_called_class_override_expr(out, override_);
            out.push_str(";\n");
            previous_override_temp
        });

        if let Some(visibility_check) = visibility_check {
            out.push_str("    PtnValue ");
            out.push_str(result_temp);
            out.push_str(";\n");
            out.push_str("    if (!ptn_declared_method_visible(");
            out.push_str(c_property_visibility(visibility_check.visibility));
            out.push_str(", \"");
            out.push_str(&c_string(&visibility_check.declaring_class_name));
            out.push_str("\", \"");
            out.push_str(&c_string(&visibility_check.target_class_name));
            out.push_str("\", \"");
            out.push_str(&c_string(&visibility_check.method_name));
            out.push_str("\", runtime.current_class_name)) {\n");
            out.push_str("        ");
            out.push_str(result_temp);
            out.push_str(" = ptn_throw_method_visibility_error(&runtime, \"");
            out.push_str(&c_string(&visibility_check.declaring_class_name));
            out.push_str("\", \"");
            out.push_str(&c_string(&visibility_check.method_name));
            out.push_str("\", ");
            out.push_str(c_property_visibility(visibility_check.visibility));
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            if let Some(previous_override_temp) = &previous_override_temp {
                out.push_str("        runtime.called_class_name_override = ");
                out.push_str(previous_override_temp);
                out.push_str(";\n");
            }
            out.push_str("    } else {\n");
            out.push_str("        ");
            out.push_str(result_temp);
            out.push_str(" = ");
        } else {
            out.push_str("    PtnValue ");
            out.push_str(result_temp);
            out.push_str(" = ");
        }
        out.push_str(c_name);
        out.push_str("(&runtime, ");
        emit_static_call_receiver(out, receiver_class_name);
        out.push_str(", ");
        out.push_str(argc_expr);
        out.push_str(", ");
        out.push_str(args_expr);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        if visibility_check.is_some() {
            out.push_str("    }\n");
        }

        if let Some(previous_override_temp) = previous_override_temp {
            out.push_str("    runtime.called_class_name_override = ");
            out.push_str(&previous_override_temp);
            out.push_str(";\n");
        }
    }

    fn emit_relative_scoped_method_call_or_function_fallback(
        &self,
        out: &mut String,
        result_temp: &str,
        target_class_name: &str,
        method_name: &str,
        resolved_name: &str,
        argc: &str,
        args: &str,
        line: usize,
    ) {
        out.push_str("    PtnValue ");
        out.push_str(result_temp);
        out.push_str(";\n");
        out.push_str("    if (runtime.has_current_receiver && ptn_call_declared_method_in_scope(&runtime, runtime.current_receiver, \"");
        out.push_str(&c_string(target_class_name));
        out.push_str("\", \"");
        out.push_str(&c_string(method_name));
        out.push_str("\", runtime.current_called_class_name, ");
        out.push_str(argc);
        out.push_str(", ");
        out.push_str(args);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(", &");
        out.push_str(result_temp);
        out.push_str(")) {\n");
        out.push_str("    } else {\n");
        out.push_str("        ");
        out.push_str(result_temp);
        out.push_str(" = ptn_call_function(&runtime, \"");
        out.push_str(&c_string(resolved_name));
        out.push_str("\", ");
        out.push_str(argc);
        out.push_str(", ");
        out.push_str(args);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        out.push_str("    }\n");
    }

    fn emit_named_user_call(
        &mut self,
        out: &mut String,
        result_temp: &str,
        name: &str,
        c_name: &str,
        parameters: &[crate::ir::FunctionParameter],
        receiver_class_name: Option<&str>,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        line: usize,
        called_class_override: Option<&CalledClassOverride>,
        visibility_check: Option<&StaticMethodVisibilityCheck>,
    ) -> String {
        let argument_slots = match bind_named_call_arguments(parameters, argument_names) {
            Ok(argument_slots) => argument_slots,
            Err(error) => {
                self.emit_fatal_value(out, result_temp, &error.message());
                return result_temp.to_string();
            }
        };

        let frame_len = argument_slots
            .iter()
            .copied()
            .max()
            .map(|slot| slot + 1)
            .unwrap_or(arguments.len())
            .max(arguments.len());
        let mut slot_temps = vec![None; frame_len];
        let mut temps = Vec::with_capacity(arguments.len());
        let mut unwrap_append_reference_temps = Vec::new();
        for (argument_index, argument) in arguments.iter().enumerate() {
            let slot_index = argument_slots[argument_index];
            let by_ref_parameter = parameters
                .get(slot_index)
                .filter(|parameter| parameter.by_ref);
            let temp = if let Some(parameter) = by_ref_parameter {
                self.emit_by_ref_call_argument(
                    out,
                    argument,
                    name,
                    slot_index,
                    &parameter.name,
                    line,
                    true,
                    false,
                )
            } else {
                self.emit_call_argument(out, name, argument_index, argument)
            };
            if by_ref_parameter.is_some() && value_is_append_reference_target(argument) {
                unwrap_append_reference_temps.push(temp.clone());
            }
            slot_temps[slot_index] = Some(temp.clone());
            temps.push(temp);
        }

        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ");
        for (index, temp) in slot_temps.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            if let Some(temp) = temp {
                out.push_str("ptn_value_share(");
                out.push_str(temp);
                out.push(')');
            } else {
                out.push_str("ptn_null()");
            }
        }
        out.push_str(" };\n");
        self.emit_direct_user_function_call(
            out,
            result_temp,
            c_name,
            &arguments.len().to_string(),
            &args_temp,
            line,
            called_class_override,
            receiver_class_name,
            visibility_check,
        );
        for temp in &unwrap_append_reference_temps {
            emit_unwrap_append_reference_call_argument(out, "    ", temp);
        }
        for index in 0..slot_temps.len() {
            emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
        }
        for temp in temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        result_temp.to_string()
    }

    fn emit_fatal_value(&mut self, out: &mut String, result_temp: &str, message: &str) {
        out.push_str("    PtnValue ");
        out.push_str(result_temp);
        out.push_str(" = ptn_null();\n");
        out.push_str("    ptn_emit_type_error(&runtime.diagnostics, \"");
        out.push_str(&c_string(message));
        out.push_str("\");\n");
        out.push_str("    exit(255);\n");
    }

    fn emit_dynamic_call(
        &mut self,
        out: &mut String,
        callee: &ValueExpr,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        argument_unpacks: &[bool],
        line: usize,
    ) -> String {
        if argument_names.iter().any(Option::is_some) {
            let result_temp = self.next_temp();
            self.emit_fatal_value(
                out,
                &result_temp,
                "named arguments currently support user-defined functions",
            );
            return result_temp;
        }
        let callee_temp = self.emit_materialized_value(out, callee);
        let result_temp = self.next_temp();
        if argument_unpacks.iter().any(|unpack| *unpack) {
            let args_temp = self.emit_call_arguments_builder(
                out,
                "{closure}",
                arguments,
                argument_unpacks,
                line,
                true,
            );
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_callable(&runtime, ");
            out.push_str(&callee_temp);
            out.push_str(", ");
            out.push_str(&args_temp);
            out.push_str(".len, ");
            out.push_str(&args_temp);
            out.push_str(".values, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_call_arguments_destroy(&");
            out.push_str(&args_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &callee_temp);
            return result_temp;
        }
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_callable(&runtime, ");
            out.push_str(&callee_temp);
            out.push_str(", 0, NULL, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &callee_temp);
            return result_temp;
        }

        let mut temps = Vec::with_capacity(arguments.len());
        let mut unwrap_append_reference_temps = Vec::new();
        for argument in arguments {
            let temp = self.emit_dynamic_call_argument(out, argument);
            if value_is_append_reference_target(argument) {
                unwrap_append_reference_temps.push(temp.clone());
            }
            temps.push(temp);
        }

        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ");
        for (index, temp) in temps.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("ptn_value_share(");
            out.push_str(temp);
            out.push(')');
        }
        out.push_str(" };\n");
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_call_callable(&runtime, ");
        out.push_str(&callee_temp);
        out.push_str(", ");
        out.push_str(&arguments.len().to_string());
        out.push_str(", ");
        out.push_str(&args_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        for temp in &unwrap_append_reference_temps {
            emit_unwrap_append_reference_call_argument(out, "    ", temp);
        }
        for index in 0..temps.len() {
            emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
        }
        for temp in temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        emit_value_cleanup(out, "    ", &callee_temp);
        result_temp
    }

    fn emit_first_class_callable(
        &mut self,
        out: &mut String,
        callable: &ValueExpr,
        line: usize,
    ) -> String {
        let callable_temp = self.emit_materialized_first_class_callable_target(out, callable);
        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ptn_value_share(");
        out.push_str(&callable_temp);
        out.push_str(") };\n");

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_internal_closure_from_callable(&runtime, 1, ");
        out.push_str(&args_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");

        emit_value_cleanup(out, "    ", &format!("{args_temp}[0]"));
        emit_value_cleanup(out, "    ", &callable_temp);
        result_temp
    }

    fn emit_materialized_first_class_callable_target(
        &mut self,
        out: &mut String,
        callable: &ValueExpr,
    ) -> String {
        match callable {
            ValueExpr::String(name) => {
                let target_name = self.first_class_callable_static_name(name);
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_string(\"");
                out.push_str(&c_string(&target_name));
                out.push_str("\");\n");
                temp
            }
            _ => self.emit_materialized_value(out, callable),
        }
    }

    fn first_class_callable_static_name(&self, name: &str) -> String {
        let Some((class_name, method_name)) = name.split_once("::") else {
            return name.to_string();
        };
        format!(
            "{}::{}",
            self.static_property_class_name(class_name),
            method_name
        )
    }

    fn emit_variable_array_mutator_call(
        &mut self,
        out: &mut String,
        name: &str,
        arguments: &[ValueExpr],
        line: usize,
    ) -> Option<String> {
        let first_argument = arguments.first()?;
        let variable_name = match first_argument {
            ValueExpr::Load { name, .. } => Some(name.as_str()),
            _ => None,
        };

        if name.eq_ignore_ascii_case("array_multisort") {
            let mut temps = Vec::with_capacity(arguments.len());
            for argument in arguments {
                if let ValueExpr::Load { name, .. } = argument {
                    let temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&temp);
                    out.push_str(" = ptn_runtime_reference_for_variable(&runtime, \"");
                    out.push_str(&c_string(name));
                    out.push_str("\");\n");
                    temps.push(temp);
                } else {
                    temps.push(self.emit_materialized_value(out, argument));
                }
            }

            let args_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&args_temp);
            out.push_str("[] = { ");
            for (index, temp) in temps.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str("ptn_value_share(");
                out.push_str(temp);
                out.push(')');
            }
            out.push_str(" };\n");

            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_function(&runtime, \"array_multisort\", ");
            out.push_str(&arguments.len().to_string());
            out.push_str(", ");
            out.push_str(&args_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");

            for index in 0..temps.len() {
                emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
            }
            for temp in temps {
                emit_value_cleanup(out, "    ", &temp);
            }
            return Some(result_temp);
        }

        if (name.eq_ignore_ascii_case("array_walk")
            || name.eq_ignore_ascii_case("array_walk_recursive"))
            && (arguments.len() == 2 || arguments.len() == 3)
        {
            let variable_name = variable_name?;
            let array_temp = self.emit_materialized_value(out, &arguments[0]);
            let callback_temp = self.emit_materialized_value(out, &arguments[1]);
            let userdata_temp = arguments
                .get(2)
                .map(|argument| self.emit_materialized_value(out, argument));
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            if name.eq_ignore_ascii_case("array_walk_recursive") {
                out.push_str("ptn_runtime_array_walk_recursive_variable");
            } else {
                out.push_str("ptn_runtime_array_walk_variable");
            }
            out.push_str("(&runtime, \"");
            out.push_str(&c_string(variable_name));
            out.push_str("\", ");
            out.push_str(&array_temp);
            out.push_str(", ");
            out.push_str(&callback_temp);
            out.push_str(", ");
            out.push_str(if userdata_temp.is_some() { "1" } else { "0" });
            out.push_str(", ");
            if let Some(userdata_temp) = &userdata_temp {
                out.push_str(userdata_temp);
            } else {
                out.push_str("ptn_null()");
            }
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            if let Some(userdata_temp) = &userdata_temp {
                emit_value_cleanup(out, "    ", userdata_temp);
            }
            emit_value_cleanup(out, "    ", &callback_temp);
            emit_value_cleanup(out, "    ", &array_temp);
            return Some(result_temp);
        }

        if matches!(
            name.to_ascii_lowercase().as_str(),
            "uasort" | "uksort" | "usort"
        ) && arguments.len() == 2
        {
            let variable_name = variable_name?;
            let array_temp = self.emit_materialized_value(out, &arguments[0]);
            let callback_temp = self.emit_materialized_value(out, &arguments[1]);
            let result_temp = self.next_temp();
            let helper = if name.eq_ignore_ascii_case("uasort") {
                "ptn_runtime_array_uasort_variable"
            } else if name.eq_ignore_ascii_case("uksort") {
                "ptn_runtime_array_uksort_variable"
            } else {
                "ptn_runtime_array_usort_variable"
            };
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str(helper);
            out.push_str("(&runtime, \"");
            out.push_str(&c_string(variable_name));
            out.push_str("\", ");
            out.push_str(&array_temp);
            out.push_str(", ");
            out.push_str(&callback_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &callback_temp);
            emit_value_cleanup(out, "    ", &array_temp);
            return Some(result_temp);
        }

        let sort_regular_flag_argument = arguments.len() == 2
            && matches!(
                name.to_ascii_lowercase().as_str(),
                "arsort" | "asort" | "krsort" | "ksort" | "rsort" | "sort"
            );
        let helper = if arguments.len() == 1 || sort_regular_flag_argument {
            if name.eq_ignore_ascii_case("array_pop") {
                Some("ptn_runtime_array_pop_variable")
            } else if name.eq_ignore_ascii_case("array_shift") {
                Some("ptn_runtime_array_shift_variable")
            } else if name.eq_ignore_ascii_case("arsort") {
                Some("ptn_runtime_array_arsort_variable")
            } else if name.eq_ignore_ascii_case("asort") {
                Some("ptn_runtime_array_asort_variable")
            } else if name.eq_ignore_ascii_case("krsort") {
                Some("ptn_runtime_array_krsort_variable")
            } else if name.eq_ignore_ascii_case("ksort") {
                Some("ptn_runtime_array_ksort_variable")
            } else if name.eq_ignore_ascii_case("natcasesort") {
                Some("ptn_runtime_array_natcasesort_variable")
            } else if name.eq_ignore_ascii_case("natsort") {
                Some("ptn_runtime_array_natsort_variable")
            } else if name.eq_ignore_ascii_case("next") {
                Some("ptn_runtime_array_next_variable")
            } else if name.eq_ignore_ascii_case("end") {
                Some("ptn_runtime_array_end_variable")
            } else if name.eq_ignore_ascii_case("prev") {
                Some("ptn_runtime_array_prev_variable")
            } else if name.eq_ignore_ascii_case("reset") {
                Some("ptn_runtime_array_reset_variable")
            } else if name.eq_ignore_ascii_case("rsort") {
                Some("ptn_runtime_array_rsort_variable")
            } else if name.eq_ignore_ascii_case("shuffle") {
                Some("ptn_runtime_array_shuffle_variable")
            } else if name.eq_ignore_ascii_case("sort") {
                Some("ptn_runtime_array_sort_variable")
            } else if name.eq_ignore_ascii_case("rsort") {
                Some("ptn_runtime_array_rsort_variable")
            } else {
                None
            }
        } else {
            None
        };

        if let Some(helper) = helper {
            if let Some(variable_name) = variable_name {
                let array_temp = self.emit_materialized_value(out, &arguments[0]);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(helper);
                out.push_str("(&runtime, \"");
                out.push_str(&c_string(variable_name));
                out.push_str("\", ");
                out.push_str(&array_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &array_temp);
                return Some(result_temp);
            }

            if let Some(ReferenceTarget::ArrayDim(target)) =
                reference_array_dim_target_from_value(first_argument)
            {
                let path_helper = if name.eq_ignore_ascii_case("array_pop") {
                    Some("ptn_runtime_array_pop_path")
                } else if name.eq_ignore_ascii_case("array_shift") {
                    Some("ptn_runtime_array_shift_path")
                } else if name.eq_ignore_ascii_case("next") {
                    Some("ptn_runtime_array_next_path")
                } else if name.eq_ignore_ascii_case("end") {
                    Some("ptn_runtime_array_end_path")
                } else if name.eq_ignore_ascii_case("prev") {
                    Some("ptn_runtime_array_prev_path")
                } else if name.eq_ignore_ascii_case("reset") {
                    Some("ptn_runtime_array_reset_path")
                } else {
                    None
                }?;
                let path = emit_array_path_segments(out, self, &target.dimensions);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(path_helper);
                out.push_str("(&runtime, \"");
                out.push_str(&c_string(&target.array));
                out.push_str("\", ");
                out.push_str(&path.name);
                out.push_str(", ");
                out.push_str(&path.len.to_string());
                out.push_str(", ");
                out.push_str(&target.line.to_string());
                out.push_str(");\n");
                for segment_temp in path.value_temps {
                    emit_value_cleanup(out, "    ", &segment_temp);
                }
                return Some(result_temp);
            }

            if by_ref_temporary_argument_allowed(first_argument) {
                let temporary_helper = if name.eq_ignore_ascii_case("array_shift") {
                    Some("ptn_runtime_array_shift_temporary")
                } else {
                    cursor_temporary_helper_name(name)
                };
                if let Some(temporary_helper) = temporary_helper {
                    let value_temp = self.emit_materialized_value(out, first_argument);
                    let result_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&result_temp);
                    out.push_str(" = ");
                    out.push_str(temporary_helper);
                    out.push_str("(&runtime, ");
                    out.push_str(&value_temp);
                    out.push_str(", ");
                    out.push_str(&line.to_string());
                    out.push_str(");\n");
                    emit_value_cleanup(out, "    ", &value_temp);
                    return Some(result_temp);
                }
            }

            if cursor_temporary_helper_name(name).is_some() {
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_runtime_by_reference_argument_error(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", 1, \"array\", ");
                out.push_str(&line.to_string());
                out.push_str(");\n");
                return Some(result_temp);
            }

            return None;
        }

        let helpers = if name.eq_ignore_ascii_case("array_push") {
            Some((
                "ptn_runtime_array_push_variable",
                "ptn_runtime_array_push_path",
            ))
        } else if name.eq_ignore_ascii_case("array_unshift") {
            Some((
                "ptn_runtime_array_unshift_variable",
                "ptn_runtime_array_unshift_path",
            ))
        } else {
            None
        };
        let Some((variable_helper, path_helper)) = helpers else {
            return None;
        };

        let path_target = if variable_name.is_none() {
            match reference_array_dim_target_from_value(first_argument) {
                Some(ReferenceTarget::ArrayDim(target)) => Some(target),
                _ => None,
            }
        } else {
            None
        };
        if variable_name.is_none() && path_target.is_none() {
            return None;
        }

        let path = path_target
            .as_ref()
            .map(|target| emit_array_path_segments(out, self, &target.dimensions));
        let array_temp = variable_name.map(|_| self.emit_materialized_value(out, &arguments[0]));
        let mut value_temps = Vec::with_capacity(arguments.len().saturating_sub(1));
        for argument in &arguments[1..] {
            value_temps.push(self.emit_materialized_value(out, argument));
        }

        let values_temp = if value_temps.is_empty() {
            None
        } else {
            let values_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&values_temp);
            out.push_str("[] = { ");
            for (index, temp) in value_temps.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str("ptn_value_share(");
                out.push_str(temp);
                out.push(')');
            }
            out.push_str(" };\n");
            Some(values_temp)
        };

        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        if let Some(variable_name) = variable_name {
            out.push_str(variable_helper);
            out.push_str("(&runtime, \"");
            out.push_str(&c_string(variable_name));
            out.push_str("\", ");
            out.push_str(array_temp.as_ref().expect("variable mutator temp"));
            out.push_str(", ");
            out.push_str(&value_temps.len().to_string());
            out.push_str(", ");
            if let Some(values_temp) = &values_temp {
                out.push_str(values_temp);
            } else {
                out.push_str("NULL");
            }
        } else {
            let target = path_target.as_ref().expect("array path mutator target");
            let path = path.as_ref().expect("array path mutator segments");
            out.push_str(path_helper);
            out.push_str("(&runtime, \"");
            out.push_str(&c_string(&target.array));
            out.push_str("\", ");
            out.push_str(&path.name);
            out.push_str(", ");
            out.push_str(&path.len.to_string());
            out.push_str(", ");
            out.push_str(&target.line.to_string());
            out.push_str(", ");
            out.push_str(&value_temps.len().to_string());
            out.push_str(", ");
            if let Some(values_temp) = &values_temp {
                out.push_str(values_temp);
            } else {
                out.push_str("NULL");
            }
        }
        out.push_str(");\n");
        if let Some(values_temp) = &values_temp {
            for index in 0..value_temps.len() {
                emit_value_cleanup(out, "    ", &format!("{values_temp}[{index}]"));
            }
        }
        for temp in value_temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        if let Some(array_temp) = &array_temp {
            emit_value_cleanup(out, "    ", array_temp);
        }
        if let Some(path) = path {
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        Some(result_temp)
    }

    fn emit_by_ref_call_argument(
        &mut self,
        out: &mut String,
        argument: &ValueExpr,
        function_name: &str,
        argument_index: usize,
        parameter_name: &str,
        line: usize,
        allow_temporary: bool,
        throw_on_failure: bool,
    ) -> String {
        match reference_target_from_value(argument) {
            Some(target) => self.emit_reference_target(out, &target),
            None => {
                if allow_temporary && by_ref_temporary_argument_allowed(argument) {
                    let value_temp = self.emit_materialized_value(out, argument);
                    emit_only_variables_passed_by_reference_notice(out, "    ", line);
                    let reference_temp = self.next_temp();
                    out.push_str("    PtnValue ");
                    out.push_str(&reference_temp);
                    out.push_str(" = ptn_reference_value(ptn_reference_new_owned(ptn_value_clone(ptn_value_deref(");
                    out.push_str(&value_temp);
                    out.push_str("))));\n");
                    emit_value_cleanup(out, "    ", &value_temp);
                    return reference_temp;
                }
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_null();\n");
                if throw_on_failure {
                    out.push_str("    ptn_throw_by_reference_argument_error(&runtime, \"");
                    out.push_str(&c_string(function_name));
                    out.push_str("\", ");
                    out.push_str(&(argument_index + 1).to_string());
                    out.push_str(", \"");
                    out.push_str(&c_string(parameter_name));
                    out.push_str("\", ");
                    out.push_str(&line.to_string());
                    out.push_str(");\n");
                } else {
                    out.push_str("    ptn_abort_by_reference_argument_error(\"");
                    out.push_str(&c_string(function_name));
                    out.push_str("\", ");
                    out.push_str(&(argument_index + 1).to_string());
                    out.push_str(", \"");
                    out.push_str(&c_string(parameter_name));
                    out.push_str("\");\n");
                }
                temp
            }
        }
    }

    fn emit_method_call(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &str,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        argument_unpacks: &[bool],
        line: usize,
    ) -> String {
        if argument_names.iter().any(Option::is_some) {
            let result_temp = self.next_temp();
            self.emit_fatal_value(
                out,
                &result_temp,
                "named arguments currently support user-defined functions",
            );
            return result_temp;
        }
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let result_temp = self.next_temp();
        if argument_unpacks.iter().any(|unpack| *unpack) {
            let args_temp = self.emit_call_arguments_builder(
                out,
                name,
                arguments,
                argument_unpacks,
                line,
                true,
            );
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_declared_method(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&args_temp);
            out.push_str(".len, ");
            out.push_str(&args_temp);
            out.push_str(".values, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_call_arguments_destroy(&");
            out.push_str(&args_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &receiver_temp);
            return result_temp;
        }
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_declared_method(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", 0, NULL, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &receiver_temp);
            return result_temp;
        }

        let declared_signature =
            self.declared_instance_method_signature_for_receiver(receiver, name);
        let mut temps = Vec::with_capacity(arguments.len());
        let mut unwrap_append_reference_temps = Vec::new();
        for (argument_index, argument) in arguments.iter().enumerate() {
            let by_ref_parameter = declared_signature.as_ref().and_then(|(_, parameters)| {
                by_ref_parameter_for_argument(parameters, argument_index)
            });
            let temp = if let Some(parameter) = by_ref_parameter {
                let display_name = declared_signature
                    .as_ref()
                    .map(|(display_name, _)| display_name.as_str())
                    .unwrap_or(name);
                self.emit_by_ref_call_argument(
                    out,
                    argument,
                    display_name,
                    argument_index,
                    &parameter.name,
                    line,
                    true,
                    false,
                )
            } else if declared_signature.is_some() {
                self.emit_call_argument(out, name, argument_index, argument)
            } else {
                self.emit_dynamic_call_argument(out, argument)
            };
            if by_ref_parameter.is_some() && value_is_append_reference_target(argument) {
                unwrap_append_reference_temps.push(temp.clone());
            }
            temps.push(temp);
        }
        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ");
        for (index, temp) in temps.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("ptn_value_share(");
            out.push_str(temp);
            out.push(')');
        }
        out.push_str(" };\n");
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_call_declared_method(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&arguments.len().to_string());
        out.push_str(", ");
        out.push_str(&args_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        for temp in &unwrap_append_reference_temps {
            emit_unwrap_append_reference_call_argument(out, "    ", temp);
        }
        for index in 0..temps.len() {
            emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
        }
        for temp in temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        emit_value_cleanup(out, "    ", &receiver_temp);
        result_temp
    }

    fn emit_dynamic_method_call(
        &mut self,
        out: &mut String,
        receiver: &ValueExpr,
        name: &ValueExpr,
        arguments: &[ValueExpr],
        argument_names: &[Option<String>],
        argument_unpacks: &[bool],
        line: usize,
    ) -> String {
        if argument_names.iter().any(Option::is_some) {
            let result_temp = self.next_temp();
            self.emit_fatal_value(
                out,
                &result_temp,
                "named arguments currently support user-defined functions",
            );
            return result_temp;
        }
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let name_temp = self.emit_materialized_value(out, name);
        let method_name_temp = self.next_temp();
        out.push_str("    char *");
        out.push_str(&method_name_temp);
        out.push_str(" = ptn_value_to_string(ptn_value_deref(");
        out.push_str(&name_temp);
        out.push_str("));\n");
        let result_temp = self.next_temp();
        if argument_unpacks.iter().any(|unpack| *unpack) {
            let args_temp = self.emit_call_arguments_builder(
                out,
                "dynamic method call",
                arguments,
                argument_unpacks,
                line,
                true,
            );
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_declared_method(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", ");
            out.push_str(&method_name_temp);
            out.push_str(", ");
            out.push_str(&args_temp);
            out.push_str(".len, ");
            out.push_str(&args_temp);
            out.push_str(".values, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    ptn_call_arguments_destroy(&");
            out.push_str(&args_temp);
            out.push_str(");\n");
            out.push_str("    free(");
            out.push_str(&method_name_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &name_temp);
            emit_value_cleanup(out, "    ", &receiver_temp);
            return result_temp;
        }
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_declared_method(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", ");
            out.push_str(&method_name_temp);
            out.push_str(", 0, NULL, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            out.push_str("    free(");
            out.push_str(&method_name_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &name_temp);
            emit_value_cleanup(out, "    ", &receiver_temp);
            return result_temp;
        }

        let mut temps = Vec::with_capacity(arguments.len());
        let mut unwrap_append_reference_temps = Vec::new();
        for argument in arguments {
            let temp = self.emit_dynamic_call_argument(out, argument);
            if value_is_append_reference_target(argument) {
                unwrap_append_reference_temps.push(temp.clone());
            }
            temps.push(temp);
        }
        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ");
        for (index, temp) in temps.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("ptn_value_share(");
            out.push_str(temp);
            out.push(')');
        }
        out.push_str(" };\n");
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_call_declared_method(&runtime, ");
        out.push_str(&receiver_temp);
        out.push_str(", ");
        out.push_str(&method_name_temp);
        out.push_str(", ");
        out.push_str(&arguments.len().to_string());
        out.push_str(", ");
        out.push_str(&args_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
        for temp in &unwrap_append_reference_temps {
            emit_unwrap_append_reference_call_argument(out, "    ", temp);
        }
        for index in 0..temps.len() {
            emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
        }
        for temp in temps {
            emit_value_cleanup(out, "    ", &temp);
        }
        out.push_str("    free(");
        out.push_str(&method_name_temp);
        out.push_str(");\n");
        emit_value_cleanup(out, "    ", &name_temp);
        emit_value_cleanup(out, "    ", &receiver_temp);
        result_temp
    }

    fn next_temp(&mut self) -> String {
        let temp = format!("ptn_tmp_{}", self.next_temp);
        self.next_temp += 1;
        temp
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}_{}", self.next_label);
        self.next_label += 1;
        label
    }
}

fn collect_concat_operands<'a>(
    value: &'a ValueExpr,
    line: usize,
    operands: &mut Vec<ConcatOperand<'a>>,
) {
    match value {
        ValueExpr::Binary {
            op: BinaryOp::Concat,
            left,
            right,
            line: concat_line,
        } => {
            collect_concat_operands(left, *concat_line, operands);
            collect_concat_operands(right, *concat_line, operands);
        }
        _ => operands.push(ConcatOperand { value, line }),
    }
}

fn c_string(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if let Some(byte) = php_binary_sentinel_byte(ch) {
            push_c_string_byte(&mut out, byte);
        } else {
            let mut encoded = [0; 4];
            for byte in ch.encode_utf8(&mut encoded).bytes() {
                push_c_string_byte(&mut out, byte);
            }
        }
    }
    out
}

fn c_optional_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", c_string(value)),
        None => "NULL".to_string(),
    }
}

fn c_property_visibility(visibility: PropertyVisibility) -> &'static str {
    match visibility {
        PropertyVisibility::Public => "PTN_PROPERTY_PUBLIC",
        PropertyVisibility::Protected => "PTN_PROPERTY_PROTECTED",
        PropertyVisibility::Private => "PTN_PROPERTY_PRIVATE",
    }
}

fn c_method_visibility(visibility: PropertyVisibility) -> &'static str {
    c_property_visibility(visibility)
}

fn method_visibility_name(visibility: PropertyVisibility) -> &'static str {
    match visibility {
        PropertyVisibility::Public => "public",
        PropertyVisibility::Protected => "protected",
        PropertyVisibility::Private => "private",
    }
}

fn push_c_string_byte(out: &mut String, byte: u8) {
    match byte {
        b'\\' => out.push_str("\\\\"),
        b'"' => out.push_str("\\\""),
        b'\n' => out.push_str("\\n"),
        b'\r' => out.push_str("\\r"),
        b'\t' => out.push_str("\\t"),
        0x20..=0x7e => out.push(byte as char),
        _ => out.push_str(&format!("\\{byte:03o}")),
    }
}

fn php_string_byte_len(value: &str) -> usize {
    value
        .chars()
        .map(|ch| {
            if php_binary_sentinel_byte(ch).is_some() {
                1
            } else {
                ch.len_utf8()
            }
        })
        .sum()
}

fn php_binary_sentinel_byte(ch: char) -> Option<u8> {
    let value = ch as u32;
    let offset = value.checked_sub(PHP_BINARY_BYTE_SENTINEL_BASE)?;
    if (0x80..=0xff).contains(&offset) {
        Some(offset as u8)
    } else {
        None
    }
}

fn c_label(value: &str) -> String {
    let mut out = String::from("ptn_user_label_");
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            out.push(byte as char);
        } else {
            out.push_str(&format!("_x{byte:02x}"));
        }
    }
    out
}

fn display_os(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::cc_optimization_args_for;

    #[test]
    fn default_c_compiler_profile_uses_o2() {
        assert_eq!(cc_optimization_args_for(None).unwrap(), vec!["-O2"]);
        assert_eq!(cc_optimization_args_for(Some("")).unwrap(), vec!["-O2"]);
    }

    #[test]
    fn c_compiler_profile_accepts_debug_and_optimization_levels() {
        assert_eq!(
            cc_optimization_args_for(Some("debug")).unwrap(),
            vec!["-O0", "-g"]
        );
        assert_eq!(
            cc_optimization_args_for(Some("0")).unwrap(),
            vec!["-O0", "-g"]
        );
        assert_eq!(cc_optimization_args_for(Some("-O1")).unwrap(), vec!["-O1"]);
        assert_eq!(cc_optimization_args_for(Some("2")).unwrap(), vec!["-O2"]);
        assert_eq!(cc_optimization_args_for(Some("O3")).unwrap(), vec!["-O3"]);
        assert_eq!(cc_optimization_args_for(Some("s")).unwrap(), vec!["-Os"]);
        assert_eq!(cc_optimization_args_for(Some("Oz")).unwrap(), vec!["-Oz"]);
    }

    #[test]
    fn c_compiler_profile_rejects_unknown_values() {
        let error = cc_optimization_args_for(Some("fast")).unwrap_err();
        assert!(error.message.contains("invalid PTN_CC_OPT_LEVEL value"));
    }
}
