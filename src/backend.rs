use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{
    ArrayElement as IrArrayElement, BinaryOp, CastKind, FunctionDecl, IncDecOp, Instruction,
    MagicConstantKind, Module, TypeHint, UnaryOp, ValueExpr,
};

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    out.push_str(RUNTIME_C);
    emit_user_function_prototypes(&mut out, &module.functions);
    emit_user_functions(
        &mut out,
        &module.functions,
        &module.source_file,
        &module.source_dir,
    );
    emit_user_function_dispatch(&mut out, &module.functions);
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
    let mut values = ValueEmitter::new(&module.source_file, &module.source_dir);
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

fn emit_user_function_prototypes(out: &mut String, functions: &[FunctionDecl]) {
    out.push_str(
        "\nstatic PTN_UNUSED PtnValue ptn_call_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line);\n",
    );
    for (index, _) in functions.iter().enumerate() {
        out.push_str("static PtnValue ");
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
        out.push_str("\nstatic PtnValue ");
        out.push_str(&c_name);
        out.push_str(
            "(PtnRuntime *caller_runtime, size_t argc, const PtnValue *args, size_t line) {\n",
        );
        out.push_str("    (void)line;\n");
        if function.parameters.is_empty() {
            out.push_str("    (void)argc;\n");
            out.push_str("    (void)args;\n");
        } else {
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
        out.push_str("    ptn_runtime_init(&runtime);\n");
        out.push_str("    ptn_runtime_import_constants(&runtime, caller_runtime);\n");
        out.push_str("    PtnValue ptn_return_value = ptn_null();\n");
        for (parameter_index, parameter) in function.parameters.iter().enumerate() {
            if let Some(TypeHint::Null) = parameter.type_hint {
                out.push_str("    if (args[");
                out.push_str(&parameter_index.to_string());
                out.push_str("].type != PTN_NULL) {\n");
                out.push_str("        ptn_emit_type_error(&caller_runtime->diagnostics, \"");
                out.push_str(&c_string(&function.name));
                out.push_str("() argument $");
                out.push_str(&c_string(&parameter.name));
                out.push_str(" must be of type null\");\n");
                out.push_str("        ptn_runtime_free(&runtime);\n");
                out.push_str("        exit(255);\n");
                out.push_str("    }\n");
            }
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(&parameter.name));
            out.push_str("\", args[");
            out.push_str(&parameter_index.to_string());
            out.push_str("]);\n");
        }
        let mut values = ValueEmitter::new(source_file, source_dir);
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
            let emitted_value = values.emit_value(out, value);
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&emitted_value);
            out.push_str(");\n");
        }
        Instruction::DefineConstant { name, value, line } => {
            let emitted_value = values.emit_value(out, value);
            out.push_str("    (void)ptn_runtime_define_constant_if_absent(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&emitted_value);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
        }
        Instruction::Expression(value) => {
            let emitted_value = values.emit_materialized_value(out, value);
            out.push_str("    (void)");
            out.push_str(&emitted_value);
            out.push_str(";\n");
        }
        Instruction::Echo(value) => {
            let emitted_value = values.emit_value(out, value);
            out.push_str("    ptn_echo(");
            out.push_str(&emitted_value);
            out.push_str(");\n");
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
        }
        Instruction::Return { value, .. } => match return_target {
            Some(target) => {
                let result_value = value
                    .as_ref()
                    .map(|value| values.emit_value(out, value))
                    .unwrap_or_else(|| "ptn_null()".to_string());
                out.push_str("    ptn_return_value = ");
                out.push_str(&result_value);
                out.push_str(";\n");
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
                }
                out.push_str("    ptn_runtime_free(&runtime);\n");
                out.push_str("    return 0;\n");
            }
        },
        Instruction::Branch {
            condition,
            then_body,
            else_body,
        } => {
            let condition_temp = values.emit_materialized_value(out, condition);
            out.push_str("    if (ptn_is_truthy(");
            out.push_str(&condition_temp);
            out.push_str(")) {\n");
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
            let condition_temp = values.emit_materialized_value(out, condition);
            out.push_str("        if (!ptn_is_truthy(");
            out.push_str(&condition_temp);
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
            let condition_temp = values.emit_materialized_value(out, condition);
            out.push_str("        if (!ptn_is_truthy(");
            out.push_str(&condition_temp);
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
                let condition_temp = values.emit_materialized_value(out, condition);
                out.push_str("        if (!ptn_is_truthy(");
                out.push_str(&condition_temp);
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
            body,
            line,
        } => {
            let end_label = values.next_label("ptn_foreach_end");
            let continue_label = values.next_label("ptn_foreach_continue");
            let iterable_temp = values.emit_materialized_value(out, iterable);
            let iterator_temp = values.next_temp();
            out.push_str("    PtnArrayIterator ");
            out.push_str(&iterator_temp);
            out.push_str(" = ptn_array_iterator_from_value(&runtime, ");
            out.push_str(&iterable_temp);
            out.push_str(", ");
            out.push_str(&line.to_string());
            out.push_str(");\n");
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
            }
            let value_temp = values.next_temp();
            out.push_str("        PtnValue ");
            out.push_str(&value_temp);
            out.push_str(" = ptn_array_iterator_current_value(&");
            out.push_str(&iterator_temp);
            out.push_str(");\n");
            out.push_str("        ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(value));
            out.push_str("\", ");
            out.push_str(&value_temp);
            out.push_str(");\n");
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
            out.push_str("        ptn_array_iterator_advance(&");
            out.push_str(&iterator_temp);
            out.push_str(");\n");
            out.push_str("    }\n");
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
            Instruction::Continue { level, line } => {
                if *level > 0 && *level <= contexts.len() {
                    let target_index = contexts.len() - *level;
                    if contexts[target_index] == ControlTargetKind::Switch {
                        warnings.push(ControlWarning {
                            message: continue_targeting_switch_warning(
                                *level,
                                contexts,
                                target_index,
                            ),
                            line: *line,
                        });
                    }
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
            out.push_str("        if (ptn_compare_equal(");
            out.push_str(&switch_temp);
            out.push_str(", ");
            out.push_str(&condition_temp);
            out.push_str(")) {\n");
            out.push_str("            goto ");
            out.push_str(label);
            out.push_str(";\n");
            out.push_str("        }\n");
            out.push_str("    }\n");
        } else {
            default_label = Some(label.as_str());
        }
    }

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
    let status = Command::new("cc")
        .arg("-std=c11")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-O2")
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

struct ValueEmitter {
    next_temp: usize,
    next_label: usize,
    source_file: String,
    source_dir: String,
}

impl ValueEmitter {
    fn new(source_file: &str, source_dir: &str) -> Self {
        Self {
            next_temp: 0,
            next_label: 0,
            source_file: source_file.to_string(),
            source_dir: source_dir.to_string(),
        }
    }

    fn emit_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Binary { op, left, right } => self.emit_binary(out, *op, left, right),
            ValueExpr::Unary { op, expr } => {
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
                });
                out.push('(');
                out.push_str(&expr_temp);
                out.push_str(");\n");
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
                result_temp
            }
            ValueExpr::String(value) => format!("ptn_string(\"{}\")", c_string(value)),
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
                    format!("ptn_string(\"{}\")", c_string(&self.source_file))
                }
                MagicConstantKind::Dir => {
                    format!("ptn_string(\"{}\")", c_string(&self.source_dir))
                }
                MagicConstantKind::Function
                | MagicConstantKind::Method
                | MagicConstantKind::Class
                | MagicConstantKind::Trait
                | MagicConstantKind::Namespace => "ptn_string(\"\")".to_string(),
            },
            ValueExpr::InternalCall {
                name,
                arguments,
                line,
            } => self.emit_internal_call(out, name, arguments, *line),
        }
    }

    fn emit_binary(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Power
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::Concat
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
        out.push_str(match op {
            BinaryOp::Add => "ptn_add",
            BinaryOp::Subtract => "ptn_subtract",
            BinaryOp::Multiply => "ptn_multiply",
            BinaryOp::Power => "ptn_power",
            BinaryOp::Divide => "ptn_divide",
            BinaryOp::Modulo => "ptn_modulo",
            BinaryOp::Concat => "ptn_concat",
            BinaryOp::BitwiseAnd => "ptn_bitwise_and",
            BinaryOp::BitwiseXor => "ptn_bitwise_xor",
            BinaryOp::BitwiseOr => "ptn_bitwise_or",
            BinaryOp::ShiftLeft => "ptn_shift_left",
            BinaryOp::ShiftRight => "ptn_shift_right",
            _ => unreachable!(),
        });
        out.push('(');
        out.push_str(&left_temp);
        out.push_str(", ");
        out.push_str(&right_temp);
        out.push_str(");\n");
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
                format!("!ptn_compare_identical({left_temp}, {right_temp})")
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
        result_temp
    }

    fn emit_isset(&mut self, out: &mut String, targets: &[ValueExpr]) -> String {
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(1);\n");
        for target in targets {
            out.push_str("    if (ptn_is_truthy(");
            out.push_str(&result_temp);
            out.push_str(")) {\n");
            let lookup_temp = self.emit_quiet_lookup(out, target);
            out.push_str("        ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_bool(");
            out.push_str(&lookup_temp);
            out.push_str(".exists && ");
            out.push_str(&lookup_temp);
            out.push_str(".value.type != PTN_NULL);\n");
            out.push_str("    }\n");
        }
        result_temp
    }

    fn emit_empty(&mut self, out: &mut String, target: &ValueExpr) -> String {
        let lookup_temp = self.emit_quiet_lookup(out, target);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(!");
        out.push_str(&lookup_temp);
        out.push_str(".exists || !ptn_is_truthy(");
        out.push_str(&lookup_temp);
        out.push_str(".value));\n");
        result_temp
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
        for element in elements {
            let (has_key, key_temp) = if let Some(key) = &element.key {
                ("1", self.emit_materialized_value(out, key))
            } else {
                ("0", "ptn_null()".to_string())
            };
            let value_temp = self.emit_materialized_value(out, &element.value);
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
        result_temp
    }

    fn emit_short_circuit(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (ptn_is_truthy(");
        out.push_str(&left_temp);
        out.push_str(")) {\n");
        match op {
            BinaryOp::And => {
                let right_value = self.emit_value(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(ptn_is_truthy(");
                out.push_str(&right_value);
                out.push_str("));\n");
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
                let right_value = self.emit_value(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(ptn_is_truthy(");
                out.push_str(&right_value);
                out.push_str("));\n");
            }
            _ => unreachable!(),
        }
        out.push_str("    }\n");
        result_temp
    }

    fn emit_boolean_xor(
        &mut self,
        out: &mut String,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(ptn_is_truthy(");
        out.push_str(&left_temp);
        out.push_str(") != ptn_is_truthy(");
        out.push_str(&right_temp);
        out.push_str("));\n");
        result_temp
    }

    fn emit_materialized_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        if matches!(
            value,
            ValueExpr::Binary { .. }
                | ValueExpr::InternalCall { .. }
                | ValueExpr::Unary { .. }
                | ValueExpr::Cast { .. }
                | ValueExpr::Array(_)
                | ValueExpr::ArrayAccess { .. }
                | ValueExpr::Isset { .. }
                | ValueExpr::Empty { .. }
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

    fn emit_internal_call(
        &mut self,
        out: &mut String,
        name: &str,
        arguments: &[ValueExpr],
        line: usize,
    ) -> String {
        let result_temp = self.next_temp();
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_function(&runtime, \"");
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
        out.push_str(&temps.join(", "));
        out.push_str(" };\n");
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_call_function(&runtime, \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&arguments.len().to_string());
        out.push_str(", ");
        out.push_str(&args_temp);
        out.push_str(", ");
        out.push_str(&line.to_string());
        out.push_str(");\n");
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
            _ => out.push_str(&format!("\\x{byte:02x}")),
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

const RUNTIME_C: &str = r#"#include <ctype.h>
#include <errno.h>
#include <math.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <process.h>
#else
#include <unistd.h>
#endif

#if defined(__GNUC__) || defined(__clang__)
#define PTN_UNUSED __attribute__((unused))
#else
#define PTN_UNUSED
#endif

#define PTN_PHP_VERSION "8.4.0"
#define PTN_PHP_SAPI_NAME "cli"
#define PTN_ARRAY_INDEX_MIN_ENTRIES 16

typedef struct PtnArray PtnArray;

typedef enum {
    PTN_NULL,
    PTN_BOOL,
    PTN_INT,
    PTN_FLOAT,
    PTN_STRING,
    PTN_ARRAY
} PtnType;

typedef enum {
    PTN_ARRAY_KEY_INT,
    PTN_ARRAY_KEY_STRING
} PtnArrayKeyType;

typedef struct {
    PtnArrayKeyType type;
    union {
        int64_t integer;
        const char *string;
    } as;
} PtnArrayKey;

typedef struct {
    PtnType type;
    union {
        int boolean;
        int64_t integer;
        double floating;
        const char *string;
        PtnArray *array;
    } as;
} PtnValue;

typedef struct {
    int exists;
    PtnValue value;
} PtnLookupResult;

typedef struct {
    PtnArrayKey key;
    PtnValue value;
} PtnArrayEntry;

typedef struct {
    int occupied;
    uint64_t hash;
    size_t entry_index;
} PtnArrayIndexSlot;

typedef struct {
    PtnArray *array;
    size_t index;
    int valid;
} PtnArrayIterator;

struct PtnArray {
    size_t len;
    size_t capacity;
    PtnArrayEntry *entries;
    PtnArrayIndexSlot *index_slots;
    size_t index_capacity;
    int64_t next_auto_key;
};

typedef struct {
    int has_key;
    PtnValue key;
    PtnValue value;
} PtnArrayLiteralEntry;

typedef enum {
    PTN_NUMBER_INT,
    PTN_NUMBER_FLOAT
} PtnNumberType;

typedef struct {
    PtnNumberType type;
    int64_t integer;
    double floating;
} PtnNumber;

typedef struct {
    const char *data;
    char *owned;
    size_t len;
} PtnStringOperand;

typedef struct {
    char *name;
    PtnValue value;
} PtnSymbol;

typedef struct {
    PtnSymbol *items;
    size_t len;
    size_t capacity;
} PtnSymbolTable;

typedef struct {
    FILE *stream;
    int emitted_deprecation;
} PtnDiagnosticSink;

typedef struct {
    PtnSymbolTable symbols;
    PtnSymbolTable constants;
    PtnDiagnosticSink diagnostics;
} PtnRuntime;

typedef PtnValue (*PtnInternalFunctionHandler)(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);

typedef struct {
    const char *name;
    size_t min_args;
    size_t max_args;
    PtnInternalFunctionHandler handler;
} PtnInternalFunction;

#define PTN_VARIADIC_ARGS ((size_t)-1)

static PTN_UNUSED PtnValue ptn_null(void) {
    PtnValue value;
    value.type = PTN_NULL;
    return value;
}

static PTN_UNUSED PtnValue ptn_bool(int boolean) {
    PtnValue value;
    value.type = PTN_BOOL;
    value.as.boolean = boolean ? 1 : 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_int(int64_t integer) {
    PtnValue value;
    value.type = PTN_INT;
    value.as.integer = integer;
    return value;
}

static PTN_UNUSED PtnValue ptn_float(double floating) {
    PtnValue value;
    value.type = PTN_FLOAT;
    value.as.floating = floating;
    return value;
}

static PTN_UNUSED PtnValue ptn_string(const char *string) {
    PtnValue value;
    value.type = PTN_STRING;
    value.as.string = string;
    return value;
}

static PTN_UNUSED PtnValue ptn_owned_string(char *string) {
    PtnValue value;
    value.type = PTN_STRING;
    value.as.string = string;
    return value;
}

static PTN_UNUSED PtnValue ptn_array(PtnArray *array) {
    PtnValue value;
    value.type = PTN_ARRAY;
    value.as.array = array;
    return value;
}

static PTN_UNUSED PtnLookupResult ptn_lookup_missing(void) {
    PtnLookupResult result;
    result.exists = 0;
    result.value = ptn_null();
    return result;
}

static PTN_UNUSED PtnLookupResult ptn_lookup_found(PtnValue value) {
    PtnLookupResult result;
    result.exists = 1;
    result.value = value;
    return result;
}

static void ptn_abort_out_of_memory(void) {
    fputs("Fatal error: out of memory\n", stderr);
    exit(1);
}

static PTN_UNUSED char *ptn_duplicate_string(const char *string) {
    size_t len = strlen(string);
    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(copy, string, len + 1);
    return copy;
}

static PTN_UNUSED PtnArrayKey ptn_array_int_key(int64_t integer) {
    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_INT;
    key.as.integer = integer;
    return key;
}

static PTN_UNUSED PtnArrayKey ptn_array_string_key(const char *string) {
    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_STRING;
    key.as.string = ptn_duplicate_string(string);
    return key;
}

static PTN_UNUSED int ptn_string_is_integer_array_key(const char *string, int64_t *integer) {
    if (*string == '\0' || *string == '+') {
        return 0;
    }
    if (strcmp(string, "-0") == 0) {
        return 0;
    }

    const char *digits = string;
    if (*digits == '-') {
        digits++;
    }
    if (*digits == '\0') {
        return 0;
    }
    if (*digits == '0' && digits[1] != '\0') {
        return 0;
    }
    for (const char *cursor = digits; *cursor != '\0'; cursor++) {
        if (!isdigit((unsigned char)*cursor)) {
            return 0;
        }
    }

    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(string, &end, 10);
    if (errno == ERANGE || end == string || *end != '\0') {
        return 0;
    }
    *integer = (int64_t)parsed;
    return 1;
}

static PTN_UNUSED void ptn_abort_illegal_array_key(void) {
    fputs("Fatal error: Illegal offset type\n", stderr);
    exit(255);
}

static PTN_UNUSED PtnArrayKey ptn_array_key_from_value(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_array_string_key("");
        case PTN_BOOL:
            return ptn_array_int_key(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_array_int_key(value.as.integer);
        case PTN_FLOAT:
            return ptn_array_int_key((int64_t)value.as.floating);
        case PTN_STRING: {
            int64_t integer = 0;
            if (ptn_string_is_integer_array_key(value.as.string, &integer)) {
                return ptn_array_int_key(integer);
            }
            return ptn_array_string_key(value.as.string);
        }
        case PTN_ARRAY:
            ptn_abort_illegal_array_key();
    }
    return ptn_array_string_key("");
}

static PTN_UNUSED void ptn_array_key_free(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_STRING) {
        free((char *)key.as.string);
    }
}

static PTN_UNUSED int ptn_array_keys_equal(PtnArrayKey left, PtnArrayKey right) {
    if (left.type != right.type) {
        return 0;
    }
    if (left.type == PTN_ARRAY_KEY_INT) {
        return left.as.integer == right.as.integer;
    }
    return strcmp(left.as.string, right.as.string) == 0;
}

static PTN_UNUSED uint64_t ptn_hash_mix_uint64(uint64_t value) {
    value ^= value >> 30;
    value *= 0xbf58476d1ce4e5b9ULL;
    value ^= value >> 27;
    value *= 0x94d049bb133111ebULL;
    value ^= value >> 31;
    return value;
}

static PTN_UNUSED uint64_t ptn_array_key_hash(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_hash_mix_uint64((uint64_t)key.as.integer ^ 0x9e3779b97f4a7c15ULL);
    }

    uint64_t hash = 1469598103934665603ULL ^ 0x517cc1b727220a95ULL;
    for (const unsigned char *cursor = (const unsigned char *)key.as.string; *cursor != '\0'; cursor++) {
        hash ^= (uint64_t)*cursor;
        hash *= 1099511628211ULL;
    }
    return ptn_hash_mix_uint64(hash);
}

static PTN_UNUSED void ptn_array_index_init(PtnArray *array, size_t expected_entries) {
    array->index_slots = NULL;
    array->index_capacity = 0;

    if (expected_entries < PTN_ARRAY_INDEX_MIN_ENTRIES) {
        return;
    }
    if (expected_entries > SIZE_MAX / 2) {
        ptn_abort_out_of_memory();
    }

    size_t wanted = expected_entries * 2;
    size_t capacity = PTN_ARRAY_INDEX_MIN_ENTRIES;
    while (capacity < wanted) {
        if (capacity > SIZE_MAX / 2) {
            ptn_abort_out_of_memory();
        }
        capacity *= 2;
    }

    array->index_slots = calloc(capacity, sizeof(PtnArrayIndexSlot));
    if (array->index_slots == NULL) {
        ptn_abort_out_of_memory();
    }
    array->index_capacity = capacity;
}

static PTN_UNUSED size_t ptn_array_linear_find_key(PtnArray *array, PtnArrayKey key) {
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_array_keys_equal(array->entries[i].key, key)) {
            return i;
        }
    }
    return array->len;
}

static PTN_UNUSED size_t ptn_array_index_slot_for_key(PtnArray *array, PtnArrayKey key, uint64_t hash) {
    size_t mask = array->index_capacity - 1;
    size_t slot_index = (size_t)hash & mask;
    for (;;) {
        PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
        if (!slot->occupied ||
            (slot->hash == hash && ptn_array_keys_equal(array->entries[slot->entry_index].key, key))) {
            return slot_index;
        }
        slot_index = (slot_index + 1) & mask;
    }
}

static PTN_UNUSED void ptn_array_update_next_auto_key(PtnArray *array, PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT &&
        key.as.integer >= array->next_auto_key &&
        key.as.integer < INT64_MAX) {
        array->next_auto_key = key.as.integer + 1;
    }
}

static PTN_UNUSED size_t ptn_array_find_key(PtnArray *array, PtnArrayKey key) {
    if (array->index_capacity != 0) {
        uint64_t hash = ptn_array_key_hash(key);
        size_t slot_index = ptn_array_index_slot_for_key(array, key, hash);
        PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
        return slot->occupied ? slot->entry_index : array->len;
    }
    return ptn_array_linear_find_key(array, key);
}

static PTN_UNUSED void ptn_array_index_insert(PtnArray *array, PtnArrayKey key, size_t entry_index) {
    if (array->index_capacity == 0) {
        return;
    }
    uint64_t hash = ptn_array_key_hash(key);
    size_t slot_index = ptn_array_index_slot_for_key(array, key, hash);
    PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
    if (!slot->occupied) {
        slot->occupied = 1;
        slot->hash = hash;
        slot->entry_index = entry_index;
    }
}

static PTN_UNUSED void ptn_array_set_entry(PtnArray *array, PtnArrayKey key, PtnValue value) {
    size_t index = ptn_array_find_key(array, key);
    ptn_array_update_next_auto_key(array, key);
    if (index < array->len) {
        array->entries[index].value = value;
        ptn_array_key_free(key);
        return;
    }
    if (array->len == array->capacity) {
        ptn_abort_out_of_memory();
    }
    size_t entry_index = array->len;
    array->entries[entry_index].key = key;
    array->entries[entry_index].value = value;
    array->len++;
    ptn_array_index_insert(array, key, entry_index);
}

static PTN_UNUSED PtnValue ptn_array_from_literal_entries(size_t entry_count, const PtnArrayLiteralEntry *entries) {
    PtnArray *array = malloc(sizeof(PtnArray));
    if (array == NULL) {
        ptn_abort_out_of_memory();
    }
    array->len = 0;
    array->capacity = entry_count;
    array->entries = NULL;
    array->index_slots = NULL;
    array->index_capacity = 0;
    array->next_auto_key = 0;
    if (entry_count != 0) {
        array->entries = malloc(entry_count * sizeof(PtnArrayEntry));
        if (array->entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }
    ptn_array_index_init(array, entry_count);

    for (size_t i = 0; i < entry_count; i++) {
        PtnArrayKey key = entries[i].has_key
            ? ptn_array_key_from_value(entries[i].key)
            : ptn_array_int_key(array->next_auto_key);
        ptn_array_set_entry(array, key, entries[i].value);
    }
    return ptn_array(array);
}

static void ptn_symbols_init(PtnSymbolTable *symbols) {
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
}

static void ptn_symbols_free(PtnSymbolTable *symbols) {
    for (size_t i = 0; i < symbols->len; i++) {
        free(symbols->items[i].name);
    }
    free(symbols->items);
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
}

static size_t ptn_symbols_find(PtnSymbolTable *symbols, const char *name) {
    for (size_t i = 0; i < symbols->len; i++) {
        if (strcmp(symbols->items[i].name, name) == 0) {
            return i;
        }
    }
    return symbols->len;
}

static PTN_UNUSED void ptn_symbols_set(PtnSymbolTable *symbols, const char *name, PtnValue value) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        symbols->items[index].value = value;
        return;
    }
    if (symbols->len == symbols->capacity) {
        size_t new_capacity = symbols->capacity == 0 ? 8 : symbols->capacity * 2;
        PtnSymbol *new_items = realloc(symbols->items, new_capacity * sizeof(PtnSymbol));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        symbols->items = new_items;
        symbols->capacity = new_capacity;
    }
    symbols->items[symbols->len].name = ptn_duplicate_string(name);
    symbols->items[symbols->len].value = value;
    symbols->len++;
}

static int ptn_symbols_get(PtnSymbolTable *symbols, const char *name, PtnValue *out) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        *out = symbols->items[index].value;
        return 1;
    }
    return 0;
}

static void ptn_diagnostics_init(PtnDiagnosticSink *diagnostics, FILE *stream) {
    diagnostics->stream = stream;
    diagnostics->emitted_deprecation = 0;
}

static void ptn_emit_undefined_variable_warning(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    const char *path,
    size_t line
) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Warning: Undefined variable $", stream);
    fputs(name, stream);
    fputs(" in ", stream);
    fputs(path, stream);
    fputs(" on line ", stream);
    fprintf(stream, "%zu", line);
    fputc('\n', stream);
}

static void ptn_emit_undefined_function_error(PtnDiagnosticSink *diagnostics, const char *name) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: Call to undefined function ", stream);
    fputs(name, stream);
    fputs("()\n", stream);
}

static void ptn_emit_undefined_constant_error(PtnDiagnosticSink *diagnostics, const char *name) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: Undefined constant \"", stream);
    fputs(name, stream);
    fputs("\"\n", stream);
}

static void ptn_emit_argument_count_error(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t min_args,
    size_t argc
) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(name, stream);
    fputs("() expects at least ", stream);
    fprintf(stream, "%zu", min_args);
    fputs(" argument", stream);
    if (min_args != 1) {
        fputc('s', stream);
    }
    fputs(", ", stream);
    fprintf(stream, "%zu", argc);
    fputs(" given\n", stream);
}

static void ptn_emit_too_many_arguments_error(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t max_args,
    size_t argc
) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(name, stream);
    fputs("() expects at most ", stream);
    fprintf(stream, "%zu", max_args);
    fputs(" argument", stream);
    if (max_args != 1) {
        fputc('s', stream);
    }
    fputs(", ", stream);
    fprintf(stream, "%zu", argc);
    fputs(" given\n", stream);
}

static PTN_UNUSED void ptn_emit_type_error(PtnDiagnosticSink *diagnostics, const char *message) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(message, stream);
    fputc('\n', stream);
}

