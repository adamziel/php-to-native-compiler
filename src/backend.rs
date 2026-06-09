use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{
    ArrayElement as IrArrayElement, BinaryOp, CastKind, FunctionDecl, IncDecOp, Instruction,
    MagicConstantKind, Module, TypeHint, UnaryOp, ValueExpr,
};

mod runtime;

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    out.push_str(runtime::RUNTIME_C);
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
            } => {
                let predicate = self.emit_condition(out, expr);
                format!("!({predicate})")
            }
            ValueExpr::Binary { op, left, right } => match op {
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
                    let emitted_value = self.emit_value(out, value);
                    format!("ptn_is_truthy({emitted_value})")
                }
            },
            _ => {
                let emitted_value = self.emit_value(out, value);
                format!("ptn_is_truthy({emitted_value})")
            }
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

    fn emit_comparison_predicate(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        match op {
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
        }
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
