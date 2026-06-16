use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{
    ArrayDimTarget, ArrayElementValue, AssignmentOp, AssignmentTarget, BinaryOp, CatchClause, Expr,
    IncDecTarget, InstanceOfTarget, ListAssignmentElementTarget, MagicConstantKind, Program,
    ReferenceTarget, Statement, SwitchCase, UnsetTarget,
};
use crate::backend::{compile_c, emit_c};
use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{lower_with_source_and_includes, IncludeResolutionMap, IncludeSource};
use crate::lexer::decode_php_source_bytes;
use crate::parser::{parse, parse_with_runtime_class_aliases};

const MAX_BOUNDED_INCLUDE_CANDIDATES: usize = 32;

type IncludePathEnv = HashMap<String, Vec<String>>;

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
    let source_bytes = fs::read(input).map_err(|error| {
        Diagnostic::new(format!("failed to read {}: {error}", input.display()), None)
    })?;
    let source = decode_php_source_bytes(&source_bytes);
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
    path_env: IncludePathEnv,
    runtime_class_aliases: HashMap<String, String>,
}

impl IncludeCollector {
    fn new() -> Self {
        Self {
            sources: Vec::new(),
            by_path: HashMap::new(),
            resolutions: IncludeResolutionMap::new(),
            path_env: IncludePathEnv::new(),
            runtime_class_aliases: HashMap::new(),
        }
    }

    fn collect_program(
        &mut self,
        program: &Program,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        self.collect_with_fresh_path_env(|collector| {
            collector.collect_program_with_current_env(program, source_file, source_dir)
        })
    }

    fn collect_program_with_current_env(
        &mut self,
        program: &Program,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        for function in &program.functions {
            self.collect_with_fresh_path_env(|collector| {
                collector.collect_statements(&function.body, source_file, source_dir)
            })?;
        }
        for class in &program.classes {
            for property in &class.properties {
                if let Some(value) = &property.value {
                    self.collect_with_fresh_path_env(|collector| {
                        collector.collect_expr(value, source_file, source_dir)
                    })?;
                }
            }
            for property in &class.static_properties {
                if let Some(value) = &property.value {
                    self.collect_with_fresh_path_env(|collector| {
                        collector.collect_expr(value, source_file, source_dir)
                    })?;
                }
            }
            for method in &class.methods {
                self.collect_with_fresh_path_env(|collector| {
                    collector.collect_statements(&method.body, source_file, source_dir)
                })?;
            }
        }
        self.collect_top_level_statements(&program.statements, source_file, source_dir)
    }

    fn collect_top_level_statements(
        &mut self,
        statements: &[Statement],
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        for statement in statements {
            self.note_runtime_class_alias_statement(statement);
            self.collect_statement(statement, source_file, source_dir)?;
        }
        Ok(())
    }