static void ptn_emit_deprecation(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (diagnostics->emitted_deprecation) {
        fputc('\n', stdout);
    }
    diagnostics->emitted_deprecation = 1;
    fputs("Deprecated: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    (void)diagnostics;
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_control_warning(const char *message, const char *path, size_t line) {
    fputc('\n', stdout);
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ", stdout);
    fputs(path, stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static void ptn_emit_constant_already_defined_warning(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t line
) {
    (void)diagnostics;
    fputs("Warning: Constant ", stdout);
    fputs(name, stdout);
    fputs(" already defined, this will be an error in PHP 9 in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static void ptn_runtime_init(PtnRuntime *runtime) {
    ptn_symbols_init(&runtime->symbols);
    ptn_symbols_init(&runtime->constants);
    ptn_diagnostics_init(&runtime->diagnostics, stderr);
}

static void ptn_runtime_free(PtnRuntime *runtime) {
    ptn_symbols_free(&runtime->constants);
    ptn_symbols_free(&runtime->symbols);
}

static PTN_UNUSED void ptn_runtime_write_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_symbols_set(&runtime->symbols, name, value);
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return value;
    }
    ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    return ptn_null();
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_variable_quiet(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return ptn_lookup_found(value);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED void ptn_runtime_define_constant(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_symbols_set(&runtime->constants, name, value);
}

static PTN_UNUSED void ptn_runtime_import_constants(PtnRuntime *runtime, PtnRuntime *source) {
    for (size_t i = 0; i < source->constants.len; i++) {
        PtnSymbol *constant = &source->constants.items[i];
        ptn_runtime_define_constant(runtime, constant->name, constant->value);
    }
}

static PTN_UNUSED PtnNumber ptn_number_int(int64_t integer) {
    PtnNumber number;
    number.type = PTN_NUMBER_INT;
    number.integer = integer;
    number.floating = (double)integer;
    return number;
}

static PTN_UNUSED PtnNumber ptn_number_float(double floating) {
    PtnNumber number;
    number.type = PTN_NUMBER_FLOAT;
    number.integer = 0;
    number.floating = floating;
    return number;
}

static PTN_UNUSED int ptn_contains_float_marker(const char *start, const char *end) {
    for (const char *cursor = start; cursor < end; cursor++) {
        if (*cursor == '.' || *cursor == 'e' || *cursor == 'E') {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED PtnNumber ptn_string_to_number(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return ptn_number_int(0);
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return ptn_number_int(0);
    }

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        return ptn_number_int((int64_t)integer);
    }
    return ptn_number_float(floating);
}

static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_number_int(0);
        case PTN_BOOL:
            return ptn_number_int(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_number_int(value.as.integer);
        case PTN_FLOAT:
            return ptn_number_float(value.as.floating);
        case PTN_STRING:
            return ptn_string_to_number(value.as.string);
        case PTN_ARRAY:
            return ptn_number_int(value.as.array->len == 0 ? 0 : 1);
    }
    return ptn_number_int(0);
}

static PTN_UNUSED PtnValue ptn_negate(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(-number.floating);
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(-(double)number.integer);
    }
    return ptn_int(-number.integer);
}

static PTN_UNUSED PtnValue ptn_positive(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED int ptn_is_truthy(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return 0;
        case PTN_BOOL:
            return value.as.boolean != 0;
        case PTN_INT:
            return value.as.integer != 0;
        case PTN_FLOAT:
            return value.as.floating != 0.0;
        case PTN_STRING:
            return value.as.string[0] != '\0' && strcmp(value.as.string, "0") != 0;
        case PTN_ARRAY:
            return value.as.array->len != 0;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_not(PtnValue value) {
    return ptn_bool(!ptn_is_truthy(value));
}

static PTN_UNUSED PtnValue ptn_cast_int(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_int((int64_t)number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED PtnValue ptn_cast_float(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    return ptn_float(number.floating);
}

static PTN_UNUSED void ptn_abort_arithmetic_error(const char *message) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(1);
}

static PTN_UNUSED void ptn_abort_control_error(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED int ptn_is_number_type(PtnValue value) {
    return value.type == PTN_INT || value.type == PTN_FLOAT;
}

static PTN_UNUSED int ptn_is_numeric_string(const char *string, double *number) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    double parsed = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    if (*end != '\0') {
        return 0;
    }
    *number = parsed;
    return 1;
}

static PTN_UNUSED int ptn_comparison_numeric_value(PtnValue value, double *number) {
    switch (value.type) {
        case PTN_INT:
            *number = (double)value.as.integer;
            return 1;
        case PTN_FLOAT:
            *number = value.as.floating;
            return 1;
        case PTN_STRING:
            return ptn_is_numeric_string(value.as.string, number);
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_ARRAY:
            return 0;
    }
    return 0;
}

enum {
    PTN_COMPARE_LESS = -1,
    PTN_COMPARE_EQUAL = 0,
    PTN_COMPARE_GREATER = 1,
    PTN_COMPARE_UNORDERED = 2
};

static PTN_UNUSED int ptn_compare_numbers(double left, double right) {
    if (isnan(left) || isnan(right)) {
        return PTN_COMPARE_UNORDERED;
    }
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_strings(const char *left, const char *right) {
    int compared = strcmp(left, right);
    return compared < 0 ? -1 : (compared > 0 ? 1 : 0);
}

static PTN_UNUSED void ptn_number_value_to_string(PtnValue value, char *buffer, size_t buffer_len) {
    if (value.type == PTN_INT) {
        snprintf(buffer, buffer_len, "%lld", (long long)value.as.integer);
    } else {
        snprintf(buffer, buffer_len, "%.14g", value.as.floating);
    }
}

static PTN_UNUSED int ptn_compare_number_and_string(PtnValue number, const char *string, int number_is_left) {
    char number_string[128];
    ptn_number_value_to_string(number, number_string, sizeof(number_string));
    int compared = ptn_compare_strings(number_string, string);
    return number_is_left ? compared : -compared;
}

static PTN_UNUSED int ptn_compare_equal(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_identical(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right);

static PTN_UNUSED PtnArrayEntry *ptn_array_entry_for_key(PtnArray *array, PtnArrayKey key) {
    size_t index = ptn_array_find_key(array, key);
    return index < array->len ? &array->entries[index] : NULL;
}

static PTN_UNUSED const char *ptn_offset_container_type_name(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return "null";
        case PTN_BOOL:
            return "bool";
        case PTN_INT:
            return "int";
        case PTN_FLOAT:
            return "float";
        case PTN_STRING:
            return "string";
        case PTN_ARRAY:
            return "array";
    }
    return "unknown";
}

static PTN_UNUSED void ptn_emit_array_runtime_diagnostic(const char *kind, const char *message, size_t line) {
    fputc('\n', stdout);
    fputs(kind, stdout);
    fputs(": ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_foreach_non_array_warning(PtnValue value, size_t line) {
    char message[128];
    snprintf(
        message,
        sizeof(message),
        "foreach() argument must be of type array|object, %s given",
        ptn_offset_container_type_name(value)
    );
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    (void)runtime;
    PtnArrayIterator iterator;
    iterator.array = NULL;
    iterator.index = 0;
    iterator.valid = 0;
    if (value.type != PTN_ARRAY) {
        ptn_emit_foreach_non_array_warning(value, line);
        return iterator;
    }
    iterator.array = value.as.array;
    iterator.valid = iterator.array->len != 0;
    return iterator;
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_key(PtnArrayIterator *iterator) {
    PtnArrayKey key = iterator->array->entries[iterator->index].key;
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_string(key.as.string);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator) {
    return iterator->array->entries[iterator->index].value;
}

static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator) {
    iterator->index++;
    iterator->valid = iterator->array != NULL && iterator->index < iterator->array->len;
}

static PTN_UNUSED char *ptn_array_key_diagnostic_name(PtnArrayKey key) {
    char buffer[64];
    if (key.type == PTN_ARRAY_KEY_INT) {
        int written = snprintf(buffer, sizeof(buffer), "%lld", (long long)key.as.integer);
        if (written < 0 || (size_t)written >= sizeof(buffer)) {
            ptn_abort_out_of_memory();
        }
        return ptn_duplicate_string(buffer);
    }

    size_t key_len = strlen(key.as.string);
    if (key_len > SIZE_MAX - 3) {
        ptn_abort_out_of_memory();
    }
    char *display = malloc(key_len + 3);
    if (display == NULL) {
        ptn_abort_out_of_memory();
    }
    display[0] = '"';
    memcpy(display + 1, key.as.string, key_len);
    display[key_len + 1] = '"';
    display[key_len + 2] = '\0';
    return display;
}

static PTN_UNUSED void ptn_emit_undefined_array_key_warning(PtnArrayKey key, size_t line) {
    const char *prefix = "Undefined array key ";
    char *display = ptn_array_key_diagnostic_name(key);
    size_t prefix_len = strlen(prefix);
    size_t display_len = strlen(display);
    if (prefix_len > SIZE_MAX - display_len - 1) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc(prefix_len + display_len + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(message, prefix, prefix_len);
    memcpy(message + prefix_len, display, display_len + 1);
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
    free(message);
    free(display);
}

static PTN_UNUSED void ptn_emit_string_offset_cast_warning(size_t line) {
    ptn_emit_array_runtime_diagnostic("Warning", "String offset cast occurred", line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_warning(const char *key, size_t line) {
    const char *prefix = "Illegal string offset \"";
    size_t prefix_len = strlen(prefix);
    size_t key_len = strlen(key);
    if (key_len > SIZE_MAX - prefix_len - 2) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc(prefix_len + key_len + 2);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(message, prefix, prefix_len);
    memcpy(message + prefix_len, key, key_len);
    message[prefix_len + key_len] = '"';
    message[prefix_len + key_len + 1] = '\0';
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
    free(message);
}

static PTN_UNUSED void ptn_emit_uninitialized_string_offset_warning(int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Uninitialized string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED int ptn_string_to_offset(const char *string, int64_t *offset, int *warn_illegal) {
    const char *cursor = string;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }

    const char *number_start = cursor;
    if (*cursor == '-' || *cursor == '+') {
        cursor++;
    }
    if (!isdigit((unsigned char)*cursor)) {
        return 0;
    }
    while (isdigit((unsigned char)*cursor)) {
        cursor++;
    }

    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(number_start, &end, 10);
    if (errno == ERANGE || end == number_start) {
        return 0;
    }

    cursor = end;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }
    if (*cursor == '\0') {
        *offset = (int64_t)parsed;
        return 1;
    }
    if (*cursor == '.') {
        return 0;
    }

    *offset = (int64_t)parsed;
    *warn_illegal = 1;
    return 1;
}

static PTN_UNUSED int ptn_string_offset_from_value(PtnValue key_value, size_t line, int quiet, int64_t *offset) {
    switch (key_value.type) {
        case PTN_INT:
            *offset = key_value.as.integer;
            return 1;
        case PTN_BOOL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = key_value.as.boolean ? 1 : 0;
            return 1;
        case PTN_NULL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = 0;
            return 1;
        case PTN_FLOAT:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = (int64_t)key_value.as.floating;
            return 1;
        case PTN_STRING: {
            int warn_illegal = 0;
            if (ptn_string_to_offset(key_value.as.string, offset, &warn_illegal)) {
                if (warn_illegal) {
                    if (quiet) {
                        return 0;
                    }
                    ptn_emit_illegal_string_offset_warning(key_value.as.string, line);
                }
                return 1;
            }
            if (quiet) {
                return 0;
            }
            fputs("Fatal error: Cannot access offset of type string on string\n", stderr);
            exit(255);
        }
        case PTN_ARRAY:
            if (quiet) {
                return 0;
            }
            fputs("Fatal error: Cannot access offset of type array on string\n", stderr);
            exit(255);
    }
    return 0;
}

static PTN_UNUSED int ptn_string_offset_index(size_t string_len, int64_t offset, size_t *index) {
    if (offset >= 0) {
        uint64_t positive = (uint64_t)offset;
        if (positive >= string_len) {
            return 0;
        }
        *index = (size_t)positive;
        return 1;
    }

    uint64_t distance = (uint64_t)(-(offset + 1)) + 1;
    if (distance > string_len) {
        return 0;
    }
    *index = string_len - (size_t)distance;
    return 1;
}

static PTN_UNUSED PtnLookupResult ptn_string_offset_lookup(PtnValue container, PtnValue key_value, size_t line, int quiet) {
    int64_t offset = 0;
    if (!ptn_string_offset_from_value(key_value, line, quiet, &offset)) {
        return ptn_lookup_missing();
    }
    size_t index = 0;
    if (!ptn_string_offset_index(strlen(container.as.string), offset, &index)) {
        if (!quiet) {
            ptn_emit_uninitialized_string_offset_warning(offset, line);
            return ptn_lookup_found(ptn_string(""));
        }
        return ptn_lookup_missing();
    }

    char *result = malloc(2);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = container.as.string[index];
    result[1] = '\0';
    return ptn_lookup_found(ptn_owned_string(result));
}

static PTN_UNUSED PtnLookupResult ptn_offset_lookup(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line, int quiet) {
    (void)runtime;
    if (container.type == PTN_STRING) {
        return ptn_string_offset_lookup(container, key_value, line, quiet);
    }

    if (container.type != PTN_ARRAY) {
        if (!quiet) {
            const char *prefix = "Trying to access array offset on value of type ";
            const char *type_name = ptn_offset_container_type_name(container);
            char message[128];
            int written = snprintf(message, sizeof(message), "%s%s", prefix, type_name);
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_array_runtime_diagnostic("Warning", message, line);
        }
        return ptn_lookup_missing();
    }

    if (key_value.type == PTN_NULL) {
        ptn_emit_array_runtime_diagnostic(
            "Deprecated",
            "Using null as an array offset is deprecated, use an empty string instead",
            line
        );
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    if (entry == NULL) {
        if (!quiet) {
            ptn_emit_undefined_array_key_warning(key, line);
        }
        ptn_array_key_free(key);
        return ptn_lookup_missing();
    }
    PtnValue value = entry->value;
    ptn_array_key_free(key);
    return ptn_lookup_found(value);
}

static PTN_UNUSED PtnValue ptn_array_read(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    PtnLookupResult result = ptn_offset_lookup(runtime, container, key_value, line, 0);
    return result.exists ? result.value : ptn_null();
}

static PTN_UNUSED int ptn_compare_arrays_equal(PtnArray *left, PtnArray *right) {
    if (left->len != right->len) {
        return 0;
    }
    for (size_t i = 0; i < left->len; i++) {
        PtnArrayEntry *right_entry = ptn_array_entry_for_key(right, left->entries[i].key);
        if (right_entry == NULL || !ptn_compare_equal(left->entries[i].value, right_entry->value)) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_compare_arrays_identical(PtnArray *left, PtnArray *right) {
    if (left->len != right->len) {
        return 0;
    }
    for (size_t i = 0; i < left->len; i++) {
        if (!ptn_array_keys_equal(left->entries[i].key, right->entries[i].key) ||
            !ptn_compare_identical(left->entries[i].value, right->entries[i].value)) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_compare_arrays_order(PtnArray *left, PtnArray *right) {
    if (left->len < right->len) {
        return PTN_COMPARE_LESS;
    }
    if (left->len > right->len) {
        return PTN_COMPARE_GREATER;
    }
    for (size_t i = 0; i < left->len; i++) {
        PtnArrayEntry *right_entry = ptn_array_entry_for_key(right, left->entries[i].key);
        if (right_entry == NULL) {
            return PTN_COMPARE_UNORDERED;
        }
        int compared = ptn_compare_order(left->entries[i].value, right_entry->value);
        if (compared != PTN_COMPARE_EQUAL) {
            return compared;
        }
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_equal(PtnValue left, PtnValue right) {
    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_is_truthy(left) == ptn_is_truthy(right);
    }
    if (left.type == PTN_NULL || right.type == PTN_NULL) {
        if (left.type == PTN_NULL && right.type == PTN_NULL) {
            return 1;
        }
        PtnValue other = left.type == PTN_NULL ? right : left;
        switch (other.type) {
            case PTN_NULL:
                return 1;
            case PTN_BOOL:
                return ptn_is_truthy(other) == 0;
            case PTN_INT:
                return other.as.integer == 0;
            case PTN_FLOAT:
                return other.as.floating == 0.0;
            case PTN_STRING:
                return other.as.string[0] == '\0';
            case PTN_ARRAY:
                return other.as.array->len == 0;
        }
    }

    if (left.type == PTN_ARRAY || right.type == PTN_ARRAY) {
        if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
            return ptn_compare_arrays_equal(left.as.array, right.as.array);
        }
        return 0;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number) == PTN_COMPARE_EQUAL;
    }
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return strcmp(left.as.string, right.as.string) == 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_identical(PtnValue left, PtnValue right) {
    if (left.type != right.type) {
        return 0;
    }
    switch (left.type) {
        case PTN_NULL:
            return 1;
        case PTN_BOOL:
            return left.as.boolean == right.as.boolean;
        case PTN_INT:
            return left.as.integer == right.as.integer;
        case PTN_FLOAT:
            return left.as.floating == right.as.floating;
        case PTN_STRING:
            return strcmp(left.as.string, right.as.string) == 0;
        case PTN_ARRAY:
            return ptn_compare_arrays_identical(left.as.array, right.as.array);
    }
    return 0;
}

static PTN_UNUSED int ptn_value_is_nan(PtnValue value) {
    return value.type == PTN_FLOAT && isnan(value.as.floating);
}

static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right) {
    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
    }
    if (left.type == PTN_NULL && right.type == PTN_NULL) {
        return 0;
    }
    if (left.type == PTN_NULL) {
        if (ptn_value_is_nan(right)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (ptn_is_number_type(right)) {
            double right_number = right.type == PTN_INT ? (double)right.as.integer : right.as.floating;
            return ptn_compare_numbers(0.0, right_number);
        }
        if (right.type == PTN_STRING) {
            return ptn_compare_strings("", right.as.string);
        }
        if (right.type == PTN_ARRAY) {
            return ptn_compare_numbers(0.0, (double)ptn_is_truthy(right));
        }
    }
    if (right.type == PTN_NULL) {
        if (ptn_value_is_nan(left)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (ptn_is_number_type(left)) {
            double left_number = left.type == PTN_INT ? (double)left.as.integer : left.as.floating;
            return ptn_compare_numbers(left_number, 0.0);
        }
        if (left.type == PTN_STRING) {
            return ptn_compare_strings(left.as.string, "");
        }
        if (left.type == PTN_ARRAY) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), 0.0);
        }
    }

    if (left.type == PTN_ARRAY || right.type == PTN_ARRAY) {
        if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
            return ptn_compare_arrays_order(left.as.array, right.as.array);
        }
        return left.type == PTN_ARRAY ? PTN_COMPARE_GREATER : PTN_COMPARE_LESS;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number);
    }
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_compare_strings(left.as.string, right.as.string);
    }
    if (ptn_is_number_type(left) && right.type == PTN_STRING) {
        if (ptn_value_is_nan(left)) {
            return PTN_COMPARE_UNORDERED;
        }
        return ptn_compare_number_and_string(left, right.as.string, 1);
    }
    if (left.type == PTN_STRING && ptn_is_number_type(right)) {
        if (ptn_value_is_nan(right)) {
            return PTN_COMPARE_UNORDERED;
        }
        return ptn_compare_number_and_string(right, left.as.string, 0);
    }
    return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
}

static PTN_UNUSED int ptn_compare_less(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) == PTN_COMPARE_LESS;
}

static PTN_UNUSED int ptn_compare_less_equal(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    return compared == PTN_COMPARE_LESS || compared == PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_greater(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) == PTN_COMPARE_GREATER;
}

static PTN_UNUSED int ptn_compare_greater_equal(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    return compared == PTN_COMPARE_GREATER || compared == PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_spaceship(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_EQUAL) {
        return 0;
    }
    return 1;
}

static PTN_UNUSED PtnValue ptn_add(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating + right_number.floating);
    }

    if ((right_number.integer > 0 && left_number.integer > INT64_MAX - right_number.integer) ||
        (right_number.integer < 0 && left_number.integer < INT64_MIN - right_number.integer)) {
        return ptn_float((double)left_number.integer + (double)right_number.integer);
    }
    return ptn_int(left_number.integer + right_number.integer);
}

static PTN_UNUSED PtnValue ptn_subtract(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating - right_number.floating);
    }

    if ((right_number.integer < 0 && left_number.integer > INT64_MAX + right_number.integer) ||
        (right_number.integer > 0 && left_number.integer < INT64_MIN + right_number.integer)) {
        return ptn_float((double)left_number.integer - (double)right_number.integer);
    }
    return ptn_int(left_number.integer - right_number.integer);
}

static PTN_UNUSED int ptn_multiply_overflows(int64_t left, int64_t right) {
    if (left == 0 || right == 0) {
        return 0;
    }
    if (left > 0) {
        if (right > 0) {
            return left > INT64_MAX / right;
        }
        return right < INT64_MIN / left;
    }
    if (right > 0) {
        return left < INT64_MIN / right;
    }
    return right < INT64_MAX / left;
}

static PTN_UNUSED PtnValue ptn_multiply(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating * right_number.floating);
    }

    if (ptn_multiply_overflows(left_number.integer, right_number.integer)) {
        return ptn_float((double)left_number.integer * (double)right_number.integer);
    }
    return ptn_int(left_number.integer * right_number.integer);
}

static PTN_UNUSED int ptn_integer_power_fits(int64_t base, int64_t exponent, int64_t *out) {
    if (exponent < 0) {
        return 0;
    }

    int64_t result = 1;
    int64_t factor = base;
    int64_t remaining = exponent;
    while (remaining > 0) {
        if ((remaining & 1) != 0) {
            if (ptn_multiply_overflows(result, factor)) {
                return 0;
            }
            result *= factor;
        }
        remaining >>= 1;
        if (remaining > 0) {
            if (ptn_multiply_overflows(factor, factor)) {
                return 0;
            }
            factor *= factor;
        }
    }

    *out = result;
    return 1;
}

static PTN_UNUSED PtnValue ptn_power(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        int64_t integer_result = 0;
        if (ptn_integer_power_fits(left_number.integer, right_number.integer, &integer_result)) {
            return ptn_int(integer_result);
        }
    }
    return ptn_float(pow(left_number.floating, right_number.floating));
}

static PTN_UNUSED PtnValue ptn_divide(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (right_number.floating == 0.0) {
        ptn_abort_arithmetic_error("Division by zero");
    }

    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        if (left_number.integer == INT64_MIN && right_number.integer == -1) {
            return ptn_float((double)left_number.integer / (double)right_number.integer);
        }
        if (left_number.integer % right_number.integer == 0) {
            return ptn_int(left_number.integer / right_number.integer);
        }
    }
    return ptn_float(left_number.floating / right_number.floating);
}

static PTN_UNUSED int ptn_float_to_int_loses_precision(double value) {
    if (value < -9223372036854775808.0 || value >= 9223372036854775808.0) {
        return 1;
    }
    int64_t integer = (int64_t)value;
    return (double)integer != value;
}

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation(double value) {
    printf(
        "\nDeprecated: Implicit conversion from float %.14g to int loses precision in ptn-generated-code on line 0\n",
        value
    );
}

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation(const char *value) {
    printf(
        "\nDeprecated: Implicit conversion from float-string \"%s\" to int loses precision in ptn-generated-code on line 0\n",
        value
    );
}

static PTN_UNUSED int ptn_string_has_trailing_non_numeric_data(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    (void)strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    return *end != '\0';
}

static PTN_UNUSED void ptn_emit_non_numeric_value_warning(void) {
    printf("\nWarning: A non-numeric value encountered in ptn-generated-code on line 0\n");
}

static PTN_UNUSED int64_t ptn_number_to_integer(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return (int64_t)number.floating;
    }
    return number.integer;
}

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (value.type == PTN_STRING && ptn_string_has_trailing_non_numeric_data(value.as.string)) {
        ptn_emit_non_numeric_value_warning();
    }
    if (number.type == PTN_NUMBER_FLOAT && ptn_float_to_int_loses_precision(number.floating)) {
        if (value.type == PTN_STRING) {
            ptn_emit_float_string_to_int_precision_deprecation(value.as.string);
        } else {
            ptn_emit_float_to_int_precision_deprecation(number.floating);
        }
    }
    return ptn_number_to_integer(number);
}

