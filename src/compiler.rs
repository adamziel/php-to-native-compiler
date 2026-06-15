use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{
    ArrayDimTarget, ArrayElementValue, AssignmentTarget, BinaryOp, CatchClause, Expr,
    ListAssignmentElementTarget, MagicConstantKind, Program, ReferenceTarget, Statement,
    SwitchCase, UnsetTarget,
};
use crate::backend::{compile_c, emit_c};
use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{lower_with_source_and_includes, IncludeResolutionMap, IncludeSource};
use crate::parser::parse;

const MAX_BOUNDED_INCLUDE_CANDIDATES: usize = 32;

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
    let source_file = input.to_string_lossy().into_owned();
    let source_dir = input
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut includes = IncludeCollector::new();
    includes.collect_program(&program, &source_file, &source_dir)?;
    let include_sources = includes.sources;
    let include_resolutions = includes.resolutions;
    let module = lower_with_source_and_includes(
        &program,
        source_file,
        source_dir,
        include_sources,
        &include_resolutions,
    );
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

struct IncludeCollector {
    sources: Vec<IncludeSource>,
    by_path: HashMap<PathBuf, usize>,
    resolutions: IncludeResolutionMap,
}

impl IncludeCollector {
    fn new() -> Self {
        Self {
            sources: Vec::new(),
            by_path: HashMap::new(),
            resolutions: IncludeResolutionMap::new(),
        }
    }

    fn collect_program(
        &mut self,
        program: &Program,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        for function in &program.functions {
            self.collect_statements(&function.body, source_file, source_dir)?;
        }
        for class in &program.classes {
            for property in &class.properties {
                if let Some(value) = &property.value {
                    self.collect_expr(value, source_file, source_dir)?;
                }
            }
            for property in &class.static_properties {
                if let Some(value) = &property.value {
                    self.collect_expr(value, source_file, source_dir)?;
                }
            }
            for method in &class.methods {
                self.collect_statements(&method.body, source_file, source_dir)?;
            }
        }
        self.collect_statements(&program.statements, source_file, source_dir)
    }

    fn collect_statements(
        &mut self,
        statements: &[Statement],
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        for statement in statements {
            self.collect_statement(statement, source_file, source_dir)?;
        }
        Ok(())
    }