    fn collect_with_fresh_path_env<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let saved_env = std::mem::take(&mut self.path_env);
        let result = f(self);
        self.path_env = saved_env;
        result
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
            Statement::Assign {
                name, op, value, ..
            } => self.collect_direct_assignment(name, *op, value, source_file, source_dir),
            Statement::Print {
                expression: value, ..
            } => self.collect_expr(value, source_file, source_dir),
            Statement::AssignRef { name, source, .. } => {
                self.collect_expr(source, source_file, source_dir)?;
                self.path_env.remove(name);
                Ok(())
            }
            Statement::ArrayAssign { target, value, .. } => {
                self.collect_array_dim_target(target, source_file, source_dir)?;
                self.collect_expr(value, source_file, source_dir)?;
                self.path_env.remove(&target.array);
                Ok(())
            }
            Statement::ArrayAssignRef { target, source, .. } => {
                self.collect_array_dim_target(target, source_file, source_dir)?;
                self.collect_expr(source, source_file, source_dir)?;
                self.path_env.remove(&target.array);
                Ok(())
            }
            Statement::Unset { targets, .. } => {
                for target in targets {
                    match target {
                        UnsetTarget::Variable { name, .. } => {
                            self.path_env.remove(name);
                        }
                        UnsetTarget::ArrayDim(target) => {
                            self.collect_array_dim_target(target, source_file, source_dir)?;
                            self.path_env.remove(&target.array);
                        }
                        UnsetTarget::DynamicArrayDim {
                            name, dimensions, ..
                        } => {
                            self.collect_expr(name, source_file, source_dir)?;
                            for dimension in dimensions {
                                self.collect_expr(dimension, source_file, source_dir)?;
                            }
                            self.path_env.clear();
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
                            self.path_env.clear();
                        }
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
            Statement::Static { declarations, .. } => {
                for declaration in declarations {
                    if let Some(value) = &declaration.value {
                        self.collect_direct_assignment(
                            &declaration.name,
                            AssignmentOp::Assign,
                            value,
                            source_file,
                            source_dir,
                        )?;
                    } else {
                        self.path_env.remove(&declaration.name);
                    }
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
                let before = self.path_env.clone();
                self.path_env = before.clone();
                self.collect_statements(then_body, source_file, source_dir)?;
                let then_env = self.path_env.clone();
                self.path_env = before;
                self.collect_statements(else_body, source_file, source_dir)?;
                let else_env = self.path_env.clone();
                self.path_env = merge_include_path_envs(&then_env, &else_env);
                Ok(())
            }
            Statement::While {
                condition, body, ..
            } => {
                self.collect_expr(condition, source_file, source_dir)?;
                let before = self.path_env.clone();
                self.collect_statements(body, source_file, source_dir)?;
                let body_env = self.path_env.clone();
                self.path_env = merge_include_path_envs(&before, &body_env);
                Ok(())
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
                let before_loop = self.path_env.clone();
                self.collect_statements(body, source_file, source_dir)?;
                self.collect_statements(updates, source_file, source_dir)?;
                let loop_env = self.path_env.clone();
                self.path_env = merge_include_path_envs(&before_loop, &loop_env);
                Ok(())
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
                    self.invalidate_assignment_target(key);
                }
                self.collect_assignment_target(value, source_file, source_dir)?;
                self.invalidate_assignment_target(value);
                let before_body = self.path_env.clone();
                self.collect_statements(body, source_file, source_dir)?;
                let body_env = self.path_env.clone();
                self.path_env = merge_include_path_envs(&before_body, &body_env);
                Ok(())
            }
            Statement::Switch {
                expression, cases, ..
            } => {
                self.collect_expr(expression, source_file, source_dir)?;
                let before_switch = self.path_env.clone();
                let mut case_envs = vec![before_switch.clone()];
                for case in cases {
                    self.path_env = before_switch.clone();
                    self.collect_switch_case(case, source_file, source_dir)?;
                    case_envs.push(self.path_env.clone());
                }
                self.path_env = merge_many_include_path_envs(&case_envs);
                Ok(())
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    self.collect_expr(value, source_file, source_dir)?;
                }
                Ok(())
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                self.collect_statements(body, source_file, source_dir)?;
                for catch in catches {
                    self.collect_catch(catch, source_file, source_dir)?;
                }
                self.collect_statements(finally_body, source_file, source_dir)
            }
            Statement::Increment { target, .. } => {
                self.collect_inc_dec_target(target, source_file, source_dir)
            }
            Statement::Empty { .. }
            | Statement::ClassDeclaration { .. }
            | Statement::FunctionDeclaration { .. }
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Label { .. }
            | Statement::Goto { .. }
            | Statement::InlineHtml { .. } => Ok(()),
            Statement::Global { names, .. } => {
                for name in names {
                    self.path_env.remove(name);
                }
                Ok(())
            }
        }
    }

    fn note_runtime_class_alias_statement(&mut self, statement: &Statement) {
        let Some((name, arguments, argument_names, argument_unpacks)) = (match statement {
            Statement::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                ..
            } => Some((name, arguments, argument_names, argument_unpacks)),
            Statement::Expression {
                expression:
                    Expr::Call {
                        name,
                        arguments,
                        argument_names,
                        argument_unpacks,
                        ..
                    },
                ..
            } => Some((name, arguments, argument_names, argument_unpacks)),
            _ => None,
        }) else {
            return;
        };
        if !name.eq_ignore_ascii_case("class_alias")
            || arguments.len() < 2
            || argument_names.iter().take(2).any(Option::is_some)
            || argument_unpacks.iter().take(2).any(|unpack| *unpack)
        {
            return;
        }
        let Some(target) = compile_time_class_alias_string(&arguments[0]) else {
            return;
        };
        let Some(alias) = compile_time_class_alias_string(&arguments[1]) else {
            return;
        };
        self.runtime_class_aliases.insert(
            normalize_runtime_class_alias_key(&alias),
            normalize_runtime_class_alias_target(&target),
        );
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