static PTN_UNUSED PtnValue ptn_modulo(PtnValue left, PtnValue right) {
    int64_t left_integer = ptn_value_to_integer_with_precision_deprecation(left);
    int64_t right_integer = ptn_value_to_integer_with_precision_deprecation(right);
    if (right_integer == 0) {
        ptn_abort_arithmetic_error("Modulo by zero");
    }
    if (left_integer == INT64_MIN && right_integer == -1) {
        return ptn_int(0);
    }
    return ptn_int(left_integer % right_integer);
}

static PTN_UNUSED PtnValue ptn_increment(PtnValue value) {
    return ptn_add(value, ptn_int(1));
}

static PTN_UNUSED PtnValue ptn_decrement(PtnValue value) {
    return ptn_subtract(value, ptn_int(1));
}

static PTN_UNUSED PtnValue ptn_bitwise_string_and(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left[i] & (unsigned char)right[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_or(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len > right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        unsigned char left_byte = i < left_len ? (unsigned char)left[i] : 0;
        unsigned char right_byte = i < right_len ? (unsigned char)right[i] : 0;
        result[i] = (char)(left_byte | right_byte);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_xor(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left[i] ^ (unsigned char)right[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_not(const char *value) {
    size_t len = strlen(value);
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        result[i] = (char)(~(unsigned char)value[i]);
    }
    result[len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED int64_t ptn_value_to_integer(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(value);
}

static PTN_UNUSED int64_t ptn_bitwise_integer_operand(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(value);
}

static PTN_UNUSED PtnValue ptn_bitwise_and(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_and(left.as.string, right.as.string);
    }
    return ptn_int(ptn_bitwise_integer_operand(left) & ptn_bitwise_integer_operand(right));
}

static PTN_UNUSED PtnValue ptn_bitwise_or(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_or(left.as.string, right.as.string);
    }
    return ptn_int(ptn_bitwise_integer_operand(left) | ptn_bitwise_integer_operand(right));
}

static PTN_UNUSED PtnValue ptn_bitwise_xor(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_xor(left.as.string, right.as.string);
    }
    return ptn_int(ptn_bitwise_integer_operand(left) ^ ptn_bitwise_integer_operand(right));
}

static PTN_UNUSED PtnValue ptn_bitwise_not(PtnValue value) {
    if (value.type == PTN_STRING) {
        return ptn_bitwise_string_not(value.as.string);
    }
    return ptn_int(~ptn_bitwise_integer_operand(value));
}

static PTN_UNUSED int64_t ptn_shift_distance(PtnValue value) {
    int64_t distance = ptn_bitwise_integer_operand(value);
    if (distance < 0) {
        ptn_abort_arithmetic_error("Bit shift by negative number");
    }
    return distance;
}

static PTN_UNUSED PtnValue ptn_shift_left(PtnValue left, PtnValue right) {
    uint64_t left_bits = (uint64_t)ptn_bitwise_integer_operand(left);
    int64_t distance = ptn_shift_distance(right);
    if (distance >= 64) {
        return ptn_int(0);
    }
    return ptn_int((int64_t)(left_bits << (unsigned int)distance));
}

static PTN_UNUSED PtnValue ptn_shift_right(PtnValue left, PtnValue right) {
    int64_t left_integer = ptn_bitwise_integer_operand(left);
    int64_t distance = ptn_shift_distance(right);
    if (distance >= 64) {
        return ptn_int(left_integer < 0 ? -1 : 0);
    }
    return ptn_int(left_integer >> (unsigned int)distance);
}

static PTN_UNUSED char *ptn_value_to_string(PtnValue value) {
    char buffer[128];
    int written = 0;

    switch (value.type) {
        case PTN_NULL:
            return ptn_duplicate_string("");
        case PTN_BOOL:
            return ptn_duplicate_string(value.as.boolean ? "1" : "");
        case PTN_INT:
            written = snprintf(buffer, sizeof(buffer), "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            written = snprintf(buffer, sizeof(buffer), "%.14g", value.as.floating);
            break;
        case PTN_STRING:
            return ptn_duplicate_string(value.as.string);
        case PTN_ARRAY:
            return ptn_duplicate_string("Array");
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_duplicate_string(buffer);
}

static PTN_UNUSED PtnStringOperand ptn_string_operand_borrowed(const char *data) {
    PtnStringOperand operand;
    operand.data = data;
    operand.owned = NULL;
    operand.len = strlen(data);
    return operand;
}

static PTN_UNUSED PtnStringOperand ptn_string_operand_owned(char *data) {
    PtnStringOperand operand;
    operand.data = data;
    operand.owned = data;
    operand.len = strlen(data);
    return operand;
}

static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand) {
    free(operand.owned);
}

static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand(PtnValue value) {
    char buffer[128];
    int written = 0;

    switch (value.type) {
        case PTN_NULL:
            return ptn_string_operand_borrowed("");
        case PTN_BOOL:
            return ptn_string_operand_borrowed(value.as.boolean ? "1" : "");
        case PTN_INT:
            written = snprintf(buffer, sizeof(buffer), "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            written = snprintf(buffer, sizeof(buffer), "%.14g", value.as.floating);
            break;
        case PTN_STRING:
            return ptn_string_operand_borrowed(value.as.string);
        case PTN_ARRAY:
            return ptn_string_operand_borrowed("Array");
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_string_operand_owned(ptn_duplicate_string(buffer));
}

static PTN_UNUSED PtnValue ptn_concat(PtnValue left, PtnValue right) {
    PtnStringOperand left_string = ptn_value_to_string_operand(left);
    PtnStringOperand right_string = ptn_value_to_string_operand(right);
    if (left_string.len > SIZE_MAX - right_string.len) {
        ptn_abort_out_of_memory();
    }
    size_t joined_len = left_string.len + right_string.len;
    if (joined_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    char *joined = malloc(joined_len + 1);
    if (joined == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(joined, left_string.data, left_string.len);
    memcpy(joined + left_string.len, right_string.data, right_string.len);
    joined[joined_len] = '\0';
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return ptn_owned_string(joined);
}

static PTN_UNUSED PtnValue ptn_cast_string(PtnValue value) {
    return ptn_owned_string(ptn_value_to_string(value));
}

static PTN_UNUSED PtnValue ptn_cast_bool(PtnValue value) {
    return ptn_bool(ptn_is_truthy(value));
}

typedef enum {
    PTN_CAST_TARGET_INT,
    PTN_CAST_TARGET_FLOAT,
    PTN_CAST_TARGET_STRING,
    PTN_CAST_TARGET_BOOL
} PtnCastTarget;

static PTN_UNUSED PtnValue ptn_cast_target(PtnValue value, PtnCastTarget target) {
    switch (target) {
        case PTN_CAST_TARGET_INT:
            return ptn_cast_int(value);
        case PTN_CAST_TARGET_FLOAT:
            return ptn_cast_float(value);
        case PTN_CAST_TARGET_STRING:
            return ptn_cast_string(value);
        case PTN_CAST_TARGET_BOOL:
            return ptn_cast_bool(value);
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_cast_noncanonical(
    PtnRuntime *runtime,
    PtnValue value,
    const char *spelling,
    const char *canonical,
    PtnCastTarget target,
    size_t line
) {
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "Non-canonical cast (%s) is deprecated, use the (%s) cast instead",
        spelling,
        canonical
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_deprecation(&runtime->diagnostics, message, line);
    return ptn_cast_target(value, target);
}

static PTN_UNUSED PtnValue ptn_gettype_value(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_string("NULL");
        case PTN_BOOL:
            return ptn_string("boolean");
        case PTN_INT:
            return ptn_string("integer");
        case PTN_FLOAT:
            return ptn_string("double");
        case PTN_STRING:
            return ptn_string("string");
        case PTN_ARRAY:
            return ptn_string("array");
    }
    return ptn_string("unknown type");
}

static PTN_UNUSED PtnValue ptn_is_type(PtnValue value, PtnType type) {
    return ptn_bool(value.type == type);
}

static PTN_UNUSED PtnValue ptn_is_scalar(PtnValue value) {
    return ptn_bool(
        value.type == PTN_BOOL ||
        value.type == PTN_INT ||
        value.type == PTN_FLOAT ||
        value.type == PTN_STRING
    );
}

static PTN_UNUSED int ptn_ascii_case_compare(const char *left, const char *right) {
    while (*left != '\0' && *right != '\0') {
        int left_byte = tolower((unsigned char)*left);
        int right_byte = tolower((unsigned char)*right);
        if (left_byte != right_byte) {
            return left_byte < right_byte ? -1 : 1;
        }
        left++;
        right++;
    }
    if (*left == '\0' && *right == '\0') {
        return 0;
    }
    return *left == '\0' ? -1 : 1;
}

static PTN_UNUSED int ptn_ascii_case_equal(const char *left, const char *right) {
    return ptn_ascii_case_compare(left, right) == 0;
}

static PTN_UNUSED int ptn_builtin_constant_value(const char *name, PtnValue *out) {
    if (strcmp(name, "E_ERROR") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "M_E") == 0) {
        *out = ptn_float(2.718281828459045);
        return 1;
    }
    if (strcmp(name, "M_LOG2E") == 0) {
        *out = ptn_float(1.4426950408889634);
        return 1;
    }
    if (strcmp(name, "M_LOG10E") == 0) {
        *out = ptn_float(0.4342944819032518);
        return 1;
    }
    if (strcmp(name, "M_LN2") == 0) {
        *out = ptn_float(0.6931471805599453);
        return 1;
    }
    if (strcmp(name, "M_LN10") == 0) {
        *out = ptn_float(2.302585092994046);
        return 1;
    }
    if (strcmp(name, "PHP_INT_MIN") == 0) {
        *out = ptn_int(INT64_MIN);
        return 1;
    }
    if (strcmp(name, "PHP_INT_MAX") == 0) {
        *out = ptn_int(INT64_MAX);
        return 1;
    }
    if (strcmp(name, "PHP_INT_SIZE") == 0) {
        *out = ptn_int((int64_t)sizeof(int64_t));
        return 1;
    }
    if (strcmp(name, "PHP_EOL") == 0) {
        *out = ptn_string("\n");
        return 1;
    }
    if (strcmp(name, "DIRECTORY_SEPARATOR") == 0) {
#if defined(_WIN32)
        *out = ptn_string("\\");
#else
        *out = ptn_string("/");
#endif
        return 1;
    }
    if (strcmp(name, "PATH_SEPARATOR") == 0) {
#if defined(_WIN32)
        *out = ptn_string(";");
#else
        *out = ptn_string(":");
#endif
        return 1;
    }
    if (strcmp(name, "INF") == 0) {
        *out = ptn_float(INFINITY);
        return 1;
    }
    if (strcmp(name, "NAN") == 0) {
        *out = ptn_float(NAN);
        return 1;
    }
    if (strcmp(name, "M_PI") == 0) {
        *out = ptn_float(3.14159265358979323846264338327950288);
        return 1;
    }
    if (strcmp(name, "M_PI_2") == 0) {
        *out = ptn_float(1.5707963267948966);
        return 1;
    }
    if (strcmp(name, "M_PI_4") == 0) {
        *out = ptn_float(0.7853981633974483);
        return 1;
    }
    if (strcmp(name, "M_1_PI") == 0) {
        *out = ptn_float(0.3183098861837907);
        return 1;
    }
    if (strcmp(name, "M_2_PI") == 0) {
        *out = ptn_float(0.6366197723675814);
        return 1;
    }
    if (strcmp(name, "M_SQRTPI") == 0) {
        *out = ptn_float(1.772453850905516);
        return 1;
    }
    if (strcmp(name, "M_2_SQRTPI") == 0) {
        *out = ptn_float(1.1283791670955126);
        return 1;
    }
    if (strcmp(name, "M_LNPI") == 0) {
        *out = ptn_float(1.1447298858494002);
        return 1;
    }
    if (strcmp(name, "M_EULER") == 0) {
        *out = ptn_float(0.5772156649015329);
        return 1;
    }
    if (strcmp(name, "M_SQRT2") == 0) {
        *out = ptn_float(1.4142135623730951);
        return 1;
    }
    if (strcmp(name, "M_SQRT1_2") == 0) {
        *out = ptn_float(0.7071067811865476);
        return 1;
    }
    if (strcmp(name, "M_SQRT3") == 0) {
        *out = ptn_float(1.7320508075688772);
        return 1;
    }
    return 0;
}

static int ptn_same_double(double left, double right) {
    return memcmp(&left, &right, sizeof(double)) == 0;
}

static void ptn_normalize_var_dump_exponent(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == 'e' || *cursor == 'E') {
            *cursor = 'E';
            cursor++;
            if (*cursor == '+' || *cursor == '-') {
                cursor++;
            }
            while (*cursor == '0' && isdigit((unsigned char)cursor[1])) {
                memmove(cursor, cursor + 1, strlen(cursor));
            }
            return;
        }
    }
}

static void ptn_format_var_dump_float(double value, char *buffer, size_t buffer_size) {
    if (isnan(value)) {
        snprintf(buffer, buffer_size, "NAN");
        return;
    }
    if (isinf(value)) {
        snprintf(buffer, buffer_size, signbit(value) ? "-INF" : "INF");
        return;
    }

    for (int precision = 1; precision <= 17; precision++) {
        char candidate[64];
        char *end = NULL;
        double reparsed;
        snprintf(candidate, sizeof(candidate), "%.*g", precision, value);
        ptn_normalize_var_dump_exponent(candidate);
        errno = 0;
        reparsed = strtod(candidate, &end);
        if (errno == 0 && end != NULL && *end == '\0' && ptn_same_double(reparsed, value)) {
            snprintf(buffer, buffer_size, "%s", candidate);
            return;
        }
    }

    snprintf(buffer, buffer_size, "%.17g", value);
    ptn_normalize_var_dump_exponent(buffer);
}

static PTN_UNUSED int ptn_runtime_constant_value(PtnRuntime *runtime, const char *name, PtnValue *out) {
    if (ptn_symbols_get(&runtime->constants, name, out)) {
        return 1;
    }
    return ptn_builtin_constant_value(name, out);
}

static PTN_UNUSED int ptn_runtime_constant_is_defined(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return ptn_runtime_constant_value(runtime, name, &value);
}

static PTN_UNUSED int ptn_runtime_define_constant_if_absent(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t line
) {
    if (ptn_runtime_constant_is_defined(runtime, name)) {
        ptn_emit_constant_already_defined_warning(&runtime->diagnostics, name, line);
        return 0;
    }
    ptn_runtime_define_constant(runtime, name, value);
    return 1;
}

static PTN_UNUSED PtnValue ptn_read_constant(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_runtime_constant_value(runtime, name, &value)) {
        return value;
    }
    ptn_emit_undefined_constant_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}

static PTN_UNUSED void ptn_echo(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            break;
        case PTN_BOOL:
            if (value.as.boolean) {
                fputs("1", stdout);
            }
            break;
        case PTN_INT:
            printf("%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            printf("%.14g", value.as.floating);
            break;
        case PTN_STRING:
            fputs(value.as.string, stdout);
            break;
        case PTN_ARRAY:
            fputs("Array", stdout);
            break;
    }
}

static void ptn_var_dump_indent(size_t indent) {
    for (size_t i = 0; i < indent; i++) {
        fputs("  ", stdout);
    }
}

static void ptn_var_dump_value_indented(PtnValue value, size_t indent) {
    switch (value.type) {
        case PTN_NULL:
            ptn_var_dump_indent(indent);
            fputs("NULL\n", stdout);
            break;
        case PTN_BOOL:
            ptn_var_dump_indent(indent);
            fputs(value.as.boolean ? "bool(true)\n" : "bool(false)\n", stdout);
            break;
        case PTN_INT:
            ptn_var_dump_indent(indent);
            printf("int(%lld)\n", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[64];
            ptn_format_var_dump_float(value.as.floating, formatted, sizeof(formatted));
            ptn_var_dump_indent(indent);
            printf("float(%s)\n", formatted);
            break;
        }
        case PTN_STRING:
            ptn_var_dump_indent(indent);
            printf("string(%zu) \"", strlen(value.as.string));
            fputs(value.as.string, stdout);
            fputs("\"\n", stdout);
            break;
        case PTN_ARRAY: {
            PtnArray *array = value.as.array;
            ptn_var_dump_indent(indent);
            printf("array(%zu) {\n", array->len);
            for (size_t i = 0; i < array->len; i++) {
                ptn_var_dump_indent(indent + 1);
                PtnArrayKey key = array->entries[i].key;
                if (key.type == PTN_ARRAY_KEY_INT) {
                    printf("[%lld]=>\n", (long long)key.as.integer);
                } else {
                    printf("[\"%s\"]=>\n", key.as.string);
                }
                ptn_var_dump_value_indented(array->entries[i].value, indent + 1);
            }
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        }
    }
}

static void ptn_var_dump_value(PtnValue value) {
    ptn_var_dump_value_indented(value, 0);
}

static PtnValue ptn_internal_var_dump(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    for (size_t i = 0; i < argc; i++) {
        ptn_var_dump_value(args[i]);
    }
    return ptn_null();
}

static PtnValue ptn_internal_strlen(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *string = ptn_value_to_string(args[0]);
    size_t len = strlen(string);
    free(string);
    return ptn_int((int64_t)len);
}

static char *ptn_rot13_string(const char *string) {
    size_t len = strlen(string);
    char *rotated = malloc(len + 1);
    if (rotated == NULL) {
        ptn_abort_out_of_memory();
    }

    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string[i];
        if (byte >= 'a' && byte <= 'z') {
            rotated[i] = (char)('a' + ((byte - 'a' + 13) % 26));
        } else if (byte >= 'A' && byte <= 'Z') {
            rotated[i] = (char)('A' + ((byte - 'A' + 13) % 26));
        } else {
            rotated[i] = (char)byte;
        }
    }
    rotated[len] = '\0';
    return rotated;
}

static PtnValue ptn_internal_str_rot13(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *string = ptn_value_to_string(args[0]);
    char *rotated = ptn_rot13_string(string);
    free(string);
    return ptn_owned_string(rotated);
}

static PtnValue ptn_internal_strcmp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *left = ptn_value_to_string(args[0]);
    char *right = ptn_value_to_string(args[1]);
    int compared = strcmp(left, right);
    free(left);
    free(right);
    if (compared < 0) {
        return ptn_int(-1);
    }
    if (compared > 0) {
        return ptn_int(1);
    }
    return ptn_int(0);
}

static PtnValue ptn_internal_str_contains(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *haystack = ptn_value_to_string(args[0]);
    char *needle = ptn_value_to_string(args[1]);
    int contains = needle[0] == '\0' || strstr(haystack, needle) != NULL;
    free(haystack);
    free(needle);
    return ptn_bool(contains);
}

static PtnValue ptn_internal_str_starts_with(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *haystack = ptn_value_to_string(args[0]);
    char *needle = ptn_value_to_string(args[1]);
    size_t haystack_len = strlen(haystack);
    size_t needle_len = strlen(needle);
    int starts = needle_len <= haystack_len && memcmp(haystack, needle, needle_len) == 0;
    free(haystack);
    free(needle);
    return ptn_bool(starts);
}

static PtnValue ptn_internal_str_ends_with(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *haystack = ptn_value_to_string(args[0]);
    char *needle = ptn_value_to_string(args[1]);
    size_t haystack_len = strlen(haystack);
    size_t needle_len = strlen(needle);
    int ends =
        needle_len <= haystack_len &&
        memcmp(haystack + haystack_len - needle_len, needle, needle_len) == 0;
    free(haystack);
    free(needle);
    return ptn_bool(ends);
}

static int ptn_quotemeta_needs_escape(unsigned char byte) {
    switch (byte) {
        case '.':
        case '\\':
        case '+':
        case '*':
        case '?':
        case '[':
        case '^':
        case ']':
        case '(':
        case '$':
        case ')':
            return 1;
        default:
            return 0;
    }
}

static char *ptn_quotemeta_string(const char *input) {
    size_t len = strlen(input);
    size_t escape_count = 0;
    for (size_t i = 0; i < len; i++) {
        if (ptn_quotemeta_needs_escape((unsigned char)input[i])) {
            escape_count++;
        }
    }
    if (escape_count > SIZE_MAX - len - 1) {
        ptn_abort_out_of_memory();
    }

    char *output = malloc(len + escape_count + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)input[i];
        if (ptn_quotemeta_needs_escape(byte)) {
            output[out++] = '\\';
        }
        output[out++] = (char)byte;
    }
    output[out] = '\0';
    return output;
}

static PtnValue ptn_internal_quotemeta(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *input = ptn_value_to_string(args[0]);
    char *output = ptn_quotemeta_string(input);
    free(input);
    return ptn_owned_string(output);
}

static char *ptn_chunk_split_string(const char *input, size_t chunk_len, const char *ending) {
    size_t input_len = strlen(input);
    size_t ending_len = strlen(ending);
    size_t chunk_count = input_len == 0 ? 0 : ((input_len - 1) / chunk_len) + 1;
    if (chunk_count != 0 && ending_len > (SIZE_MAX - input_len) / chunk_count) {
        ptn_abort_out_of_memory();
    }
    size_t output_len = input_len + (chunk_count * ending_len);
    if (output_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }

    char *output = malloc(output_len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < input_len) {
        size_t remaining = input_len - input_offset;
        size_t copy_len = remaining < chunk_len ? remaining : chunk_len;
        memcpy(output + output_offset, input + input_offset, copy_len);
        input_offset += copy_len;
        output_offset += copy_len;
        memcpy(output + output_offset, ending, ending_len);
        output_offset += ending_len;
    }
    output[output_offset] = '\0';
    return output;
}

static PtnValue ptn_internal_chunk_split(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    char *input = ptn_value_to_string(args[0]);
    int64_t chunk_len_value = argc >= 2 ? ptn_value_to_integer(args[1]) : 76;
    if (chunk_len_value <= 0) {
        free(input);
        ptn_abort_arithmetic_error("chunk_split(): Argument #2 ($length) must be greater than 0");
    }
    char *ending = argc >= 3 ? ptn_value_to_string(args[2]) : ptn_duplicate_string("\r\n");
    char *output = ptn_chunk_split_string(input, (size_t)chunk_len_value, ending);
    free(input);
    free(ending);
    return ptn_owned_string(output);
}

static char *ptn_strip_tags_string(const char *input) {
    size_t len = strlen(input);
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < len) {
        if (input[input_offset] == '<') {
            if (input_offset + 1 < len && input[input_offset + 1] == '?') {
                size_t tag_end = input_offset + 2;
                while (tag_end + 1 < len && !(input[tag_end] == '?' && input[tag_end + 1] == '>')) {
                    tag_end++;
                }
                if (tag_end + 1 < len) {
                    input_offset = tag_end + 2;
                    continue;
                }
            } else if (input_offset + 1 < len && input[input_offset + 1] == '%') {
                size_t tag_end = input_offset + 2;
                while (tag_end + 1 < len && !(input[tag_end] == '%' && input[tag_end + 1] == '>')) {
                    tag_end++;
                }
                if (tag_end + 1 < len) {
                    input_offset = tag_end + 2;
                    continue;
                }
            } else if (input_offset + 3 < len && strncmp(input + input_offset, "<!--", 4) == 0) {
                size_t tag_end = input_offset + 4;
                while (tag_end + 2 < len && strncmp(input + tag_end, "-->", 3) != 0) {
                    tag_end++;
                }
                if (tag_end + 2 < len) {
                    input_offset = tag_end + 3;
                    continue;
                }
            } else {
                size_t tag_end = input_offset + 1;
                while (tag_end < len && input[tag_end] != '>') {
                    tag_end++;
                }
                if (tag_end < len) {
                    input_offset = tag_end + 1;
                    continue;
                }
            }
        }
        output[output_offset++] = input[input_offset++];
    }
    output[output_offset] = '\0';
    return output;
}

static PtnValue ptn_internal_strip_tags(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *input = ptn_value_to_string(args[0]);
    char *output = ptn_strip_tags_string(input);
    free(input);
    return ptn_owned_string(output);
}

static uint32_t ptn_rotate_left32(uint32_t value, uint32_t amount) {
    return (value << amount) | (value >> (32 - amount));
}

static char *ptn_digest_hex_string(const unsigned char *digest, size_t digest_len) {
    static const char hex_digits[] = "0123456789abcdef";
    if (digest_len > (SIZE_MAX - 1) / 2) {
        ptn_abort_out_of_memory();
    }
    char *hex = malloc((digest_len * 2) + 1);
    if (hex == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < digest_len; i++) {
        hex[i * 2] = hex_digits[digest[i] >> 4];
        hex[i * 2 + 1] = hex_digits[digest[i] & 0x0f];
    }
    hex[digest_len * 2] = '\0';
    return hex;
}

static char *ptn_digest_raw_string(const unsigned char *digest, size_t digest_len) {
    char *raw = malloc(digest_len + 1);
    if (raw == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(raw, digest, digest_len);
    raw[digest_len] = '\0';
    return raw;
}

static PtnValue ptn_digest_value(const unsigned char *digest, size_t digest_len, int raw_output) {
    return ptn_owned_string(raw_output
        ? ptn_digest_raw_string(digest, digest_len)
        : ptn_digest_hex_string(digest, digest_len));
}

static void ptn_md5_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[16]) {
    static const uint32_t shifts[64] = {
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
    };
    static const uint32_t constants[64] = {
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391
    };

    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xefcdab89;
    uint32_t h2 = 0x98badcfe;
    uint32_t h3 = 0x10325476;

    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t words[16];
        for (size_t i = 0; i < 16; i++) {
            size_t base = offset + i * 4;
            words[i] = (uint32_t)message[base]
                | ((uint32_t)message[base + 1] << 8)
                | ((uint32_t)message[base + 2] << 16)
                | ((uint32_t)message[base + 3] << 24);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;

        for (uint32_t i = 0; i < 64; i++) {
            uint32_t f;
            uint32_t g;
            if (i < 16) {
                f = (b & c) | ((~b) & d);
                g = i;
            } else if (i < 32) {
                f = (d & b) | ((~d) & c);
                g = (5 * i + 1) % 16;
            } else if (i < 48) {
                f = b ^ c ^ d;
                g = (3 * i + 5) % 16;
            } else {
                f = c ^ (b | (~d));
                g = (7 * i) % 16;
            }

            uint32_t next = d;
            d = c;
            c = b;
            b = b + ptn_rotate_left32(a + f + constants[i] + words[g], shifts[i]);
            a = next;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
    }

    free(message);

    uint32_t words[4] = { h0, h1, h2, h3 };
    for (size_t i = 0; i < 4; i++) {
        digest[i * 4] = (unsigned char)words[i];
        digest[i * 4 + 1] = (unsigned char)(words[i] >> 8);
        digest[i * 4 + 2] = (unsigned char)(words[i] >> 16);
        digest[i * 4 + 3] = (unsigned char)(words[i] >> 24);
    }
}

static PtnValue ptn_internal_md5(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    char *input = ptn_value_to_string(args[0]);
    unsigned char digest[16];
    ptn_md5_digest_bytes((const unsigned char *)input, strlen(input), digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    free(input);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static void ptn_sha1_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[20]) {
    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + 7 - i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xefcdab89;
    uint32_t h2 = 0x98badcfe;
    uint32_t h3 = 0x10325476;
    uint32_t h4 = 0xc3d2e1f0;

    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t words[80];
        for (size_t i = 0; i < 16; i++) {
            size_t base = offset + i * 4;
            words[i] = ((uint32_t)message[base] << 24)
                | ((uint32_t)message[base + 1] << 16)
                | ((uint32_t)message[base + 2] << 8)
                | (uint32_t)message[base + 3];
        }
        for (size_t i = 16; i < 80; i++) {
            words[i] = ptn_rotate_left32(words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16], 1);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;
        uint32_t e = h4;

        for (size_t i = 0; i < 80; i++) {
            uint32_t f;
            uint32_t k;
            if (i < 20) {
                f = (b & c) | ((~b) & d);
                k = 0x5a827999;
            } else if (i < 40) {
                f = b ^ c ^ d;
                k = 0x6ed9eba1;
            } else if (i < 60) {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8f1bbcdc;
            } else {
                f = b ^ c ^ d;
                k = 0xca62c1d6;
            }

            uint32_t temp = ptn_rotate_left32(a, 5) + f + e + k + words[i];
            e = d;
            d = c;
            c = ptn_rotate_left32(b, 30);
            b = a;
            a = temp;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
        h4 += e;
    }

    free(message);

    uint32_t words[5] = { h0, h1, h2, h3, h4 };
    for (size_t i = 0; i < 5; i++) {
        digest[i * 4] = (unsigned char)(words[i] >> 24);
        digest[i * 4 + 1] = (unsigned char)(words[i] >> 16);
        digest[i * 4 + 2] = (unsigned char)(words[i] >> 8);
        digest[i * 4 + 3] = (unsigned char)words[i];
    }
}

static PtnValue ptn_internal_sha1(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    char *input = ptn_value_to_string(args[0]);
    unsigned char digest[20];
    ptn_sha1_digest_bytes((const unsigned char *)input, strlen(input), digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    free(input);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static size_t ptn_substr_clamped_positive(int64_t value, size_t limit) {
    if (value <= 0) {
        return 0;
    }
    uint64_t unsigned_value = (uint64_t)value;
    if (unsigned_value > (uint64_t)limit) {
        return limit;
    }
    return (size_t)unsigned_value;
}

static size_t ptn_substr_clamped_negative_distance(int64_t value, size_t limit) {
    if (value >= 0) {
        return 0;
    }
    if (value == INT64_MIN) {
        return limit;
    }
    uint64_t distance = (uint64_t)(-value);
    if (distance > (uint64_t)limit) {
        return limit;
    }
    return (size_t)distance;
}

static size_t ptn_substr_start_offset(size_t string_len, int64_t start) {
    if (start >= 0) {
        return ptn_substr_clamped_positive(start, string_len);
    }
    size_t distance = ptn_substr_clamped_negative_distance(start, string_len);
    return string_len - distance;
}

static char *ptn_substr_copy(const char *string, size_t start, size_t len) {
    if (len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(result, string + start, len);
    result[len] = '\0';
    return result;
}

static PtnValue ptn_internal_substr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    char *string = ptn_value_to_string(args[0]);
    size_t string_len = strlen(string);
    size_t start = ptn_substr_start_offset(string_len, ptn_value_to_integer(args[1]));
    size_t end = string_len;

    if (argc >= 3 && args[2].type != PTN_NULL) {
        int64_t length = ptn_value_to_integer(args[2]);
        if (length >= 0) {
            size_t requested_len = ptn_substr_clamped_positive(length, string_len);
            size_t available_len = string_len - start;
            if (requested_len > available_len) {
                requested_len = available_len;
            }
            end = start + requested_len;
        } else {
            size_t truncate_len = ptn_substr_clamped_negative_distance(length, string_len);
            end = string_len - truncate_len;
            if (end < start) {
                end = start;
            }
        }
    }

    char *substring = ptn_substr_copy(string, start, end - start);
    free(string);
    return ptn_owned_string(substring);
}

static int ptn_is_path_separator(char byte) {
    return byte == '/' || byte == '\\';
}

static char *ptn_dirname_string(const char *path) {
    size_t len = strlen(path);
    if (len == 0) {
        return ptn_duplicate_string(".");
    }
    while (len > 1 && ptn_is_path_separator(path[len - 1])) {
        len--;
    }

    size_t end = len;
    while (end > 0 && !ptn_is_path_separator(path[end - 1])) {
        end--;
    }
    if (end == 0) {
        return ptn_duplicate_string(".");
    }
    while (end > 1 && ptn_is_path_separator(path[end - 1])) {
        end--;
    }

    char *dirname = malloc(end + 1);
    if (dirname == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(dirname, path, end);
    dirname[end] = '\0';
    return dirname;
}

static PtnValue ptn_internal_dirname(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *path = ptn_value_to_string(args[0]);
    char *dirname = ptn_dirname_string(path);
    free(path);
    return ptn_owned_string(dirname);
}

static PtnValue ptn_internal_gettype(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_gettype_value(args[0]);
}

static PtnValue ptn_internal_is_null(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_NULL);
}

static PtnValue ptn_internal_is_bool(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_BOOL);
}

static PtnValue ptn_internal_is_int(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_INT);
}

static PtnValue ptn_internal_is_float(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_FLOAT);
}

static PtnValue ptn_internal_is_string(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_STRING);
}

static PtnValue ptn_internal_is_scalar(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_scalar(args[0]);
}

static PtnValue ptn_internal_is_finite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    if (args[0].type != PTN_FLOAT) {
        return ptn_bool(1);
    }
    return ptn_bool(isfinite(args[0].as.floating));
}