    fn collect_statement(
        &mut self,
        statement: &Statement,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        match statement {
            Statement::Assign { value, .. }
            | Statement::Print {
                expression: value, ..
            } => self.collect_expr(value, source_file, source_dir),
            Statement::AssignRef { source, .. } => {
                self.collect_expr(source, source_file, source_dir)
            }
            Statement::ArrayAssign { target, value, .. } => {
                self.collect_array_dim_target(target, source_file, source_dir)?;
                self.collect_expr(value, source_file, source_dir)
            }
            Statement::ArrayAssignRef { target, source, .. } => {
                self.collect_array_dim_target(target, source_file, source_dir)?;
                self.collect_expr(source, source_file, source_dir)
            }
            Statement::Unset { targets, .. } => {
                for target in targets {
                    match target {
                        UnsetTarget::ArrayDim(target) => {
                            self.collect_array_dim_target(target, source_file, source_dir)?;
                        }
                        UnsetTarget::DynamicArrayDim {
                            name, dimensions, ..
                        } => {
                            self.collect_expr(name, source_file, source_dir)?;
                            for dimension in dimensions {
                                self.collect_expr(dimension, source_file, source_dir)?;
                            }
                        }
                        UnsetTarget::PropertyArrayDim {
                            receiver,
                            dimensions,
                            ..
                        } => {
                            self.collect_expr(receiver, source_file, source_dir)?;
                            for dimension in dimensions {
                                self.collect_expr(dimension, source_file, source_dir)?;
                            }
                        }
                        UnsetTarget::Property { receiver, .. } => {
                            self.collect_expr(receiver, source_file, source_dir)?;
                        }
                        UnsetTarget::DynamicVariable { name, .. } => {
                            self.collect_expr(name, source_file, source_dir)?;
                        }
                        UnsetTarget::Variable { .. } => {}
                    }
                }
                Ok(())
            }
            Statement::Call { arguments, .. }
            | Statement::Echo {
                expressions: arguments,
                ..
            } => self.collect_exprs(arguments, source_file, source_dir),
            Statement::Expression { expression, .. } => {
                self.collect_expr(expression, source_file, source_dir)
            }
            Statement::Throw { value, .. } => self.collect_expr(value, source_file, source_dir),
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    self.collect_expr(&declaration.value, source_file, source_dir)?;
                }
                Ok(())
            }
            Statement::Block { statements, .. } => {
                self.collect_statements(statements, source_file, source_dir)
            }
            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.collect_expr(condition, source_file, source_dir)?;
                self.collect_statements(then_body, source_file, source_dir)?;
                self.collect_statements(else_body, source_file, source_dir)
            }
            Statement::While {
                condition, body, ..
            } => {
                self.collect_expr(condition, source_file, source_dir)?;
                self.collect_statements(body, source_file, source_dir)
            }
            Statement::DoWhile {
                body, condition, ..
            } => {
                self.collect_statements(body, source_file, source_dir)?;
                self.collect_expr(condition, source_file, source_dir)
            }
            Statement::For {
                initializers,
                condition,
                updates,
                body,
                ..
            } => {
                self.collect_statements(initializers, source_file, source_dir)?;
                if let Some(condition) = condition {
                    self.collect_expr(condition, source_file, source_dir)?;
                }
                self.collect_statements(updates, source_file, source_dir)?;
                self.collect_statements(body, source_file, source_dir)
            }
            Statement::Foreach {
                iterable,
                key,
                value,
                body,
                ..
            } => {
                self.collect_expr(iterable, source_file, source_dir)?;
                if let Some(key) = key {
                    self.collect_assignment_target(key, source_file, source_dir)?;
                }
                self.collect_assignment_target(value, source_file, source_dir)?;
                self.collect_statements(body, source_file, source_dir)
            }
            Statement::Switch {
                expression, cases, ..
            } => {
                self.collect_expr(expression, source_file, source_dir)?;
                for case in cases {
                    self.collect_switch_case(case, source_file, source_dir)?;
                }
                Ok(())
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    self.collect_expr(value, source_file, source_dir)?;
                }
                Ok(())
            }
            Statement::Try { body, catches, .. } => {
                self.collect_statements(body, source_file, source_dir)?;
                for catch in catches {
                    self.collect_catch(catch, source_file, source_dir)?;
                }
                Ok(())
            }
            Statement::Increment { .. }
            | Statement::Empty { .. }
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Global { .. }
            | Statement::Label { .. }
            | Statement::Goto { .. }
            | Statement::InlineHtml { .. } => Ok(()),
        }
    }

    fn collect_switch_case(
        &mut self,
        case: &SwitchCase,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        if let Some(condition) = &case.condition {
            self.collect_expr(condition, source_file, source_dir)?;
        }
        self.collect_statements(&case.body, source_file, source_dir)
    }

    fn collect_catch(
        &mut self,
        catch: &CatchClause,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        self.collect_statements(&catch.body, source_file, source_dir)
    }

    fn collect_exprs(&mut self, exprs: &[Expr], source_file: &str, source_dir: &str) -> Result<()> {
        for expr in exprs {
            self.collect_expr(expr, source_file, source_dir)?;
        }
        Ok(())
    }

    fn collect_expr(&mut self, expr: &Expr, source_file: &str, source_dir: &str) -> Result<()> {
        match expr {
            Expr::Include { path, span, .. } => {
                let candidates = self.resolve_include(path, *span, source_file, source_dir)?;
                self.resolutions.insert(
                    (source_file.to_string(), span.byte_start, span.byte_end),
                    candidates,
                );
                Ok(())
            }
            Expr::AnonymousFunction(function) => {
                self.collect_statements(&function.body, source_file, source_dir)
            }
            Expr::Assign { target, value, .. } => {
                self.collect_assignment_target(target, source_file, source_dir)?;
                self.collect_expr(value, source_file, source_dir)
            }
            Expr::AssignRef { target, source, .. } => {
                self.collect_assignment_target(target, source_file, source_dir)?;
                self.collect_expr(source, source_file, source_dir)
            }
            Expr::Call { arguments, .. } | Expr::NewObject { arguments, .. } => {
                self.collect_exprs(arguments, source_file, source_dir)
            }
            Expr::DynamicNewObject {
                class_name,
                arguments,
                ..
            } => {
                self.collect_expr(class_name, source_file, source_dir)?;
                self.collect_exprs(arguments, source_file, source_dir)
            }
            Expr::FirstClassCallable { callable, .. } => {
                self.collect_expr(callable, source_file, source_dir)
            }
            Expr::DynamicCall {
                callee, arguments, ..
            } => {
                self.collect_expr(callee, source_file, source_dir)?;
                self.collect_exprs(arguments, source_file, source_dir)
            }
            Expr::MethodCall {
                receiver,
                arguments,
                ..
            } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_exprs(arguments, source_file, source_dir)
            }
            Expr::PropertyFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::DynamicClassNameFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::InstanceOf { expr, .. } => self.collect_expr(expr, source_file, source_dir),
            Expr::Array { elements, .. } => {
                for element in elements {
                    if let Some(key) = &element.key {
                        self.collect_expr(key, source_file, source_dir)?;
                    }
                    match &element.value {
                        ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                            self.collect_expr(value, source_file, source_dir)?;
                        }
                        ArrayElementValue::Reference(target) => {
                            self.collect_reference_target(target, source_file, source_dir)?;
                        }
                    }
                }
                Ok(())
            }
            Expr::List(list) => {
                for element in &list.elements {
                    if let Some(key) = &element.key {
                        self.collect_expr(key, source_file, source_dir)?;
                    }
                    match &element.target {
                        Some(crate::ast::ListExprElementTarget::Value(value)) => {
                            self.collect_expr(value, source_file, source_dir)?;
                        }
                        Some(crate::ast::ListExprElementTarget::Reference(target)) => {
                            self.collect_reference_target(target, source_file, source_dir)?;
                        }
                        None => {}
                    }
                }
                Ok(())
            }
            Expr::ArrayAccess { array, index, .. } => {
                self.collect_expr(array, source_file, source_dir)?;
                if let Some(index) = index {
                    self.collect_expr(index, source_file, source_dir)?;
                }
                Ok(())
            }
            Expr::Isset { targets, .. } => self.collect_exprs(targets, source_file, source_dir),
            Expr::Empty { target, .. }
            | Expr::Print {
                expression: target, ..
            }
            | Expr::DynamicVariable { name: target, .. }
            | Expr::Clone { expr: target, .. }
            | Expr::Throw { value: target, .. }
            | Expr::Unary { expr: target, .. }
            | Expr::Cast { expr: target, .. }
            | Expr::Grouped { expr: target, .. } => {
                self.collect_expr(target, source_file, source_dir)
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expr(left, source_file, source_dir)?;
                self.collect_expr(right, source_file, source_dir)
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                self.collect_expr(condition, source_file, source_dir)?;
                if let Some(if_true) = if_true {
                    self.collect_expr(if_true, source_file, source_dir)?;
                }
                self.collect_expr(if_false, source_file, source_dir)
            }
            Expr::String(_, _)
            | Expr::InterpolatedString(_, _)
            | Expr::ShellExec { .. }
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::Bool(_, _)
            | Expr::Null(_)
            | Expr::Variable(_, _)
            | Expr::IncDec { .. }
            | Expr::Constant(_, _)
            | Expr::MagicConstant(_, _)
            | Expr::StaticPropertyFetch { .. }
            | Expr::ClassConstantFetch { .. } => Ok(()),
        }
    }

    fn collect_assignment_target(
        &mut self,
        target: &AssignmentTarget,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        match target {
            AssignmentTarget::ArrayDim(target) => {
                self.collect_array_dim_target(target, source_file, source_dir)
            }
            AssignmentTarget::PropertyArrayDim {
                receiver,
                dimensions,
                ..
            } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                for dimension in dimensions {
                    if let Some(dimension) = dimension {
                        self.collect_expr(dimension, source_file, source_dir)?;
                    }
                }
                Ok(())
            }
            AssignmentTarget::Property { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            AssignmentTarget::List(target) => {
                for element in &target.elements {
                    if let Some(key) = &element.key {
                        self.collect_expr(key, source_file, source_dir)?;
                    }
                    match &element.target {
                        ListAssignmentElementTarget::Value(target) => {
                            self.collect_assignment_target(target, source_file, source_dir)?;
                        }
                        ListAssignmentElementTarget::Reference(target) => {
                            self.collect_reference_target(target, source_file, source_dir)?;
                        }
                    }
                }
                Ok(())
            }
            AssignmentTarget::DynamicVariable { name, .. } => {
                self.collect_expr(name, source_file, source_dir)
            }
            AssignmentTarget::DynamicArrayDim {
                name, dimensions, ..
            } => {
                self.collect_expr(name, source_file, source_dir)?;
                for dimension in dimensions {
                    if let Some(dimension) = dimension {
                        self.collect_expr(dimension, source_file, source_dir)?;
                    }
                }
                Ok(())
            }
            AssignmentTarget::Variable { .. } | AssignmentTarget::StaticProperty { .. } => Ok(()),
        }
    }

    fn collect_reference_target(
        &mut self,
        target: &ReferenceTarget,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        match target {
            ReferenceTarget::ArrayDim(target) => {
                self.collect_array_dim_target(target, source_file, source_dir)
            }
            ReferenceTarget::Property { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            ReferenceTarget::Variable { .. } => Ok(()),
        }
    }

    fn collect_array_dim_target(
        &mut self,
        target: &ArrayDimTarget,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        for dimension in &target.dimensions {
            if let Some(dimension) = dimension {
                self.collect_expr(dimension, source_file, source_dir)?;
            }
        }
        Ok(())
    }

    fn resolve_include(
        &mut self,
        path: &Expr,
        span: crate::diagnostic::SourceSpan,
        source_file: &str,
        source_dir: &str,
    ) -> Result<Vec<usize>> {
        let include_paths = bounded_include_paths(path, source_file, source_dir).ok_or_else(|| {
            Diagnostic::new(
                "dynamic include paths are unsupported; use a compile-time string path or bounded conditional of compile-time string paths",
                Some(path.span()),
            )
        })?;
        let mut candidates = Vec::new();
        for include_path in include_paths {
            let index = self.resolve_include_candidate(&include_path, span, source_dir)?;
            if !candidates.contains(&index) {
                candidates.push(index);
            }
        }
        Ok(candidates)
    }

    fn resolve_include_candidate(
        &mut self,
        include_path: &str,
        span: crate::diagnostic::SourceSpan,
        source_dir: &str,
    ) -> Result<usize> {
        let resolved_path = resolve_include_path(include_path, source_dir);
        let canonical_path = fs::canonicalize(&resolved_path).map_err(|error| {
            Diagnostic::new(
                format!(
                    "failed to resolve included file {}: {error}",
                    resolved_path.display()
                ),
                Some(span),
            )
        })?;
        let path_aliases = include_path_aliases(&resolved_path, &canonical_path);
        if let Some(index) = self.by_path.get(&canonical_path).copied() {
            self.add_path_aliases(index, path_aliases);
            return Ok(index);
        }

        let source = fs::read_to_string(&canonical_path).map_err(|error| {
            Diagnostic::new(
                format!(
                    "failed to read included file {}: {error}",
                    canonical_path.display()
                ),
                Some(span),
            )
        })?;
        let program = parse(&source)?;
        if !program.classes.is_empty() {
            return Err(Diagnostic::new(
                "include files with class declarations are unsupported",
                Some(span),
            ));
        }

        let index = self.sources.len();
        self.by_path.insert(canonical_path.clone(), index);
        let source_file = canonical_path.to_string_lossy().into_owned();
        let source_dir = canonical_path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.sources.push(IncludeSource {
            source_file: source_file.clone(),
            source_dir: source_dir.clone(),
            path_aliases,
            program: program.clone(),
        });
        self.collect_program(&program, &source_file, &source_dir)?;
        Ok(index)
    }

    fn add_path_aliases(&mut self, index: usize, aliases: Vec<String>) {
        let source = &mut self.sources[index];
        for alias in aliases {
            if !source.path_aliases.contains(&alias) {
                source.path_aliases.push(alias);
            }
        }
    }
}