    fn collect_direct_assignment(
        &mut self,
        name: &str,
        op: AssignmentOp,
        value: &Expr,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        let assigned_paths = self.assigned_include_paths(name, op, value, source_file, source_dir);
        self.collect_expr(value, source_file, source_dir)?;
        self.apply_direct_assignment(name, assigned_paths);
        Ok(())
    }

    fn collect_assignment_expr(
        &mut self,
        target: &AssignmentTarget,
        op: AssignmentOp,
        value: &Expr,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        if let AssignmentTarget::Variable { name, .. } = target {
            return self.collect_direct_assignment(name, op, value, source_file, source_dir);
        }

        self.collect_assignment_target(target, source_file, source_dir)?;
        self.collect_expr(value, source_file, source_dir)?;
        self.invalidate_assignment_target(target);
        Ok(())
    }

    fn assigned_include_paths(
        &self,
        name: &str,
        op: AssignmentOp,
        value: &Expr,
        source_file: &str,
        source_dir: &str,
    ) -> Option<Vec<String>> {
        match op {
            AssignmentOp::Assign => {
                bounded_include_paths(value, source_file, source_dir, &self.path_env)
            }
            AssignmentOp::ConcatAssign => {
                let left_paths = self.path_env.get(name)?;
                let right_paths =
                    bounded_include_paths(value, source_file, source_dir, &self.path_env)?;
                concat_bounded_include_paths(left_paths, &right_paths)
            }
            AssignmentOp::CoalesceAssign => self.path_env.get(name).cloned(),
            AssignmentOp::AddAssign
            | AssignmentOp::SubtractAssign
            | AssignmentOp::MultiplyAssign
            | AssignmentOp::PowerAssign
            | AssignmentOp::DivideAssign
            | AssignmentOp::ModuloAssign
            | AssignmentOp::BitwiseAndAssign
            | AssignmentOp::BitwiseOrAssign
            | AssignmentOp::BitwiseXorAssign
            | AssignmentOp::ShiftLeftAssign
            | AssignmentOp::ShiftRightAssign => None,
        }
    }

    fn apply_direct_assignment(&mut self, name: &str, paths: Option<Vec<String>>) {
        match paths {
            Some(paths) if paths.len() <= MAX_BOUNDED_INCLUDE_CANDIDATES => {
                self.path_env.insert(name.to_string(), paths);
            }
            _ => {
                self.path_env.remove(name);
            }
        }
    }