static PtnValue ptn_internal_is_infinite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_bool(args[0].type == PTN_FLOAT && isinf(args[0].as.floating));
}

static PtnValue ptn_internal_is_nan(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_bool(args[0].type == PTN_FLOAT && isnan(args[0].as.floating));
}

static PtnValue ptn_internal_bin2hex(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    static const char hex_digits[] = "0123456789abcdef";
    char *string = ptn_value_to_string(args[0]);
    size_t len = strlen(string);
    if (len > (SIZE_MAX - 1) / 2) {
        ptn_abort_out_of_memory();
    }
    char *hex = malloc((len * 2) + 1);
    if (hex == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string[i];
        hex[i * 2] = hex_digits[byte >> 4];
        hex[(i * 2) + 1] = hex_digits[byte & 0x0f];
    }
    hex[len * 2] = '\0';
    free(string);
    return ptn_owned_string(hex);
}

static int ptn_hex_nibble(unsigned char byte) {
    if (byte >= '0' && byte <= '9') {
        return (int)(byte - '0');
    }
    if (byte >= 'a' && byte <= 'f') {
        return 10 + (int)(byte - 'a');
    }
    if (byte >= 'A' && byte <= 'F') {
        return 10 + (int)(byte - 'A');
    }
    return -1;
}

