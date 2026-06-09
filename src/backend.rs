use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{
    ArrayElement as IrArrayElement, ArrayElementValue as IrArrayElementValue, BinaryOp, CastKind,
    CatchClause as IrCatchClause, FunctionDecl, IncDecOp, Instruction, MagicConstantKind, Module,
    ReferenceTarget, TypeHint, UnaryOp, ValueExpr,
};

mod runtime;

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    let runtime_requirements = module_runtime_requirements(module);
    emit_runtime(&mut out, &runtime_requirements);
    emit_user_function_prototypes(
        &mut out,
        &module.functions,
        runtime_requirements.internal_function_dispatch,
    );
    emit_user_functions(
        &mut out,
        &module.functions,
        &module.source_file,
        &module.source_dir,
    );
    if runtime_requirements.internal_function_dispatch {
        emit_user_function_dispatch(&mut out, &module.functions);
    }
    out.push_str("\nint main(void) {\n");
    out.push_str("    PtnRuntime runtime;\n");
    out.push_str("    ptn_runtime_init(&runtime);\n");
    for warning in collect_control_warnings(&module.instructions) {
        emit_control_warning(
            &mut out,
            &warning.message,
            &module.source_file,
            warning.line,
        );
    }
    let mut values = ValueEmitter::new(&module.source_file, &module.source_dir, &module.functions);
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

#[derive(Default)]
struct RuntimeRequirements {
    internal_function_dispatch: bool,
    direct_internal_helpers: bool,
}

fn emit_runtime(out: &mut String, requirements: &RuntimeRequirements) {
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
) {
    if needs_function_dispatch {
        out.push_str(
            "\nstatic PTN_UNUSED PtnValue ptn_call_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line);\n",
        );
    }
    for (index, _) in functions.iter().enumerate() {
        out.push_str("static PTN_UNUSED PtnValue ");
        out.push_str(&user_function_c_name(index));
        out.push_str(
            "(PtnRuntime *caller_runtime, size_t argc, const PtnValue *args, size_t line);\n",
        );
    }
}

