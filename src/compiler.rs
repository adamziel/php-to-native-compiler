use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{
    ArrayDimTarget, ArrayElementValue, AssignmentTarget, Expr, FunctionDecl,
    ListAssignmentElementTarget, ListAssignmentTarget, Program, ReferenceTarget, Statement,
    UnsetTarget,
};
use crate::backend::{compile_c, emit_c};
use crate::diagnostic::{Diagnostic, Result};
use crate::ir::lower_with_source;
use crate::parser::parse;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub emit_c: bool,
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub binary: PathBuf,
    pub c_source: Option<PathBuf>,
}

pub fn compile_file(input: &Path, output: &Path, options: CompileOptions) -> Result<CompileOutput> {
    let source = fs::read_to_string(input).map_err(|error| {
        Diagnostic::new(format!("failed to read {}: {error}", input.display()), None)
    })?;
    let program = parse(&source)?;
    let source_dir_path = input.parent().unwrap_or_else(|| Path::new(""));
    let program = expand_literal_includes(program, source_dir_path)?;
    let source_file = input.to_string_lossy().into_owned();
    let source_dir = input
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    let module = lower_with_source(&program, source_file, source_dir);
    let c_source = emit_c(&module);
    compile_c(&c_source, output)?;
    let c_path = output.with_extension("c");
    if !options.emit_c {
        let _ = fs::remove_file(&c_path);
    }
    Ok(CompileOutput {
        binary: output.to_path_buf(),
        c_source: options.emit_c.then_some(c_path),
    })
}

fn expand_literal_includes(program: Program, source_dir: &Path) -> Result<Program> {
    expand_program_includes(program, source_dir, &mut Vec::new())
}

fn expand_program_includes(
    mut program: Program,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
) -> Result<Program> {
    let mut extra_functions = Vec::new();
    program.statements = expand_statements(
        program.statements,
        source_dir,
        include_stack,
        &mut extra_functions,
    )?;
    for function in &mut program.functions {
        let body = std::mem::take(&mut function.body);
        function.body = expand_statements(body, source_dir, include_stack, &mut extra_functions)?;
    }
    program.functions.extend(extra_functions);
    Ok(program)
}