static PtnValue ptn_internal_hex2bin(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *hex = ptn_value_to_string(args[0]);
    size_t len = strlen(hex);
    if ((len % 2) != 0) {
        ptn_emit_warning(
            &runtime->diagnostics,
            "hex2bin(): Hexadecimal input string must have an even length",
            line
        );
        free(hex);
        return ptn_bool(0);
    }

    char *binary = malloc((len / 2) + 1);
    if (binary == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t output_len = 0;
    for (size_t i = 0; i < len; i += 2) {
        int high = ptn_hex_nibble((unsigned char)hex[i]);
        int low = ptn_hex_nibble((unsigned char)hex[i + 1]);
        if (high < 0 || low < 0) {
            ptn_emit_warning(
                &runtime->diagnostics,
                "hex2bin(): Input string must be hexadecimal string",
                line
            );
            free(binary);
            free(hex);
            return ptn_bool(0);
        }
        binary[output_len++] = (char)((high << 4) | low);
    }
    binary[output_len] = '\0';
    free(hex);
    return ptn_owned_string(binary);
}

static char *ptn_quoted_printable_decode_string(const char *input) {
    size_t len = strlen(input);
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        if (input[i] == '=') {
            if (i + 1 < len && input[i + 1] == '\n') {
                i += 1;
                continue;
            }
            if (i + 2 < len && input[i + 1] == '\r' && input[i + 2] == '\n') {
                i += 2;
                continue;
            }
            if (i + 2 < len) {
                int high = ptn_hex_nibble((unsigned char)input[i + 1]);
                int low = ptn_hex_nibble((unsigned char)input[i + 2]);
                if (high >= 0 && low >= 0) {
                    output[out++] = (char)((high << 4) | low);
                    i += 2;
                    continue;
                }
            }
        }
        output[out++] = input[i];
    }
    output[out] = '\0';
    return output;
}