    fn collect_inc_dec_target(
        &mut self,
        target: &IncDecTarget,
        source_file: &str,
        source_dir: &str,
    ) -> Result<()> {
        match target {
            IncDecTarget::Variable { name, .. } => {
                self.path_env.remove(name);
                Ok(())
            }
            IncDecTarget::DynamicVariable { name, .. } => {
                self.collect_expr(name, source_file, source_dir)?;
                self.path_env.clear();
                Ok(())
            }
            IncDecTarget::DynamicArrayDim {
                name, dimensions, ..
            } => {
                self.collect_expr(name, source_file, source_dir)?;
                for dimension in dimensions {
                    if let Some(dimension) = dimension {
                        self.collect_expr(dimension, source_file, source_dir)?;
                    }
                }
                self.path_env.clear();
                Ok(())
            }
            IncDecTarget::ArrayDim(target) => {
                self.collect_array_dim_target(target, source_file, source_dir)?;
                self.path_env.remove(&target.array);
                Ok(())
            }
            IncDecTarget::PropertyArrayDim {
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
            IncDecTarget::Property { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            IncDecTarget::StaticProperty { .. } => Ok(()),
        }
    }

    fn invalidate_assignment_target(&mut self, target: &AssignmentTarget) {
        match target {
            AssignmentTarget::Variable { name, .. } => {
                self.path_env.remove(name);
            }
            AssignmentTarget::DynamicVariable { .. } | AssignmentTarget::DynamicArrayDim { .. } => {
                self.path_env.clear();
            }
            AssignmentTarget::ArrayDim(target) => {
                self.path_env.remove(&target.array);
            }
            AssignmentTarget::List(target) => {
                for element in &target.elements {
                    match &element.target {
                        ListAssignmentElementTarget::Value(target) => {
                            self.invalidate_assignment_target(target);
                        }
                        ListAssignmentElementTarget::Reference(target) => {
                            self.invalidate_reference_target(target);
                        }
                    }
                }
            }
            AssignmentTarget::PropertyArrayDim { .. }
            | AssignmentTarget::StaticPropertyArrayDim { .. }
            | AssignmentTarget::DynamicStaticPropertyArrayDim { .. }
            | AssignmentTarget::ValueArrayDim { .. }
            | AssignmentTarget::Property { .. }
            | AssignmentTarget::DynamicProperty { .. }
            | AssignmentTarget::StaticProperty { .. }
            | AssignmentTarget::DynamicStaticProperty { .. } => {}
        }
    }

    fn invalidate_reference_target(&mut self, target: &ReferenceTarget) {
        match target {
            ReferenceTarget::Variable { name, .. } => {
                self.path_env.remove(name);
            }
            ReferenceTarget::ArrayDim(target) => {
                self.path_env.remove(&target.array);
            }
            ReferenceTarget::PropertyArrayDim { .. }
            | ReferenceTarget::Property { .. }
            | ReferenceTarget::DynamicProperty { .. } => {}
        }
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
                    candidates.clone(),
                );
                self.apply_include_path_env_effects(&candidates)?;
                Ok(())
            }
            Expr::AnonymousFunction(function) => self.collect_with_fresh_path_env(|collector| {
                collector.collect_statements(&function.body, source_file, source_dir)
            }),
            Expr::Assign {
                target, op, value, ..
            } => self.collect_assignment_expr(target, *op, value, source_file, source_dir),
            Expr::AssignRef { target, source, .. } => {
                self.collect_assignment_target(target, source_file, source_dir)?;
                self.collect_expr(source, source_file, source_dir)?;
                self.invalidate_assignment_target(target);
                Ok(())
            }
            Expr::IncDec { target, .. } => {
                self.collect_inc_dec_target(target, source_file, source_dir)
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
            Expr::DynamicMethodCall {
                receiver,
                name,
                arguments,
                ..
            } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_expr(name, source_file, source_dir)?;
                self.collect_exprs(arguments, source_file, source_dir)
            }
            Expr::PropertyFetch { receiver, .. } | Expr::NullsafePropertyFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::DynamicPropertyFetch { receiver, name, .. } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_expr(name, source_file, source_dir)
            }
            Expr::DynamicStaticPropertyFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::DynamicClassNameFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::InstanceOf { expr, target, .. } => {
                self.collect_expr(expr, source_file, source_dir)?;
                if let InstanceOfTarget::Expr(target) = target {
                    self.collect_expr(target, source_file, source_dir)?;
                }
                Ok(())
            }
            Expr::Array { elements, .. } => {
                for element in elements {
                    if let Some(key) = &element.key {
                        self.collect_expr(key, source_file, source_dir)?;
                    }
                    match &element.value {
                        ArrayElementValue::Hole(_) => {}
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
            | Expr::Grouped { expr: target, .. }
            | Expr::PipeValue { expr: target, .. } => {
                self.collect_expr(target, source_file, source_dir)
            }
            Expr::Yield { key, value, .. } => {
                if let Some(key) = key {
                    self.collect_expr(key, source_file, source_dir)?;
                }
                if let Some(value) = value {
                    self.collect_expr(value, source_file, source_dir)?;
                }
                Ok(())
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
            Expr::Match { subject, arms, .. } => {
                self.collect_expr(subject, source_file, source_dir)?;
                for arm in arms {
                    self.collect_exprs(&arm.conditions, source_file, source_dir)?;
                    self.collect_expr(&arm.value, source_file, source_dir)?;
                }
                Ok(())
            }
            Expr::String(_, _)
            | Expr::InterpolatedString(_, _)
            | Expr::ShellExec { .. }
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::Bool(_, _)
            | Expr::Null(_)
            | Expr::Variable(_, _)
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
            AssignmentTarget::StaticPropertyArrayDim { dimensions, .. } => {
                for dimension in dimensions {
                    if let Some(dimension) = dimension {
                        self.collect_expr(dimension, source_file, source_dir)?;
                    }
                }
                Ok(())
            }
            AssignmentTarget::DynamicStaticPropertyArrayDim {
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
            AssignmentTarget::ValueArrayDim {
                array, dimensions, ..
            } => {
                self.collect_expr(array, source_file, source_dir)?;
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
            AssignmentTarget::DynamicProperty { receiver, name, .. } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_expr(name, source_file, source_dir)
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
            AssignmentTarget::DynamicStaticProperty { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
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
            ReferenceTarget::DynamicProperty { receiver, name, .. } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_expr(name, source_file, source_dir)
            }
            ReferenceTarget::PropertyArrayDim {
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
        let include_paths =
            bounded_include_paths(path, source_file, source_dir, &self.path_env).ok_or_else(
                || {
            Diagnostic::new(
                "dynamic include paths are unsupported; use a compile-time string path or bounded conditional of compile-time string paths",
                Some(path.span()),
            )
                },
            )?;
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

        let source_bytes = fs::read(&canonical_path).map_err(|error| {
            Diagnostic::new(
                format!(
                    "failed to read included file {}: {error}",
                    canonical_path.display()
                ),
                Some(span),
            )
        })?;
        let source = decode_php_source_bytes(&source_bytes);
        let program = parse_with_runtime_class_aliases(&source, &self.runtime_class_aliases)?;

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

    fn apply_include_path_env_effects(&mut self, candidates: &[usize]) -> Result<()> {
        if candidates.is_empty() {
            return Ok(());
        }

        let before = self.path_env.clone();
        let mut candidate_envs = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let include = self.sources[*candidate].clone();
            self.path_env = before.clone();
            self.collect_top_level_statements(
                &include.program.statements,
                &include.source_file,
                &include.source_dir,
            )?;
            candidate_envs.push(self.path_env.clone());
        }
        self.path_env = merge_many_include_path_envs(&candidate_envs);
        Ok(())
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

fn bounded_include_paths(
    expr: &Expr,
    source_file: &str,
    source_dir: &str,
    path_env: &IncludePathEnv,
) -> Option<Vec<String>> {
    match expr {
        Expr::String(value, _) => Some(vec![value.clone()]),
        Expr::ShellExec { .. } => None,
        Expr::Variable(name, _) => path_env.get(name).cloned(),
        Expr::MagicConstant(MagicConstantKind::File, _) => Some(vec![source_file.to_string()]),
        Expr::MagicConstant(MagicConstantKind::Dir, _) => Some(vec![source_dir.to_string()]),
        Expr::Constant(name, _) if name == "DIRECTORY_SEPARATOR" => {
            Some(vec![std::path::MAIN_SEPARATOR.to_string()])
        }
        Expr::Constant(name, _) if name == "PATH_SEPARATOR" => {
            Some(vec![if cfg!(windows) { ";" } else { ":" }.to_string()])
        }
        Expr::Call {
            name, arguments, ..
        } if name.eq_ignore_ascii_case("dirname")
            && (arguments.len() == 1 || arguments.len() == 2) =>
        {
            let paths = bounded_include_paths(&arguments[0], source_file, source_dir, path_env)?;
            let levels = if arguments.len() == 2 {
                match &arguments[1] {
                    Expr::Int(levels, _) if *levels >= 1 => usize::try_from(*levels).ok()?,
                    _ => return None,
                }
            } else {
                1
            };
            let mut resolved = Vec::new();
            for path in paths {
                push_unique_string(&mut resolved, compile_time_dirname(&path, levels));
            }
            Some(resolved)
        }
        Expr::Call {
            name, arguments, ..
        } if name.eq_ignore_ascii_case("realpath") && arguments.len() == 1 => {
            let paths = bounded_include_paths(&arguments[0], source_file, source_dir, path_env)?;
            let mut resolved = Vec::new();
            for path in paths {
                let canonical = fs::canonicalize(PathBuf::from(path)).ok()?;
                push_unique_string(&mut resolved, canonical.to_string_lossy().into_owned());
            }
            Some(resolved)
        }
        Expr::Binary {
            op: BinaryOp::Concat,
            left,
            right,
            ..
        } => {
            let left_paths = bounded_include_paths(left, source_file, source_dir, path_env)?;
            let right_paths = bounded_include_paths(right, source_file, source_dir, path_env)?;
            concat_bounded_include_paths(&left_paths, &right_paths)
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            let mut paths = Vec::new();
            let true_expr = if_true.as_deref().unwrap_or(condition);
            for path in bounded_include_paths(true_expr, source_file, source_dir, path_env)? {
                push_unique_string(&mut paths, path);
            }
            for path in bounded_include_paths(if_false, source_file, source_dir, path_env)? {
                push_unique_string(&mut paths, path);
            }
            if paths.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                return None;
            }
            Some(paths)
        }
        Expr::Match { arms, .. } => {
            let mut paths = Vec::new();
            for arm in arms {
                for path in bounded_include_paths(&arm.value, source_file, source_dir, path_env)? {
                    push_unique_string(&mut paths, path);
                }
                if paths.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                    return None;
                }
            }
            Some(paths)
        }
        Expr::Grouped { expr, .. } => {
            bounded_include_paths(expr, source_file, source_dir, path_env)
        }
        _ => None,
    }
}

fn concat_bounded_include_paths(
    left_paths: &[String],
    right_paths: &[String],
) -> Option<Vec<String>> {
    if left_paths.len().saturating_mul(right_paths.len()) > MAX_BOUNDED_INCLUDE_CANDIDATES {
        return None;
    }
    let mut paths = Vec::new();
    for left_path in left_paths {
        for right_path in right_paths {
            let mut path = left_path.clone();
            path.push_str(right_path);
            push_unique_string(&mut paths, path);
        }
    }
    Some(paths)
}

fn merge_many_include_path_envs(envs: &[IncludePathEnv]) -> IncludePathEnv {
    let Some((first, rest)) = envs.split_first() else {
        return IncludePathEnv::new();
    };
    let mut merged = first.clone();
    for env in rest {
        merged = merge_include_path_envs(&merged, env);
    }
    merged
}

fn merge_include_path_envs(left: &IncludePathEnv, right: &IncludePathEnv) -> IncludePathEnv {
    let mut merged = IncludePathEnv::new();
    for (name, left_paths) in left {
        let Some(right_paths) = right.get(name) else {
            continue;
        };
        let mut paths = left_paths.clone();
        for path in right_paths {
            push_unique_string(&mut paths, path.clone());
            if paths.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                break;
            }
        }
        if paths.len() <= MAX_BOUNDED_INCLUDE_CANDIDATES {
            merged.insert(name.clone(), paths);
        }
    }
    merged
}

fn compile_time_dirname(path: &str, levels: usize) -> String {
    let mut path = PathBuf::from(path);
    for _ in 0..levels {
        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        }
    }
    path.to_string_lossy().into_owned()
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

fn normalize_runtime_class_alias_key(name: &str) -> String {
    normalize_runtime_class_alias_target(name).to_ascii_lowercase()
}

fn normalize_runtime_class_alias_target(name: &str) -> String {
    name.trim_start_matches('\\').to_string()
}

fn compile_time_class_alias_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::String(value, _) => Some(value.clone()),
        Expr::ClassConstantFetch {
            class_name, name, ..
        } if name.eq_ignore_ascii_case("class") => Some(class_name.clone()),
        Expr::Binary {
            op: BinaryOp::Concat,
            left,
            right,
            ..
        } => {
            let mut value = compile_time_class_alias_string(left)?;
            value.push_str(&compile_time_class_alias_string(right)?);
            Some(value)
        }
        Expr::Grouped { expr, .. } => compile_time_class_alias_string(expr),
        _ => None,
    }
}