fn bounded_include_paths(expr: &Expr, source_file: &str, source_dir: &str) -> Option<Vec<String>> {
    match expr {
        Expr::String(value, _) => Some(vec![value.clone()]),
        Expr::ShellExec { .. } => None,
        Expr::MagicConstant(MagicConstantKind::File, _) => Some(vec![source_file.to_string()]),
        Expr::MagicConstant(MagicConstantKind::Dir, _) => Some(vec![source_dir.to_string()]),
        Expr::Binary {
            op: BinaryOp::Concat,
            left,
            right,
            ..
        } => {
            let left_paths = bounded_include_paths(left, source_file, source_dir)?;
            let right_paths = bounded_include_paths(right, source_file, source_dir)?;
            if left_paths.len().saturating_mul(right_paths.len()) > MAX_BOUNDED_INCLUDE_CANDIDATES {
                return None;
            }
            let mut paths = Vec::new();
            for left_path in &left_paths {
                for right_path in &right_paths {
                    let mut path = left_path.clone();
                    path.push_str(right_path);
                    push_unique_string(&mut paths, path);
                }
            }
            Some(paths)
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            let mut paths = Vec::new();
            let true_expr = if_true.as_deref().unwrap_or(condition);
            for path in bounded_include_paths(true_expr, source_file, source_dir)? {
                push_unique_string(&mut paths, path);
            }
            for path in bounded_include_paths(if_false, source_file, source_dir)? {
                push_unique_string(&mut paths, path);
            }
            if paths.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                return None;
            }
            Some(paths)
        }
        Expr::Grouped { expr, .. } => bounded_include_paths(expr, source_file, source_dir),
        _ => None,
    }
}

fn push_unique_string(strings: &mut Vec<String>, value: String) {
    if !strings.contains(&value) {
        strings.push(value);
    }
}

fn resolve_include_path(path: &str, source_dir: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        Path::new(source_dir).join(path)
    }
}

fn include_path_aliases(resolved_path: &Path, canonical_path: &Path) -> Vec<String> {
    let mut aliases = Vec::new();
    push_unique_string(&mut aliases, resolved_path.to_string_lossy().into_owned());
    push_unique_string(&mut aliases, canonical_path.to_string_lossy().into_owned());
    aliases
}