static PtnValue ptn_internal_quoted_printable_decode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *input = ptn_value_to_string(args[0]);
    char *output = ptn_quoted_printable_decode_string(input);
    free(input);
    return ptn_owned_string(output);
}

static int ptn_ascii_is_letter(unsigned char byte) {
    return (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z');
}

static unsigned char ptn_ascii_upper(unsigned char byte) {
    if (byte >= 'a' && byte <= 'z') {
        return (unsigned char)(byte - ('a' - 'A'));
    }
    return byte;
}

static char ptn_soundex_code(unsigned char byte) {
    switch (ptn_ascii_upper(byte)) {
        case 'B':
        case 'F':
        case 'P':
        case 'V':
            return '1';
        case 'C':
        case 'G':
        case 'J':
        case 'K':
        case 'Q':
        case 'S':
        case 'X':
        case 'Z':
            return '2';
        case 'D':
        case 'T':
            return '3';
        case 'L':
            return '4';
        case 'M':
        case 'N':
            return '5';
        case 'R':
            return '6';
        default:
            return '\0';
    }
}

static int ptn_soundex_resets_previous(unsigned char byte) {
    switch (ptn_ascii_upper(byte)) {
        case 'A':
        case 'E':
        case 'I':
        case 'O':
        case 'U':
        case 'Y':
        case 'H':
        case 'W':
            return 1;
        default:
            return 0;
    }
}

static PtnValue ptn_internal_soundex(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *string = ptn_value_to_string(args[0]);
    char *result = malloc(5);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = '0';
    result[1] = '0';
    result[2] = '0';
    result[3] = '0';
    result[4] = '\0';

    size_t first = 0;
    while (string[first] != '\0' && !ptn_ascii_is_letter((unsigned char)string[first])) {
        first++;
    }
    if (string[first] == '\0') {
        free(string);
        return ptn_owned_string(result);
    }

    result[0] = (char)ptn_ascii_upper((unsigned char)string[first]);
    char previous = ptn_soundex_code((unsigned char)string[first]);
    size_t output_len = 1;
    for (size_t i = first + 1; string[i] != '\0' && output_len < 4; i++) {
        unsigned char byte = (unsigned char)string[i];
        char code = ptn_soundex_code(byte);
        if (code == '\0') {
            if (ptn_soundex_resets_previous(byte)) {
                previous = '\0';
            }
            continue;
        }
        if (code != previous) {
            result[output_len++] = code;
        }
        previous = code;
    }

    free(string);
    return ptn_owned_string(result);
}

static double ptn_value_to_double(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    return number.floating;
}

static PtnValue ptn_internal_ceil(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_float(ceil(ptn_value_to_double(args[0])));
}

static PtnValue ptn_internal_floor(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_float(floor(ptn_value_to_double(args[0])));
}

static PtnValue ptn_internal_abs(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    if (args[0].type == PTN_NULL) {
        ptn_emit_array_runtime_diagnostic(
            "Deprecated",
            "abs(): Passing null to parameter #1 ($num) of type int|float is deprecated",
            line
        );
    }

    PtnNumber number = ptn_to_number(args[0]);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(fabs(number.floating));
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(fabs((double)number.integer));
    }
    if (number.integer < 0) {
        return ptn_int(-number.integer);
    }
    return ptn_int(number.integer);
}