fn emit_user_functions(
    out: &mut String,
    functions: &[FunctionDecl],
    source_file: &str,
    source_dir: &str,
) {
    for (index, function) in functions.iter().enumerate() {
        let c_name = user_function_c_name(index);
        out.push_str("\nstatic PTN_UNUSED PtnValue ");
        out.push_str(&c_name);
        out.push_str(
            "(PtnRuntime *caller_runtime, size_t argc, const PtnValue *args, size_t line) {\n",
        );
        out.push_str("    (void)line;\n");
        if !function.parameters.is_empty() {
            out.push_str("    if (argc < ");
            out.push_str(&function.parameters.len().to_string());
            out.push_str(") {\n");
            out.push_str("        ptn_emit_argument_count_error(&caller_runtime->diagnostics, \"");
            out.push_str(&c_string(&function.name));
            out.push_str("\", ");
            out.push_str(&function.parameters.len().to_string());
            out.push_str(", argc);\n");
            out.push_str("        exit(255);\n");
            out.push_str("    }\n");
        }
        out.push_str("    PtnRuntime runtime;\n");
        out.push_str("    ptn_runtime_init_function_frame(&runtime, caller_runtime);\n");
        if function.parameters.is_empty() {
            out.push_str("    ptn_runtime_set_call_frame(&runtime, argc, args, 0, NULL);\n");
        } else {
            out.push_str("    static const char *ptn_parameter_names[] = { ");
            for (parameter_index, parameter) in function.parameters.iter().enumerate() {
                if parameter_index > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                out.push_str(&c_string(&parameter.name));
                out.push('"');
            }
            out.push_str(" };\n");
            out.push_str("    ptn_runtime_set_call_frame(&runtime, argc, args, ");
            out.push_str(&function.parameters.len().to_string());
            out.push_str(", ptn_parameter_names);\n");
        }
        out.push_str("    PtnValue ptn_return_value = ptn_null();\n");
        for (parameter_index, parameter) in function.parameters.iter().enumerate() {
            if let Some(TypeHint::Null) = parameter.type_hint {
                out.push_str("    if (args[");
                out.push_str(&parameter_index.to_string());
                out.push_str("].type != PTN_NULL");
                if parameter.by_ref {
                    out.push_str(" && ptn_value_deref(args[");
                    out.push_str(&parameter_index.to_string());
                    out.push_str("]).type != PTN_NULL");
                }
                out.push_str(") {\n");
                out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
                out.push_str(&c_string(&function.name));
                out.push_str("() argument $");
                out.push_str(&c_string(&parameter.name));
                out.push_str(" must be of type null\");\n");
                out.push_str("        ptn_runtime_free(&runtime);\n");
                out.push_str("        exit(255);\n");
                out.push_str("    }\n");
            }
            if parameter.by_ref {
                out.push_str("    if (args[");
                out.push_str(&parameter_index.to_string());
                out.push_str("].type != PTN_REFERENCE) {\n");
                out.push_str("        ptn_abort_by_reference_argument_error(\"");
                out.push_str(&c_string(&function.name));
                out.push_str("\", ");
                out.push_str(&(parameter_index + 1).to_string());
                out.push_str(", \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\");\n");
                out.push_str("    }\n");
                out.push_str("    ptn_runtime_bind_variable_reference(&runtime, \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", args[");
                out.push_str(&parameter_index.to_string());
                out.push_str("]);\n");
            } else {
                out.push_str("    ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(&parameter.name));
                out.push_str("\", args[");
                out.push_str(&parameter_index.to_string());
                out.push_str("]);\n");
            }
        }
        let mut values =
            ValueEmitter::new_for_function(source_file, source_dir, functions, &function.name);
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
        if let Some(TypeHint::Null) = function.return_type {
            out.push_str("    if (ptn_return_value.type != PTN_NULL) {\n");
            out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
            out.push_str(&c_string(&function.name));
            out.push_str("() return value must be of type null\");\n");
            out.push_str("        ptn_runtime_free(&runtime);\n");
            out.push_str("        exit(255);\n");
            out.push_str("    }\n");
        }
        out.push_str("    ptn_runtime_free(&runtime);\n");
        out.push_str("    return ptn_return_value;\n");
        out.push_str("}\n");
    }
}

fn emit_user_function_dispatch(out: &mut String, functions: &[FunctionDecl]) {
    out.push_str("\nstatic int ptn_user_function_exists(const char *name) {\n");
    if functions.is_empty() {
        out.push_str("    (void)name;\n");
    }
    for function in functions {
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&function.name));
        out.push_str("\")) {\n");
        out.push_str("        return 1;\n");
        out.push_str("    }\n");
    }
    out.push_str("    return 0;\n");
    out.push_str("}\n");

    out.push_str(
        "\nstatic PtnValue ptn_call_user_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line, int *found) {\n",
    );
    if functions.is_empty() {
        out.push_str("    (void)runtime;\n");
        out.push_str("    (void)name;\n");
        out.push_str("    (void)argc;\n");
        out.push_str("    (void)args;\n");
        out.push_str("    (void)line;\n");
    }
    for (index, function) in functions.iter().enumerate() {
        out.push_str("    if (ptn_ascii_case_equal(name, \"");
        out.push_str(&c_string(&function.name));
        out.push_str("\")) {\n");
        out.push_str("        *found = 1;\n");
        out.push_str("        return ");
        out.push_str(&user_function_c_name(index));
        out.push_str("(runtime, argc, args, line);\n");
        out.push_str("    }\n");
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
        Instruction::StoreRef { name, target } => {
            let reference_temp = values.emit_reference_target(out, target);
            out.push_str("    ptn_runtime_bind_variable_reference(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&reference_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &reference_temp);
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
                    out.push_str(&current_temp);
                    out.push_str(", ");
                    out.push_str(&value_temp);
                    out.push(')');
                }
                out.push_str(";\n");
                emit_value_cleanup(out, "    ", &current_temp);
                result_temp
            } else {
                value_temp.clone()
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
            if compound_op.is_some() {
                emit_value_cleanup(out, "    ", &stored_temp);
            }
            emit_value_cleanup(out, "    ", &value_temp);
            for segment_temp in path.value_temps {
                emit_value_cleanup(out, "    ", &segment_temp);
            }
        }
        Instruction::StoreArrayDimRef { target, source } => {
            let source_temp = values.emit_reference_target(out, source);
            let index_temp = target
                .index
                .as_ref()
                .map(|index| values.emit_materialized_value(out, index));
            out.push_str("    ptn_runtime_bind_array_dim_reference(&runtime, \"");
            out.push_str(&c_string(&target.array));
            out.push_str("\", ");
            match &index_temp {
                Some(index_temp) => {
                    out.push('&');
                    out.push_str(index_temp);
                }
                None => out.push_str("NULL"),
            }
            out.push_str(", ");
            out.push_str(&source_temp);
            out.push_str(", ");
            out.push_str(&target.line.to_string());
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &source_temp);
            if let Some(index_temp) = index_temp {
                emit_value_cleanup(out, "    ", &index_temp);
            }
        }
        Instruction::DefineConstant { name, value, line } => {
            let emitted_value = values.emit_materialized_value(out, value);
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
            out.push_str("    ptn_echo(");
            out.push_str(&emitted_value);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &emitted_value);
        }
        Instruction::Increment { name, op, line } => {
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
            out.push_str(match op {
                IncDecOp::Increment => "ptn_increment",
                IncDecOp::Decrement => "ptn_decrement",
            });
            out.push('(');
            out.push_str(&current_temp);
            out.push_str(");\n");
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&result_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &current_temp);
            emit_value_cleanup(out, "    ", &result_temp);
        }
        Instruction::UnsetVariable { name } => {
            out.push_str("    ptn_runtime_unset_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\");\n");
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
        Instruction::InternalCall {
            name,
            arguments,
            line,
        } => {
            let result_temp = values.emit_internal_call(out, name, arguments, *line);
            out.push_str("    (void)");
            out.push_str(&result_temp);
            out.push_str(";\n");
            emit_value_cleanup(out, "    ", &result_temp);
        }
        Instruction::Return { value, .. } => match return_target {
            Some(target) => {
                if let Some(value) = value {
                    let result_value = values.emit_materialized_value(out, value);
                    out.push_str("    ptn_return_value = ptn_value_share(");
                    out.push_str(&result_value);
                    out.push_str(");\n");
                    emit_value_cleanup(out, "    ", &result_value);
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
                        out.push_str("\", \"");
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
                out.push_str("        ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(key));
                out.push_str("\", ");
                out.push_str(&key_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "        ", &key_temp);
            }
            let value_temp = values.next_temp();
            out.push_str("        PtnValue ");
            out.push_str(&value_temp);
            out.push_str(" = ");
            if *value_by_ref {
                out.push_str("ptn_array_iterator_current_reference(&");
            } else {
                out.push_str("ptn_array_iterator_current_value(&");
            }
            out.push_str(&iterator_temp);
            out.push_str(");\n");
            if *value_by_ref {
                out.push_str("        ptn_runtime_bind_variable_reference(&runtime, \"");
            } else {
                out.push_str("        ptn_runtime_write_variable(&runtime, \"");
            }
            out.push_str(&c_string(value));
            out.push_str("\", ");
            out.push_str(&value_temp);
            out.push_str(");\n");
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
    let end_label = values.next_label("ptn_try_end");
    out.push_str("    {\n");
    out.push_str("        PtnTryFrame ");
    out.push_str(&frame_temp);
    out.push_str(";\n");
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

fn module_runtime_requirements(module: &Module) -> RuntimeRequirements {
    let mut requirements = RuntimeRequirements::default();
    collect_instructions_runtime_requirements(
        &module.instructions,
        &module.functions,
        &mut requirements,
    );
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
        Instruction::StoreRef { target, .. } => {
            collect_reference_target_runtime_requirements(target, functions, requirements);
        }
        Instruction::StoreArrayDimRef { target, source } => {
            if let Some(index) = &target.index {
                collect_value_runtime_requirements(index, functions, requirements);
            }
            collect_reference_target_runtime_requirements(source, functions, requirements);
        }
        Instruction::Increment { .. } => {}
        Instruction::UnsetVariable { .. } => {}
        Instruction::UnsetArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                collect_value_runtime_requirements(dimension, functions, requirements);
            }
        }
        Instruction::InternalCall {
            name, arguments, ..
        } => {
            collect_call_runtime_requirements(name, arguments, functions, requirements);
        }
        Instruction::Return { value, .. } => {
            if let Some(value) = value {
                collect_value_runtime_requirements(value, functions, requirements);
            }
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
        Instruction::Foreach { iterable, body, .. } => {
            collect_value_runtime_requirements(iterable, functions, requirements);
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
            if let Some(index) = &target.index {
                collect_value_runtime_requirements(index, functions, requirements);
            }
        }
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
        | ValueExpr::Load { .. }
        | ValueExpr::Constant(_)
        | ValueExpr::MagicConstant { .. } => {}
        ValueExpr::Assign { value, .. } => {
            collect_value_runtime_requirements(value, functions, requirements);
        }
        ValueExpr::Array(elements) => {
            for element in elements {
                if let Some(key) = &element.key {
                    collect_value_runtime_requirements(key, functions, requirements);
                }
                match &element.value {
                    IrArrayElementValue::Value(value) => {
                        collect_value_runtime_requirements(value, functions, requirements);
                    }
                    IrArrayElementValue::Reference(_) => {}
                }
            }
        }
        ValueExpr::ArrayAccess { array, index, .. } => {
            collect_value_runtime_requirements(array, functions, requirements);
            collect_value_runtime_requirements(index, functions, requirements);
        }
        ValueExpr::Isset { targets } => {
            for target in targets {
                collect_value_runtime_requirements(target, functions, requirements);
            }
        }
        ValueExpr::Empty { target } => {
            collect_value_runtime_requirements(target, functions, requirements);
        }
        ValueExpr::InternalCall {
            name, arguments, ..
        } => {
            collect_call_runtime_requirements(name, arguments, functions, requirements);
        }
        ValueExpr::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            collect_value_runtime_requirements(receiver, functions, requirements);
            for argument in arguments {
                collect_value_runtime_requirements(argument, functions, requirements);
            }
        }
        ValueExpr::Unary { expr, .. } | ValueExpr::Cast { expr, .. } => {
            collect_value_runtime_requirements(expr, functions, requirements);
        }
        ValueExpr::Binary { left, right, .. } => {
            collect_value_runtime_requirements(left, functions, requirements);
            collect_value_runtime_requirements(right, functions, requirements);
        }
    }
}

fn collect_call_runtime_requirements(
    name: &str,
    arguments: &[ValueExpr],
    functions: &[FunctionDecl],
    requirements: &mut RuntimeRequirements,
) {
    for argument in arguments {
        collect_value_runtime_requirements(argument, functions, requirements);
    }
    if is_generated_user_function_call(name, functions) {
        return;
    }
    if is_direct_internal_helper_call(name, arguments.len()) {
        requirements.direct_internal_helpers = true;
        return;
    }
    requirements.internal_function_dispatch = true;
}

fn is_generated_user_function_call(name: &str, functions: &[FunctionDecl]) -> bool {
    functions
        .iter()
        .any(|function| function.name.eq_ignore_ascii_case(name))
}

fn is_direct_internal_helper_call(name: &str, argument_count: usize) -> bool {
    (name.eq_ignore_ascii_case("count") && argument_count == 1)
        || (name.eq_ignore_ascii_case("array_key_exists") && argument_count == 2)
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

fn reference_target_from_value(value: &ValueExpr) -> Option<ReferenceTarget> {
    match value {
        ValueExpr::Load { name, .. } => Some(ReferenceTarget::Variable { name: name.clone() }),
        ValueExpr::ArrayAccess { array, index, line } => match array.as_ref() {
            ValueExpr::Load { name, .. } => {
                Some(ReferenceTarget::ArrayDim(crate::ir::ArrayDimTarget {
                    array: name.clone(),
                    index: Some((**index).clone()),
                    line: *line,
                }))
            }
            _ => None,
        },
        _ => None,
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
            | "array_unshift"
            | "end"
            | "next"
            | "prev"
            | "reset"
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
    current_function_name: Option<String>,
    user_functions: Vec<FunctionDecl>,
}

struct ConcatOperand<'a> {
    value: &'a ValueExpr,
    line: usize,
}

impl ValueEmitter {
    fn new(source_file: &str, source_dir: &str, functions: &[FunctionDecl]) -> Self {
        Self::new_with_scope(source_file, source_dir, functions, None)
    }

    fn new_for_function(
        source_file: &str,
        source_dir: &str,
        functions: &[FunctionDecl],
        function_name: &str,
    ) -> Self {
        Self::new_with_scope(source_file, source_dir, functions, Some(function_name))
    }

    fn new_with_scope(
        source_file: &str,
        source_dir: &str,
        functions: &[FunctionDecl],
        current_function_name: Option<&str>,
    ) -> Self {
        Self {
            next_temp: 0,
            next_label: 0,
            source_file: source_file.to_string(),
            source_dir: source_dir.to_string(),
            current_function_name: current_function_name.map(str::to_string),
            user_functions: functions.to_vec(),
        }
    }

    fn direct_user_function(&self, name: &str) -> Option<(usize, &FunctionDecl)> {
        self.user_functions
            .iter()
            .enumerate()
            .find(|(_, function)| function.name.eq_ignore_ascii_case(name))
    }

    fn emit_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Binary {
                op,
                left,
                right,
                line,
            } => self.emit_binary(out, *op, left, right, *line),
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
            ValueExpr::Assign { name, value } => {
                let value_temp = self.emit_materialized_value(out, value);
                out.push_str("    ptn_symbols_set(&runtime.symbols, \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&value_temp);
                out.push_str(");\n");
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_value_clone(");
                out.push_str(&value_temp);
                out.push_str(");\n");
                emit_value_cleanup(out, "    ", &value_temp);
                result_temp
            }
            ValueExpr::Cast { kind, expr, line } => {
                let expr_temp = self.emit_materialized_value(out, expr);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                match kind {
                    CastKind::Int | CastKind::Float | CastKind::String | CastKind::Bool => {
                        out.push_str(match kind {
                            CastKind::Int => "ptn_cast_int",
                            CastKind::Float => "ptn_cast_float",
                            CastKind::String => "ptn_cast_string",
                            CastKind::Bool => "ptn_cast_bool",
                            CastKind::Integer
                            | CastKind::Double
                            | CastKind::Binary
                            | CastKind::Boolean => {
                                unreachable!("non-canonical casts are handled separately")
                            }
                        });
                        out.push('(');
                        out.push_str(&expr_temp);
                        out.push_str(");\n");
                    }
                    CastKind::Integer | CastKind::Double | CastKind::Binary | CastKind::Boolean => {
                        let (spelling, canonical, target) = match kind {
                            CastKind::Integer => ("integer", "int", "PTN_CAST_TARGET_INT"),
                            CastKind::Double => ("double", "float", "PTN_CAST_TARGET_FLOAT"),
                            CastKind::Binary => ("binary", "string", "PTN_CAST_TARGET_STRING"),
                            CastKind::Boolean => ("boolean", "bool", "PTN_CAST_TARGET_BOOL"),
                            CastKind::Int | CastKind::Float | CastKind::String | CastKind::Bool => {
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
                    value.len()
                )
            }
            ValueExpr::Int(value) => format!("ptn_int({value})"),
            ValueExpr::Float(value) => format!("ptn_float({value:?})"),
            ValueExpr::Bool(true) => "ptn_bool(1)".to_string(),
            ValueExpr::Bool(false) => "ptn_bool(0)".to_string(),
            ValueExpr::Null => "ptn_null()".to_string(),
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
            ValueExpr::Isset { targets } => self.emit_isset(out, targets),
            ValueExpr::Empty { target } => self.emit_empty(out, target),
            ValueExpr::Load { name, line } => format!(
                "ptn_runtime_read_variable(&runtime, \"{}\", \"{}\", {})",
                c_string(name),
                c_string(&self.source_file),
                line
            ),
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
                MagicConstantKind::Function | MagicConstantKind::Method => {
                    format!(
                        "ptn_string(\"{}\")",
                        c_string(self.current_function_name.as_deref().unwrap_or(""))
                    )
                }
                MagicConstantKind::Class
                | MagicConstantKind::Trait
                | MagicConstantKind::Namespace => "ptn_string(\"\")".to_string(),
            },
            ValueExpr::InternalCall {
                name,
                arguments,
                line,
            } => self.emit_internal_call(out, name, arguments, *line),
            ValueExpr::MethodCall {
                receiver,
                name,
                arguments,
                line,
            } => self.emit_method_call(out, receiver, name, arguments, *line),
        }
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
            | BinaryOp::ShiftRight => self.emit_runtime_binary(out, op, left, right),
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

    fn emit_runtime_binary(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(binary_runtime_function(op));
        out.push('(');
        out.push_str(&left_temp);
        out.push_str(", ");
        out.push_str(&right_temp);
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
        out.push_str(" = ptn_array_from_literal_entries(");
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
                | ValueExpr::InternalCall { .. }
                | ValueExpr::Unary { .. }
                | ValueExpr::Cast { .. }
                | ValueExpr::Array(_)
                | ValueExpr::ArrayAccess { .. }
                | ValueExpr::Isset { .. }
                | ValueExpr::Empty { .. }
                | ValueExpr::MethodCall { .. }
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

    fn emit_reference_target(&mut self, out: &mut String, target: &ReferenceTarget) -> String {
        match target {
            ReferenceTarget::Variable { name } => {
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_runtime_reference_for_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\");\n");
                temp
            }
            ReferenceTarget::ArrayDim(target) => {
                let index_temp = target
                    .index
                    .as_ref()
                    .map(|index| self.emit_materialized_value(out, index));
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_runtime_reference_for_array_dim(&runtime, \"");
                out.push_str(&c_string(&target.array));
                out.push_str("\", ");
                match &index_temp {
                    Some(index_temp) => {
                        out.push('&');
                        out.push_str(index_temp);
                    }
                    None => out.push_str("NULL"),
                }
                out.push_str(", ");
                out.push_str(&target.line.to_string());
                out.push_str(");\n");
                if let Some(index_temp) = index_temp {
                    emit_value_cleanup(out, "    ", &index_temp);
                }
                temp
            }
        }
    }

    fn emit_internal_call(
        &mut self,
        out: &mut String,
        name: &str,
        arguments: &[ValueExpr],
        line: usize,
    ) -> String {
        if name.eq_ignore_ascii_case("count") && arguments.len() == 1 {
            let argument_temp = self.emit_materialized_value(out, &arguments[0]);
            let result_temp = self.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_count_value(&runtime, ");
            out.push_str(&argument_temp);
            out.push_str(");\n");
            emit_value_cleanup(out, "    ", &argument_temp);
            return result_temp;
        }

        if name.eq_ignore_ascii_case("array_key_exists") && arguments.len() == 2 {
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

        if let Some(result_temp) = self.emit_variable_array_mutator_call(out, name, arguments) {
            return result_temp;
        }

        let result_temp = self.next_temp();
        let direct_user = self
            .direct_user_function(name)
            .map(|(index, function)| (user_function_c_name(index), function.parameters.clone()));
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            if let Some((c_name, _)) = &direct_user {
                out.push_str(&c_name);
                out.push_str("(&runtime, 0, NULL, ");
            } else {
                out.push_str("ptn_call_function(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", 0, NULL, ");
            }
            out.push_str(&line.to_string());
            out.push_str(");\n");
            return result_temp;
        }

        let mut temps = Vec::with_capacity(arguments.len());
        for (argument_index, argument) in arguments.iter().enumerate() {
            let by_ref_parameter = direct_user
                .as_ref()
                .and_then(|(_, parameters)| parameters.get(argument_index))
                .filter(|parameter| parameter.by_ref);
            if let Some(parameter) = by_ref_parameter {
                temps.push(self.emit_by_ref_call_argument(
                    out,
                    argument,
                    name,
                    argument_index,
                    &parameter.name,
                ));
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
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        if let Some((c_name, _)) = &direct_user {
            out.push_str(&c_name);
            out.push_str("(&runtime, ");
        } else {
            out.push_str("ptn_call_function(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
        }
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
        result_temp
    }

    fn emit_variable_array_mutator_call(
        &mut self,
        out: &mut String,
        name: &str,
        arguments: &[ValueExpr],
    ) -> Option<String> {
        let ValueExpr::Load {
            name: variable_name,
            ..
        } = arguments.first()?
        else {
            return None;
        };

        let helper = if arguments.len() == 1 {
            if name.eq_ignore_ascii_case("array_pop") {
                Some("ptn_runtime_array_pop_variable")
            } else if name.eq_ignore_ascii_case("array_shift") {
                Some("ptn_runtime_array_shift_variable")
            } else if name.eq_ignore_ascii_case("next") {
                Some("ptn_runtime_array_next_variable")
            } else if name.eq_ignore_ascii_case("end") {
                Some("ptn_runtime_array_end_variable")
            } else if name.eq_ignore_ascii_case("prev") {
                Some("ptn_runtime_array_prev_variable")
            } else if name.eq_ignore_ascii_case("reset") {
                Some("ptn_runtime_array_reset_variable")
            } else {
                None
            }
        } else {
            None
        };

        if let Some(helper) = helper {
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

        let helper = if name.eq_ignore_ascii_case("array_push") {
            Some("ptn_runtime_array_push_variable")
        } else if name.eq_ignore_ascii_case("array_unshift") {
            Some("ptn_runtime_array_unshift_variable")
        } else {
            None
        };
        let Some(helper) = helper else {
            return None;
        };

        let array_temp = self.emit_materialized_value(out, &arguments[0]);
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
        out.push_str(helper);
        out.push_str("(&runtime, \"");
        out.push_str(&c_string(variable_name));
        out.push_str("\", ");
        out.push_str(&array_temp);
        out.push_str(", ");
        out.push_str(&value_temps.len().to_string());
        out.push_str(", ");
        if let Some(values_temp) = &values_temp {
            out.push_str(values_temp);
        } else {
            out.push_str("NULL");
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
        emit_value_cleanup(out, "    ", &array_temp);
        Some(result_temp)
    }

    fn emit_by_ref_call_argument(
        &mut self,
        out: &mut String,
        argument: &ValueExpr,
        function_name: &str,
        argument_index: usize,
        parameter_name: &str,
    ) -> String {
        match reference_target_from_value(argument) {
            Some(target) => self.emit_reference_target(out, &target),
            None => {
                let temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&temp);
                out.push_str(" = ptn_null();\n");
                out.push_str("    ptn_abort_by_reference_argument_error(\"");
                out.push_str(&c_string(function_name));
                out.push_str("\", ");
                out.push_str(&(argument_index + 1).to_string());
                out.push_str(", \"");
                out.push_str(&c_string(parameter_name));
                out.push_str("\");\n");
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
        line: usize,
    ) -> String {
        let receiver_temp = self.emit_materialized_value(out, receiver);
        let result_temp = self.next_temp();
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_method(&runtime, ");
            out.push_str(&receiver_temp);
            out.push_str(", \"");
            out.push_str(&c_string(name));
            out.push_str("\", 0, NULL, ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
            return result_temp;
        }

        let mut temps = Vec::with_capacity(arguments.len());
        for argument in arguments {
            temps.push(self.emit_materialized_value(out, argument));
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
        out.push_str(" = ptn_call_method(&runtime, ");
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
        for index in 0..temps.len() {
            emit_value_cleanup(out, "    ", &format!("{args_temp}[{index}]"));
        }
        for temp in temps {
            emit_value_cleanup(out, "    ", &temp);
        }
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
    for byte in value.bytes() {
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
    out
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