fn expand_statements(
    mut statements: Vec<Statement>,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<Vec<Statement>> {
    for statement in &mut statements {
        expand_statement_includes(statement, source_dir, include_stack, extra_functions)?;
    }
    Ok(statements)
}

fn expand_statement_includes(
    statement: &mut Statement,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    match statement {
        Statement::Assign { value, .. }
        | Statement::AssignRef { source: value, .. }
        | Statement::Print {
            expression: value, ..
        }
        | Statement::Expression {
            expression: value, ..
        } => expand_expr_includes(value, source_dir, include_stack, extra_functions)?,
        Statement::ArrayAssign { target, value, .. } => {
            expand_array_dim_target_includes(target, source_dir, include_stack, extra_functions)?;
            expand_expr_includes(value, source_dir, include_stack, extra_functions)?;
        }
        Statement::ArrayAssignRef { target, source, .. } => {
            expand_array_dim_target_includes(target, source_dir, include_stack, extra_functions)?;
            expand_expr_includes(source, source_dir, include_stack, extra_functions)?;
        }
        Statement::Return {
            value: Some(value), ..
        } => expand_expr_includes(value, source_dir, include_stack, extra_functions)?,
        Statement::Call { arguments, .. }
        | Statement::Echo {
            expressions: arguments,
            ..
        } => {
            for argument in arguments {
                expand_expr_includes(argument, source_dir, include_stack, extra_functions)?;
            }
        }
        Statement::Const { declarations, .. } => {
            for declaration in declarations {
                expand_expr_includes(
                    &mut declaration.value,
                    source_dir,
                    include_stack,
                    extra_functions,
                )?;
            }
        }
        Statement::Unset { targets, .. } => {
            for target in targets {
                expand_unset_target_includes(target, source_dir, include_stack, extra_functions)?;
            }
        }
        Statement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expand_expr_includes(condition, source_dir, include_stack, extra_functions)?;
            *then_body = expand_statements(
                std::mem::take(then_body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
            *else_body = expand_statements(
                std::mem::take(else_body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
        Statement::Block { statements, .. } => {
            *statements = expand_statements(
                std::mem::take(statements),
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
        Statement::While {
            condition, body, ..
        }
        | Statement::DoWhile {
            condition, body, ..
        } => {
            expand_expr_includes(condition, source_dir, include_stack, extra_functions)?;
            *body = expand_statements(
                std::mem::take(body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
        Statement::For {
            initializers,
            condition,
            updates,
            body,
            ..
        } => {
            *initializers = expand_statements(
                std::mem::take(initializers),
                source_dir,
                include_stack,
                extra_functions,
            )?;
            if let Some(condition) = condition {
                expand_expr_includes(condition, source_dir, include_stack, extra_functions)?;
            }
            *updates = expand_statements(
                std::mem::take(updates),
                source_dir,
                include_stack,
                extra_functions,
            )?;
            *body = expand_statements(
                std::mem::take(body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
        Statement::Foreach { iterable, body, .. } => {
            expand_expr_includes(iterable, source_dir, include_stack, extra_functions)?;
            *body = expand_statements(
                std::mem::take(body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
        Statement::Switch {
            expression, cases, ..
        } => {
            expand_expr_includes(expression, source_dir, include_stack, extra_functions)?;
            for case in cases {
                if let Some(condition) = &mut case.condition {
                    expand_expr_includes(condition, source_dir, include_stack, extra_functions)?;
                }
                case.body = expand_statements(
                    std::mem::take(&mut case.body),
                    source_dir,
                    include_stack,
                    extra_functions,
                )?;
            }
        }
        Statement::Try { body, catches, .. } => {
            *body = expand_statements(
                std::mem::take(body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
            for catch in catches {
                catch.body = expand_statements(
                    std::mem::take(&mut catch.body),
                    source_dir,
                    include_stack,
                    extra_functions,
                )?;
            }
        }
        Statement::Return { value: None, .. }
        | Statement::Increment { .. }
        | Statement::Break { .. }
        | Statement::Continue { .. }
        | Statement::Label { .. }
        | Statement::Goto { .. }
        | Statement::InlineHtml { .. } => {}
    }
    Ok(())
}

fn expand_expr_includes(
    expr: &mut Expr,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    match expr {
        Expr::Include { path, body, span } => {
            let Expr::String(relative_path, path_span) = path.as_ref() else {
                return Err(Diagnostic::new(
                    "dynamic include paths are unsupported; only literal include paths are currently modeled",
                    Some(path.span()),
                ));
            };
            let include_path = resolve_include_path(source_dir, relative_path);
            let source = fs::read_to_string(&include_path).map_err(|error| {
                Diagnostic::new(
                    format!("failed to read include {}: {error}", include_path.display()),
                    Some(*path_span),
                )
            })?;
            let included_program = parse(&source)?;
            let include_key = fs::canonicalize(&include_path).unwrap_or(include_path.clone());
            if include_stack.contains(&include_key) {
                return Err(Diagnostic::new(
                    format!(
                        "recursive include is unsupported: {}",
                        include_path.display()
                    ),
                    Some(*span),
                ));
            }
            include_stack.push(include_key);
            let include_dir = include_path.parent().unwrap_or_else(|| Path::new(""));
            let included_program =
                expand_program_includes(included_program, include_dir, include_stack)?;
            include_stack.pop();
            extra_functions.extend(included_program.functions);
            *body = included_program.statements;
        }
        Expr::AnonymousFunction(function) => {
            function.body = expand_statements(
                std::mem::take(&mut function.body),
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
        Expr::Assign { target, value, .. } => {
            expand_assignment_target_includes(target, source_dir, include_stack, extra_functions)?;
            expand_expr_includes(value, source_dir, include_stack, extra_functions)?;
        }
        Expr::AssignRef { target, source, .. } => {
            expand_assignment_target_includes(target, source_dir, include_stack, extra_functions)?;
            expand_expr_includes(source, source_dir, include_stack, extra_functions)?;
        }
        Expr::Call { arguments, .. }
        | Expr::DynamicCall { arguments, .. }
        | Expr::MethodCall { arguments, .. }
        | Expr::NewObject { arguments, .. } => {
            for argument in arguments {
                expand_expr_includes(argument, source_dir, include_stack, extra_functions)?;
            }
            if let Expr::DynamicCall { callee, .. } = expr {
                expand_expr_includes(callee, source_dir, include_stack, extra_functions)?;
            }
            if let Expr::MethodCall { receiver, .. } = expr {
                expand_expr_includes(receiver, source_dir, include_stack, extra_functions)?;
            }
        }
        Expr::PropertyFetch { receiver, .. } => {
            expand_expr_includes(receiver, source_dir, include_stack, extra_functions)?;
        }
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &mut element.key {
                    expand_expr_includes(key, source_dir, include_stack, extra_functions)?;
                }
                match &mut element.value {
                    ArrayElementValue::Value(value) => {
                        expand_expr_includes(value, source_dir, include_stack, extra_functions)?;
                    }
                    ArrayElementValue::Reference(target) => {
                        expand_reference_target_includes(
                            target,
                            source_dir,
                            include_stack,
                            extra_functions,
                        )?;
                    }
                }
            }
        }
        Expr::ArrayAccess { array, index, .. } => {
            expand_expr_includes(array, source_dir, include_stack, extra_functions)?;
            if let Some(index) = index {
                expand_expr_includes(index, source_dir, include_stack, extra_functions)?;
            }
        }
        Expr::Isset { targets, .. } => {
            for target in targets {
                expand_expr_includes(target, source_dir, include_stack, extra_functions)?;
            }
        }
        Expr::Empty { target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::Cast { expr: target, .. }
        | Expr::Grouped { expr: target, .. } => {
            expand_expr_includes(target, source_dir, include_stack, extra_functions)?;
        }
        Expr::Binary { left, right, .. } => {
            expand_expr_includes(left, source_dir, include_stack, extra_functions)?;
            expand_expr_includes(right, source_dir, include_stack, extra_functions)?;
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _) => {}
    }
    Ok(())
}

fn expand_assignment_target_includes(
    target: &mut AssignmentTarget,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    match target {
        AssignmentTarget::Variable { .. } => {}
        AssignmentTarget::ArrayDim(target) => {
            expand_array_dim_target_includes(target, source_dir, include_stack, extra_functions)?;
        }
        AssignmentTarget::Property { receiver, .. } => {
            expand_expr_includes(receiver, source_dir, include_stack, extra_functions)?;
        }
        AssignmentTarget::List(target) => {
            expand_list_assignment_target_includes(
                target,
                source_dir,
                include_stack,
                extra_functions,
            )?;
        }
    }
    Ok(())
}

fn expand_list_assignment_target_includes(
    target: &mut ListAssignmentTarget,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    for element in &mut target.elements {
        if let Some(key) = &mut element.key {
            expand_expr_includes(key, source_dir, include_stack, extra_functions)?;
        }
        match &mut element.target {
            ListAssignmentElementTarget::Value(target) => {
                expand_assignment_target_includes(
                    target,
                    source_dir,
                    include_stack,
                    extra_functions,
                )?;
            }
            ListAssignmentElementTarget::Reference(target) => {
                expand_reference_target_includes(
                    target,
                    source_dir,
                    include_stack,
                    extra_functions,
                )?;
            }
        }
    }
    Ok(())
}

fn expand_reference_target_includes(
    target: &mut ReferenceTarget,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    match target {
        ReferenceTarget::Variable { .. } => {}
        ReferenceTarget::ArrayDim(target) => {
            expand_array_dim_target_includes(target, source_dir, include_stack, extra_functions)?;
        }
    }
    Ok(())
}

fn expand_unset_target_includes(
    target: &mut UnsetTarget,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    match target {
        UnsetTarget::Variable { .. } => {}
        UnsetTarget::ArrayDim(target) => {
            expand_array_dim_target_includes(target, source_dir, include_stack, extra_functions)?;
        }
    }
    Ok(())
}

fn expand_array_dim_target_includes(
    target: &mut ArrayDimTarget,
    source_dir: &Path,
    include_stack: &mut Vec<PathBuf>,
    extra_functions: &mut Vec<FunctionDecl>,
) -> Result<()> {
    for dimension in &mut target.dimensions {
        if let Some(dimension) = dimension {
            expand_expr_includes(dimension, source_dir, include_stack, extra_functions)?;
        }
    }
    Ok(())
}

fn resolve_include_path(source_dir: &Path, relative_path: &str) -> PathBuf {
    let path = Path::new(relative_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        source_dir.join(path)
    }
}