static PtnValue ptn_internal_sqrt(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_float(sqrt(ptn_value_to_double(args[0])));
}

static PtnValue ptn_internal_fdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    double dividend = ptn_value_to_double(args[0]);
    double divisor = ptn_value_to_double(args[1]);
    return ptn_float(dividend / divisor);
}

static PtnValue ptn_internal_intdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    int64_t dividend = ptn_value_to_integer_with_precision_deprecation(args[0]);
    int64_t divisor = ptn_value_to_integer_with_precision_deprecation(args[1]);
    if (divisor == 0) {
        ptn_abort_arithmetic_error("Division by zero");
    }
    if (dividend == INT64_MIN && divisor == -1) {
        ptn_abort_arithmetic_error("Division of PHP_INT_MIN by -1 is not an integer");
    }
    return ptn_int(dividend / divisor);
}

static PtnValue ptn_internal_pi(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_float(3.14159265358979323846264338327950288);
}

static PtnValue ptn_internal_getrandmax(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_int(2147483647);
}

static PtnValue ptn_internal_getmypid(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
#if defined(_WIN32)
    return ptn_int((int64_t)_getpid());
#else
    return ptn_int((int64_t)getpid());
#endif
}

static PtnValue ptn_internal_php_sapi_name(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_string(PTN_PHP_SAPI_NAME);
}

static PtnValue ptn_internal_phpversion(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    if (argc == 0) {
        return ptn_string(PTN_PHP_VERSION);
    }

    char *extension = ptn_value_to_string(args[0]);
    int modeled_extension =
        extension[0] == '\0' ||
        ptn_ascii_case_equal(extension, "core") ||
        ptn_ascii_case_equal(extension, "standard");
    free(extension);
    if (modeled_extension) {
        return ptn_string(PTN_PHP_VERSION);
    }
    return ptn_bool(0);
}

static int ptn_digit_value_for_base(unsigned char byte, int base) {
    int value = -1;
    if (byte >= '0' && byte <= '9') {
        value = (int)(byte - '0');
    } else if (byte >= 'a' && byte <= 'f') {
        value = 10 + (int)(byte - 'a');
    } else if (byte >= 'A' && byte <= 'F') {
        value = 10 + (int)(byte - 'A');
    }
    return value >= 0 && value < base ? value : -1;
}

static PtnValue ptn_base_string_to_number(
    PtnRuntime *runtime,
    const char *string,
    int base,
    char prefix,
    size_t line
) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }

    const char *end = string + strlen(string);
    while (end > start && isspace((unsigned char)*(end - 1))) {
        end--;
    }

    if ((end - start) >= 2 && start[0] == '0' && tolower((unsigned char)start[1]) == prefix) {
        start += 2;
    }

    int saw_digit = 0;
    int saw_invalid = 0;
    int fits_integer = 1;
    int64_t integer = 0;
    double floating = 0.0;

    for (const char *cursor = start; cursor < end; cursor++) {
        int digit = ptn_digit_value_for_base((unsigned char)*cursor, base);
        if (digit < 0) {
            saw_invalid = 1;
            continue;
        }
        saw_digit = 1;
        floating = (floating * (double)base) + (double)digit;
        if (fits_integer) {
            if (integer > (INT64_MAX - digit) / base) {
                fits_integer = 0;
            } else {
                integer = (integer * base) + digit;
            }
        }
    }

    if (saw_invalid) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Invalid characters passed for attempted conversion, these have been ignored",
            line
        );
    }
    if (!saw_digit) {
        return ptn_int(0);
    }
    return fits_integer ? ptn_int(integer) : ptn_float(floating);
}

static PtnValue ptn_internal_bindec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *string = ptn_value_to_string(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string, 2, 'b', line);
    free(string);
    return value;
}

static PtnValue ptn_internal_hexdec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *string = ptn_value_to_string(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string, 16, 'x', line);
    free(string);
    return value;
}

static PtnValue ptn_internal_octdec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *string = ptn_value_to_string(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string, 8, 'o', line);
    free(string);
    return value;
}

static PtnValue ptn_internal_intval(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    if (argc >= 2 && args[0].type == PTN_STRING) {
        int64_t base = ptn_number_to_integer(ptn_to_number(args[1]));
        if (base == 0 || (base >= 2 && base <= 36)) {
            const char *start = args[0].as.string;
            while (isspace((unsigned char)*start)) {
                start++;
            }
            errno = 0;
            long long integer = strtoll(start, NULL, (int)base);
            if (errno != ERANGE) {
                return ptn_int((int64_t)integer);
            }
        }
    }
    return ptn_cast_int(args[0]);
}

static PtnValue ptn_internal_chr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    int64_t integer = ptn_value_to_integer(args[0]);
    int64_t normalized = integer % 256;
    if (normalized < 0) {
        normalized += 256;
    }
    char *string = malloc(2);
    if (string == NULL) {
        ptn_abort_out_of_memory();
    }
    string[0] = (char)(unsigned char)normalized;
    string[1] = '\0';
    return ptn_owned_string(string);
}

static PtnValue ptn_internal_ord(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *string = ptn_value_to_string(args[0]);
    size_t len = strlen(string);
    int64_t byte = 0;
    if (len == 0) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "ord(): Providing an empty string is deprecated",
            line
        );
    } else {
        byte = (int64_t)(unsigned char)string[0];
        if (len != 1) {
            ptn_emit_deprecation(
                &runtime->diagnostics,
                "ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead",
                line
            );
        }
    }
    free(string);
    return ptn_int(byte);
}

static PtnValue ptn_internal_count(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    if (args[0].type == PTN_ARRAY) {
        return ptn_int((int64_t)args[0].as.array->len);
    }
    fputs("Fatal error: count(): Argument #1 ($value) must be of type Countable|array, ", stderr);
    fputs(ptn_offset_container_type_name(args[0]), stderr);
    fputs(" given\n", stderr);
    exit(255);
    return ptn_null();
}

static PtnValue ptn_internal_error_reporting(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_int(0);
}

static PtnValue ptn_internal_define(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_constant(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static int ptn_user_function_exists(const char *name);
static PtnValue ptn_internal_defined(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_function_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_array_key_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);

static const PtnInternalFunction *ptn_internal_functions(size_t *count) {
    /* Keep sorted by ASCII case-insensitive name for ptn_find_internal_function. */
    static const PtnInternalFunction functions[] = {
        { "abs", 1, 1, ptn_internal_abs },
        { "array_key_exists", 2, 2, ptn_internal_array_key_exists },
        { "bin2hex", 1, 1, ptn_internal_bin2hex },
        { "bindec", 1, 1, ptn_internal_bindec },
        { "ceil", 1, 1, ptn_internal_ceil },
        { "chr", 1, 1, ptn_internal_chr },
        { "chunk_split", 1, 3, ptn_internal_chunk_split },
        { "constant", 1, 1, ptn_internal_constant },
        { "count", 1, 1, ptn_internal_count },
        { "define", 2, 2, ptn_internal_define },
        { "defined", 1, 1, ptn_internal_defined },
        { "dirname", 1, 1, ptn_internal_dirname },
        { "error_reporting", 0, 1, ptn_internal_error_reporting },
        { "fdiv", 2, 2, ptn_internal_fdiv },
        { "floor", 1, 1, ptn_internal_floor },
        { "function_exists", 1, 1, ptn_internal_function_exists },
        { "getmypid", 0, 0, ptn_internal_getmypid },
        { "getrandmax", 0, 0, ptn_internal_getrandmax },
        { "gettype", 1, 1, ptn_internal_gettype },
        { "hex2bin", 1, 1, ptn_internal_hex2bin },
        { "hexdec", 1, 1, ptn_internal_hexdec },
        { "intdiv", 2, 2, ptn_internal_intdiv },
        { "intval", 1, 2, ptn_internal_intval },
        { "is_bool", 1, 1, ptn_internal_is_bool },
        { "is_double", 1, 1, ptn_internal_is_float },
        { "is_finite", 1, 1, ptn_internal_is_finite },
        { "is_float", 1, 1, ptn_internal_is_float },
        { "is_infinite", 1, 1, ptn_internal_is_infinite },
        { "is_int", 1, 1, ptn_internal_is_int },
        { "is_integer", 1, 1, ptn_internal_is_int },
        { "is_long", 1, 1, ptn_internal_is_int },
        { "is_nan", 1, 1, ptn_internal_is_nan },
        { "is_null", 1, 1, ptn_internal_is_null },
        { "is_scalar", 1, 1, ptn_internal_is_scalar },
        { "is_string", 1, 1, ptn_internal_is_string },
        { "md5", 1, 2, ptn_internal_md5 },
        { "octdec", 1, 1, ptn_internal_octdec },
        { "ord", 1, 1, ptn_internal_ord },
        { "php_sapi_name", 0, 0, ptn_internal_php_sapi_name },
        { "phpversion", 0, 1, ptn_internal_phpversion },
        { "pi", 0, 0, ptn_internal_pi },
        { "quoted_printable_decode", 1, 1, ptn_internal_quoted_printable_decode },
        { "quotemeta", 1, 1, ptn_internal_quotemeta },
        { "sha1", 1, 2, ptn_internal_sha1 },
        { "soundex", 1, 1, ptn_internal_soundex },
        { "sqrt", 1, 1, ptn_internal_sqrt },
        { "str_contains", 2, 2, ptn_internal_str_contains },
        { "str_ends_with", 2, 2, ptn_internal_str_ends_with },
        { "str_rot13", 1, 1, ptn_internal_str_rot13 },
        { "str_starts_with", 2, 2, ptn_internal_str_starts_with },
        { "strcmp", 2, 2, ptn_internal_strcmp },
        { "strip_tags", 1, 1, ptn_internal_strip_tags },
        { "strlen", 1, 1, ptn_internal_strlen },
        { "substr", 2, 3, ptn_internal_substr },
        { "var_dump", 1, PTN_VARIADIC_ARGS, ptn_internal_var_dump },
    };
    *count = sizeof(functions) / sizeof(functions[0]);
    return functions;
}

static const PtnInternalFunction *ptn_find_internal_function(const char *name) {
    size_t count = 0;
    const PtnInternalFunction *functions = ptn_internal_functions(&count);
    size_t low = 0;
    size_t high = count;
    while (low < high) {
        size_t mid = low + ((high - low) / 2);
        int ordering = ptn_ascii_case_compare(name, functions[mid].name);
        if (ordering == 0) {
            return &functions[mid];
        }
        if (ordering < 0) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    return NULL;
}

static PtnValue ptn_internal_define(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *name = ptn_value_to_string(args[0]);
    int did_define = ptn_runtime_define_constant_if_absent(runtime, name, args[1], line);
    free(name);
    return ptn_bool(did_define);
}

static PtnValue ptn_internal_constant(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    PtnValue value = ptn_read_constant(runtime, name);
    free(name);
    return value;
}

static PtnValue ptn_internal_defined(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_runtime_constant_is_defined(runtime, name);
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_function_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_user_function_exists(name) || ptn_find_internal_function(name) != NULL;
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_array_key_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    if (args[1].type != PTN_ARRAY) {
        fputs("Fatal error: array_key_exists(): Argument #2 ($array) must be of type array\n", stderr);
        exit(255);
    }
    if (args[0].type == PTN_NULL) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Using null as the key parameter for array_key_exists() is deprecated, use an empty string instead",
            line
        );
    }
    PtnArrayKey key = ptn_array_key_from_value(args[0]);
    int exists = ptn_array_entry_for_key(args[1].as.array, key) != NULL;
    ptn_array_key_free(key);
    return ptn_bool(exists);
}

static PTN_UNUSED PtnValue ptn_call_internal(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line) {
    const PtnInternalFunction *function = ptn_find_internal_function(name);
    if (function != NULL) {
        if (argc < function->min_args) {
            ptn_emit_argument_count_error(&runtime->diagnostics, name, function->min_args, argc);
            exit(255);
        }
        if (function->max_args != PTN_VARIADIC_ARGS && argc > function->max_args) {
            ptn_emit_too_many_arguments_error(&runtime->diagnostics, name, function->max_args, argc);
            exit(255);
        }
        return function->handler(runtime, argc, args, line);
    }

    ptn_emit_undefined_function_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}
"#;
