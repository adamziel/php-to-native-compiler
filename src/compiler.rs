use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};

use crate::ast::{
    ArrayDimTarget, ArrayElementValue, AssignmentOp, AssignmentTarget, BinaryOp, CatchClause,
    ClassDecl, Expr, GlobalTarget, IncDecTarget, InstanceOfTarget, ListAssignmentElementTarget,
    MagicConstantKind, Program, ReferenceTarget, Statement, StringPart, SwitchCase, TraitDecl,
    UnaryOp, UnsetTarget,
};
use crate::backend::{compile_c, emit_c};
use crate::diagnostic::{Diagnostic, DiagnosticKind, Result};
use crate::ir::{
    lower_with_source_and_includes, IncludeParseError, IncludeResolutionMap, IncludeSource,
};
use crate::lexer::{decode_php_source_bytes, decode_php_source_bytes_with_encoding};
use crate::parser::{
    parse_for_include_collection, parse_include_with_runtime_class_aliases_and_symbols,
    parse_with_runtime_class_aliases_and_symbols,
};

const MAX_BOUNDED_INCLUDE_CANDIDATES: usize = 32;

unsafe extern "C" {
    fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> *mut c_void;
    fn iconv(
        cd: *mut c_void,
        inbuf: *mut *mut c_char,
        inbytesleft: *mut usize,
        outbuf: *mut *mut c_char,
        outbytesleft: *mut usize,
    ) -> usize;
    fn iconv_close(cd: *mut c_void) -> c_int;
}

type IncludePathEnv = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IncludeSourceKey {
    canonical_path: PathBuf,
    transform: Option<IncludeSourceTransform>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IncludeSourceTransform {
    PhpFilter(Vec<PhpFilterReadFilter>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PhpFilterReadFilter {
    StringToLower,
    StringToUpper,
    StringRot13,
}

struct ResolvedIncludeCandidate {
    resource_path: PathBuf,
    path_aliases: Vec<String>,
    transform: Option<IncludeSourceTransform>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IncludePathTemplatePart {
    Static(String),
    Dynamic,
}

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub emit_c: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CompileSourceOptions {
    pub zend_multibyte: bool,
    pub script_encoding: Option<String>,
    pub internal_encoding: Option<String>,
    pub encoding_translation: bool,
    pub force_internal_function_dispatch: bool,
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub binary: PathBuf,
    pub c_source: Option<PathBuf>,
}

pub fn compile_file(input: &Path, output: &Path, options: CompileOptions) -> Result<CompileOutput> {
    compile_file_inner(input, output, options, &[])
}

pub fn compile_file_with_preloads(
    input: &Path,
    output: &Path,
    options: CompileOptions,
    preload_files: &[PathBuf],
) -> Result<CompileOutput> {
    compile_file_inner(input, output, options, preload_files)
}

fn compile_file_inner(
    input: &Path,
    output: &Path,
    options: CompileOptions,
    preload_files: &[PathBuf],
) -> Result<CompileOutput> {
    compile_file_inner_with_source_options(
        input,
        output,
        options,
        preload_files,
        &CompileSourceOptions::default(),
    )
}

pub fn compile_file_with_preloads_and_source_options(
    input: &Path,
    output: &Path,
    options: CompileOptions,
    preload_files: &[PathBuf],
    source_options: CompileSourceOptions,
) -> Result<CompileOutput> {
    compile_file_inner_with_source_options(input, output, options, preload_files, &source_options)
}

fn compile_file_inner_with_source_options(
    input: &Path,
    output: &Path,
    options: CompileOptions,
    preload_files: &[PathBuf],
    source_options: &CompileSourceOptions,
) -> Result<CompileOutput> {
    let source_bytes = fs::read(input).map_err(|error| {
        Diagnostic::new(format!("failed to read {}: {error}", input.display()), None)
    })?;
    let source = decode_compiler_source_bytes(&source_bytes, source_options)?;
    let include_program = parse_for_include_collection(&source, &HashMap::new())?;
    let source_file = input.to_string_lossy().into_owned();
    let source_dir = input
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut includes = IncludeCollector::new(
        include_program.classes.clone(),
        include_program.traits.clone(),
        source_options.clone(),
    );
    let preload_include_indices = includes.collect_preload_files(preload_files, &source_dir)?;
    includes.collect_program(&include_program, &source_file, &source_dir)?;
    includes.finalize_sources()?;
    let (included_classes, included_traits) = includes.validation_symbols(None);
    let program = parse_with_runtime_class_aliases_and_symbols(
        &source,
        &includes.runtime_class_aliases,
        &included_classes,
        &included_traits,
    )?;
    let include_sources = includes.sources;
    let include_resolutions = includes.resolutions;
    let mut module = lower_with_source_and_includes(
        &program,
        source_file,
        source_dir,
        source_bytes,
        include_sources,
        preload_include_indices,
        &include_resolutions,
    );
    module.runtime_requirements.internal_function_dispatch |=
        source_options.force_internal_function_dispatch;
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

fn decode_compiler_source_bytes(bytes: &[u8], options: &CompileSourceOptions) -> Result<String> {
    if !options.zend_multibyte {
        return Ok(decode_php_source_bytes(bytes));
    }

    let source_encoding = options
        .script_encoding
        .as_deref()
        .filter(|encoding| is_usable_source_encoding(encoding))
        .map(str::to_string)
        .or_else(|| sniff_declared_source_encoding(bytes))
        .or_else(|| {
            options
                .internal_encoding
                .as_deref()
                .filter(|encoding| is_usable_source_encoding(encoding))
                .map(str::to_string)
        });

    let mut decoded_bytes = Cow::Borrowed(bytes);
    let mut decode_encoding = source_encoding.as_deref();
    if options.encoding_translation {
        if let (Some(from), Some(to)) = (
            source_encoding.as_deref(),
            options
                .internal_encoding
                .as_deref()
                .filter(|encoding| is_usable_source_encoding(encoding)),
        ) {
            if !encoding_names_equivalent(from, to) && !source_has_utf16_bom(bytes) {
                decoded_bytes = Cow::Owned(iconv_convert_bytes(bytes, from, to)?);
                decode_encoding = Some(to);
            }
        }
    }

    Ok(decode_php_source_bytes_with_encoding(
        &decoded_bytes,
        decode_encoding,
    ))
}

fn empty_include_program() -> Program {
    Program {
        classes: Vec::new(),
        traits: Vec::new(),
        functions: Vec::new(),
        statements: Vec::new(),
        compile_warnings: Vec::new(),
        strict_types: false,
        ticks: false,
    }
}

fn is_usable_source_encoding(encoding: &str) -> bool {
    let trimmed = encoding.trim();
    !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("pass")
}

fn source_has_utf16_bom(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff])
}

fn encoding_names_equivalent(left: &str, right: &str) -> bool {
    canonical_encoding_key(left) == canonical_encoding_key(right)
}

fn canonical_encoding_key(value: &str) -> Vec<u8> {
    value
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric())
        .map(|byte| byte.to_ascii_lowercase())
        .collect()
}

fn iconv_convert_bytes(bytes: &[u8], from: &str, to: &str) -> Result<Vec<u8>> {
    let from = CString::new(from)
        .map_err(|_| Diagnostic::new("source encoding name contains an interior NUL byte", None))?;
    let to = CString::new(to)
        .map_err(|_| Diagnostic::new("target encoding name contains an interior NUL byte", None))?;
    let descriptor = unsafe { iconv_open(to.as_ptr(), from.as_ptr()) };
    if descriptor == usize::MAX as *mut c_void {
        return Err(Diagnostic::new(
            "failed to initialize source encoding conversion",
            None,
        ));
    }

    let result = iconv_convert_bytes_with_descriptor(descriptor, bytes);
    unsafe {
        iconv_close(descriptor);
    }
    result
}

fn iconv_convert_bytes_with_descriptor(descriptor: *mut c_void, bytes: &[u8]) -> Result<Vec<u8>> {
    let mut input_ptr = bytes.as_ptr() as *mut c_char;
    let mut input_left = bytes.len();
    let mut output = vec![0u8; bytes.len().saturating_mul(4).max(64)];
    let mut output_used = 0usize;

    loop {
        let mut output_ptr = unsafe { output.as_mut_ptr().add(output_used) as *mut c_char };
        let mut output_left = output.len() - output_used;
        let converted = unsafe {
            iconv(
                descriptor,
                &mut input_ptr,
                &mut input_left,
                &mut output_ptr,
                &mut output_left,
            )
        };
        output_used = output.len() - output_left;
        if converted != usize::MAX {
            output.truncate(output_used);
            return Ok(output);
        }
        if output_left == 0 {
            output.resize(output.len().saturating_mul(2).max(64), 0);
            continue;
        }
        return Err(Diagnostic::new(
            "failed to convert PHP source encoding",
            None,
        ));
    }
}

fn sniff_declared_source_encoding(bytes: &[u8]) -> Option<String> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    if source_has_utf16_bom(bytes) {
        return None;
    }
    let mut cursor = 0;
    if bytes.starts_with(b"#!") {
        cursor = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|index| index + 1)
            .unwrap_or(bytes.len());
    }
    skip_ascii_whitespace(bytes, &mut cursor);
    if !ascii_word_at(bytes, cursor, b"<?php") {
        return None;
    }
    cursor += 5;
    skip_ascii_whitespace(bytes, &mut cursor);
    if !ascii_word_at(bytes, cursor, b"declare") {
        return None;
    }
    cursor += "declare".len();
    skip_ascii_whitespace(bytes, &mut cursor);
    if bytes.get(cursor).copied() != Some(b'(') {
        return None;
    }
    cursor += 1;
    skip_ascii_whitespace(bytes, &mut cursor);
    if !ascii_word_at(bytes, cursor, b"encoding") {
        return None;
    }
    cursor += "encoding".len();
    skip_ascii_whitespace(bytes, &mut cursor);
    if bytes.get(cursor).copied() != Some(b'=') {
        return None;
    }
    cursor += 1;
    skip_ascii_whitespace(bytes, &mut cursor);
    let quote = bytes.get(cursor).copied()?;
    if quote == b'\'' || quote == b'"' {
        cursor += 1;
        let start = cursor;
        while let Some(byte) = bytes.get(cursor).copied() {
            if byte == quote {
                return std::str::from_utf8(&bytes[start..cursor])
                    .ok()
                    .map(str::to_string);
            }
            cursor += 1;
        }
        return None;
    }
    let start = cursor;
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'))
    {
        cursor += 1;
    }
    (cursor > start)
        .then(|| {
            std::str::from_utf8(&bytes[start..cursor])
                .ok()
                .map(str::to_string)
        })
        .flatten()
}

fn ascii_word_at(bytes: &[u8], cursor: usize, word: &[u8]) -> bool {
    bytes
        .get(cursor..cursor + word.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(word))
}

fn skip_ascii_whitespace(bytes: &[u8], cursor: &mut usize) {
    while bytes
        .get(*cursor)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        *cursor += 1;
    }
}

struct IncludeCollector {
    sources: Vec<IncludeSource>,
    source_transforms: Vec<Option<IncludeSourceTransform>>,
    by_source: HashMap<IncludeSourceKey, usize>,
    resolutions: IncludeResolutionMap,
    path_env: IncludePathEnv,
    include_effect_stack: Vec<usize>,
    runtime_class_aliases: HashMap<String, String>,
    root_classes: Vec<ClassDecl>,
    root_traits: Vec<TraitDecl>,
    source_options: CompileSourceOptions,
}

impl IncludeCollector {
    fn new(
        root_classes: Vec<ClassDecl>,
        root_traits: Vec<TraitDecl>,
        source_options: CompileSourceOptions,
    ) -> Self {
        Self {
            sources: Vec::new(),
            source_transforms: Vec::new(),
            by_source: HashMap::new(),
            resolutions: IncludeResolutionMap::new(),
            path_env: IncludePathEnv::new(),
            include_effect_stack: Vec::new(),
            runtime_class_aliases: HashMap::new(),
            root_classes,
            root_traits,
            source_options,
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

    fn collect_preload_files(
        &mut self,
        preload_files: &[PathBuf],
        source_dir: &str,
    ) -> Result<Vec<usize>> {
        let mut indices = Vec::new();
        for preload_file in preload_files {
            if preload_file.as_os_str().is_empty() {
                continue;
            }
            let resolved_path = if preload_file.is_absolute() {
                preload_file.clone()
            } else {
                Path::new(source_dir).join(preload_file)
            };
            let preload_path = resolved_path.to_string_lossy().into_owned();
            let Some(index) = self.resolve_include_candidate(&preload_path, source_dir)? else {
                return Err(Diagnostic::new(
                    format!("failed to read preload file {}", resolved_path.display()),
                    None,
                ));
            };
            if !indices.contains(&index) {
                indices.push(index);
            }
        }
        Ok(indices)
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
            Statement::Exit {
                value: Some(value), ..
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
                        UnsetTarget::ValueArrayDim {
                            array, dimensions, ..
                        } => {
                            self.collect_expr(array, source_file, source_dir)?;
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
                        UnsetTarget::StaticPropertyArrayDim { dimensions, .. } => {
                            for dimension in dimensions {
                                self.collect_expr(dimension, source_file, source_dir)?;
                            }
                        }
                        UnsetTarget::DynamicStaticPropertyArrayDim {
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
                        UnsetTarget::DynamicProperty { receiver, name, .. } => {
                            self.collect_expr(receiver, source_file, source_dir)?;
                            self.collect_expr(name, source_file, source_dir)?;
                        }
                        UnsetTarget::StaticProperty { .. } => {}
                        UnsetTarget::DynamicStaticPropertyName { name, .. } => {
                            self.collect_expr(name, source_file, source_dir)?;
                        }
                        UnsetTarget::DynamicVariable { name, .. } => {
                            self.collect_expr(name, source_file, source_dir)?;
                            self.path_env.clear();
                        }
                    }
                }
                Ok(())
            }
            Statement::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } if name.eq_ignore_ascii_case("opcache_compile_file")
                && arguments.len() == 1
                && argument_names.iter().all(Option::is_none)
                && argument_unpacks.iter().all(|unpack| !*unpack) =>
            {
                let candidates =
                    self.resolve_include(&arguments[0], *span, source_file, source_dir)?;
                self.resolutions.insert(
                    (source_file.to_string(), span.byte_start, span.byte_end),
                    candidates,
                );
                self.collect_exprs(arguments, source_file, source_dir)
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
                if let Some(value) = compile_time_condition_truth(condition) {
                    if value {
                        self.collect_statements(then_body, source_file, source_dir)?;
                    } else {
                        self.collect_statements(else_body, source_file, source_dir)?;
                    }
                    return Ok(());
                }
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
                conditions,
                updates,
                body,
                ..
            } => {
                self.collect_statements(initializers, source_file, source_dir)?;
                for condition in conditions {
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
            | Statement::TraitDeclaration { .. }
            | Statement::FunctionDeclaration { .. }
            | Statement::Exit { value: None, .. }
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Label { .. }
            | Statement::Goto { .. }
            | Statement::InlineHtml { .. } => Ok(()),
            Statement::Global { targets, .. } => {
                for target in targets {
                    match target {
                        GlobalTarget::Variable { name, .. } => {
                            self.path_env.remove(name);
                        }
                        GlobalTarget::DynamicVariable { name, .. } => {
                            self.collect_expr(name, source_file, source_dir)?;
                        }
                    }
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
            IncDecTarget::ValueArrayDim {
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
            IncDecTarget::StaticPropertyArrayDim { dimensions, .. } => {
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
            IncDecTarget::DynamicProperty { receiver, name, .. } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_expr(name, source_file, source_dir)
            }
            IncDecTarget::StaticProperty { .. } => Ok(()),
            IncDecTarget::DynamicStaticPropertyName { name, .. } => {
                self.collect_expr(name, source_file, source_dir)
            }
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
            | AssignmentTarget::DynamicPropertyArrayDim { .. }
            | AssignmentTarget::StaticPropertyArrayDim { .. }
            | AssignmentTarget::DynamicStaticPropertyArrayDim { .. }
            | AssignmentTarget::ValueArrayDim { .. }
            | AssignmentTarget::Property { .. }
            | AssignmentTarget::DynamicProperty { .. }
            | AssignmentTarget::StaticProperty { .. }
            | AssignmentTarget::DynamicStaticProperty { .. }
            | AssignmentTarget::DynamicStaticPropertyName { .. } => {}
        }
    }

    fn invalidate_reference_target(&mut self, target: &ReferenceTarget) {
        match target {
            ReferenceTarget::Variable { name, .. } => {
                self.path_env.remove(name);
            }
            ReferenceTarget::DynamicVariable { .. } => {
                self.path_env.clear();
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
            Expr::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } if name.eq_ignore_ascii_case("opcache_compile_file")
                && arguments.len() == 1
                && argument_names.iter().all(Option::is_none)
                && argument_unpacks.iter().all(|unpack| !*unpack) =>
            {
                let candidates =
                    self.resolve_include(&arguments[0], *span, source_file, source_dir)?;
                self.resolutions.insert(
                    (source_file.to_string(), span.byte_start, span.byte_end),
                    candidates,
                );
                self.collect_exprs(arguments, source_file, source_dir)
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
            Expr::Call { arguments, .. }
            | Expr::ParentPropertyHookCall { arguments, .. }
            | Expr::NewObject { arguments, .. } => {
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
            Expr::DynamicStaticPropertyNameFetch { name, .. } => {
                self.collect_expr(name, source_file, source_dir)
            }
            Expr::DynamicStaticPropertyFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::DynamicClassNameFetch { receiver, .. } => {
                self.collect_expr(receiver, source_file, source_dir)
            }
            Expr::DynamicClassConstantFetch { receiver, name, .. } => {
                if let Some(receiver) = receiver {
                    self.collect_expr(receiver, source_file, source_dir)?;
                }
                self.collect_expr(name, source_file, source_dir)
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
            Expr::YieldFrom { expr, .. } => self.collect_expr(expr, source_file, source_dir),
            Expr::Yield { key, value, .. } => {
                if let Some(key) = key {
                    self.collect_expr(key, source_file, source_dir)?;
                }
                if let Some(value) = value {
                    self.collect_expr(value, source_file, source_dir)?;
                }
                Ok(())
            }
            Expr::Exit { value, .. } => {
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
            AssignmentTarget::DynamicPropertyArrayDim {
                receiver,
                name,
                dimensions,
                ..
            } => {
                self.collect_expr(receiver, source_file, source_dir)?;
                self.collect_expr(name, source_file, source_dir)?;
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
            AssignmentTarget::DynamicStaticPropertyName { name, .. } => {
                self.collect_expr(name, source_file, source_dir)
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
            ReferenceTarget::DynamicVariable { name, .. } => {
                self.collect_expr(name, source_file, source_dir)
            }
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
        _span: crate::diagnostic::SourceSpan,
        source_file: &str,
        source_dir: &str,
    ) -> Result<Vec<usize>> {
        let Some(include_paths) =
            bounded_include_paths(path, source_file, source_dir, &self.path_env)
        else {
            return Ok(Vec::new());
        };
        let mut candidates = Vec::new();
        for include_path in include_paths {
            let Some(index) = self.resolve_include_candidate(&include_path, source_dir)? else {
                continue;
            };
            if !candidates.contains(&index) {
                candidates.push(index);
            }
        }
        Ok(candidates)
    }

    fn resolve_include_candidate(
        &mut self,
        include_path: &str,
        source_dir: &str,
    ) -> Result<Option<usize>> {
        let resolved = resolve_include_candidate_path(include_path, source_dir);
        let canonical_path = match fs::canonicalize(&resolved.resource_path) {
            Ok(path) => path,
            Err(_) => return Ok(None),
        };
        let path_aliases = if resolved.transform.is_none() {
            include_path_aliases(&resolved.resource_path, &canonical_path)
        } else {
            resolved.path_aliases
        };
        let key = IncludeSourceKey {
            canonical_path: canonical_path.clone(),
            transform: resolved.transform.clone(),
        };
        if let Some(index) = self.by_source.get(&key).copied() {
            self.add_path_aliases(index, path_aliases);
            return Ok(Some(index));
        }

        let mut source_bytes = match fs::read(&canonical_path) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(None),
        };
        apply_include_source_transform(&mut source_bytes, resolved.transform.as_ref());
        let source = decode_compiler_source_bytes(&source_bytes, &self.source_options)?;
        let (program, parse_error) =
            match parse_for_include_collection(&source, &self.runtime_class_aliases) {
                Ok(program) => (program, None),
                Err(error) if error.kind == DiagnosticKind::ParseError => {
                    let line = error.span.map(|span| span.line).unwrap_or(1);
                    (
                        empty_include_program(),
                        Some(IncludeParseError {
                            message: error.message,
                            line,
                        }),
                    )
                }
                Err(error) => return Err(error),
            };

        let index = self.sources.len();
        self.by_source.insert(key, index);
        let source_file = canonical_path.to_string_lossy().into_owned();
        let source_dir = canonical_path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.sources.push(IncludeSource {
            source_file: source_file.clone(),
            source_dir: source_dir.clone(),
            source_bytes,
            path_aliases,
            parse_error,
            program: program.clone(),
        });
        self.source_transforms.push(resolved.transform);
        self.collect_program(&program, &source_file, &source_dir)?;
        Ok(Some(index))
    }

    fn finalize_sources(&mut self) -> Result<()> {
        for index in 0..self.sources.len() {
            if self.sources[index].parse_error.is_some() {
                continue;
            }
            let source_file = self.sources[index].source_file.clone();
            let mut source_bytes = fs::read(&source_file).map_err(|error| {
                Diagnostic::new(format!("failed to read {source_file}: {error}"), None)
            })?;
            apply_include_source_transform(
                &mut source_bytes,
                self.source_transforms[index].as_ref(),
            );
            let source = decode_compiler_source_bytes(&source_bytes, &self.source_options)?;
            let (included_classes, included_traits) = self.include_validation_symbols(Some(index));
            self.sources[index].source_bytes = source_bytes;
            self.sources[index].program = parse_include_with_runtime_class_aliases_and_symbols(
                &source,
                &self.runtime_class_aliases,
                &included_classes,
                &included_traits,
            )?;
        }
        Ok(())
    }

    fn validation_symbols(
        &self,
        excluded_source: Option<usize>,
    ) -> (Vec<ClassDecl>, Vec<TraitDecl>) {
        let mut classes = Vec::new();
        let mut traits = Vec::new();
        for (index, source) in self.sources.iter().enumerate() {
            if excluded_source == Some(index) {
                continue;
            }
            classes.extend(source.program.classes.iter().cloned());
            traits.extend(source.program.traits.iter().cloned());
        }
        (classes, traits)
    }

    fn include_validation_symbols(
        &self,
        excluded_source: Option<usize>,
    ) -> (Vec<ClassDecl>, Vec<TraitDecl>) {
        let (mut classes, mut traits) = self.validation_symbols(excluded_source);
        classes.extend(self.root_classes.iter().cloned());
        traits.extend(self.root_traits.iter().cloned());
        (classes, traits)
    }

    fn apply_include_path_env_effects(&mut self, candidates: &[usize]) -> Result<()> {
        if candidates.is_empty() {
            return Ok(());
        }

        let before = self.path_env.clone();
        let mut candidate_envs = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            if self.include_effect_stack.contains(candidate) {
                candidate_envs.push(before.clone());
                continue;
            }
            let include = self.sources[*candidate].clone();
            self.path_env = before.clone();
            self.include_effect_stack.push(*candidate);
            let result = self.collect_top_level_statements(
                &include.program.statements,
                &include.source_file,
                &include.source_dir,
            );
            self.include_effect_stack.pop();
            result?;
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
    if let Some(paths) = bounded_static_include_paths(expr, source_file, source_dir, path_env) {
        return Some(paths);
    }
    bounded_dynamic_include_paths(expr, source_file, source_dir, path_env)
}

fn bounded_static_include_paths(
    expr: &Expr,
    source_file: &str,
    source_dir: &str,
    path_env: &IncludePathEnv,
) -> Option<Vec<String>> {
    match expr {
        Expr::String(value, _) => Some(vec![value.clone()]),
        Expr::InterpolatedString(parts, _) => bounded_interpolated_string_paths(parts, path_env),
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
            let paths =
                bounded_static_include_paths(&arguments[0], source_file, source_dir, path_env)?;
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
            let paths =
                bounded_static_include_paths(&arguments[0], source_file, source_dir, path_env)?;
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
            let left_paths = bounded_static_include_paths(left, source_file, source_dir, path_env)?;
            let right_paths =
                bounded_static_include_paths(right, source_file, source_dir, path_env)?;
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
            for path in bounded_static_include_paths(true_expr, source_file, source_dir, path_env)?
            {
                push_unique_string(&mut paths, path);
            }
            for path in bounded_static_include_paths(if_false, source_file, source_dir, path_env)? {
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
                for path in
                    bounded_static_include_paths(&arm.value, source_file, source_dir, path_env)?
                {
                    push_unique_string(&mut paths, path);
                }
                if paths.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                    return None;
                }
            }
            Some(paths)
        }
        Expr::Grouped { expr, .. } => {
            bounded_static_include_paths(expr, source_file, source_dir, path_env)
        }
        _ => None,
    }
}

fn compile_time_condition_truth(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Bool(value, _) => Some(*value),
        Expr::Null(_) => Some(false),
        Expr::Int(value, _) => Some(*value != 0),
        Expr::Float(value, _) => Some(*value != 0.0),
        Expr::String(value, _) => Some(!value.is_empty() && value != "0"),
        Expr::Grouped { expr, .. } => compile_time_condition_truth(expr),
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
            ..
        } => compile_time_condition_truth(expr).map(|value| !value),
        Expr::Call {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            ..
        } if name.eq_ignore_ascii_case("class_exists")
            && arguments.len() <= 2
            && argument_names.iter().all(Option::is_none)
            && argument_unpacks.iter().all(|unpack| !unpack) =>
        {
            let class_name = compile_time_string_literal(arguments.first()?)?;
            if include_collection_known_internal_class(class_name) {
                Some(true)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn compile_time_string_literal(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::String(value, _) => Some(value),
        Expr::Grouped { expr, .. } => compile_time_string_literal(expr),
        _ => None,
    }
}

fn include_collection_known_internal_class(class_name: &str) -> bool {
    let name = class_name.trim_start_matches('\\');
    [
        "AppendIterator",
        "ArrayIterator",
        "ArrayObject",
        "Attribute",
        "CallbackFilterIterator",
        "Closure",
        "DateInterval",
        "DatePeriod",
        "DateTime",
        "DateTimeImmutable",
        "DateTimeZone",
        "DelayedTargetValidation",
        "Deprecated",
        "DirectoryIterator",
        "EmptyIterator",
        "FilesystemIterator",
        "FilterIterator",
        "Generator",
        "GlobIterator",
        "InfiniteIterator",
        "InternalIterator",
        "IteratorIterator",
        "LimitIterator",
        "MultipleIterator",
        "NoDiscard",
        "NoRewindIterator",
        "RegexIterator",
        "RecursiveArrayIterator",
        "RecursiveCallbackFilterIterator",
        "RecursiveIteratorIterator",
        "ReflectionClass",
        "ReflectionConstant",
        "ReflectionEnum",
        "ReflectionEnumBackedCase",
        "ReflectionEnumUnitCase",
        "ReflectionExtension",
        "ReflectionFiber",
        "ReflectionFunction",
        "ReflectionMethod",
        "ReflectionObject",
        "ReflectionParameter",
        "ReflectionProperty",
        "ReturnTypeWillChange",
        "SensitiveParameter",
        "SensitiveParameterValue",
        "SplDoublyLinkedList",
        "SplFileInfo",
        "SplFileObject",
        "SplTempFileObject",
        "SplFixedArray",
        "SplHeap",
        "SplMaxHeap",
        "SplMinHeap",
        "SplObjectStorage",
        "SplPriorityQueue",
        "SplQueue",
        "SplStack",
        "stdClass",
    ]
    .iter()
    .any(|known| known.eq_ignore_ascii_case(name))
}

fn bounded_dynamic_include_paths(
    expr: &Expr,
    source_file: &str,
    source_dir: &str,
    path_env: &IncludePathEnv,
) -> Option<Vec<String>> {
    let templates = bounded_include_path_templates(expr, source_file, source_dir, path_env)?;
    let mut paths = Vec::new();
    let mut saw_dynamic = false;
    for template in templates {
        if !template
            .iter()
            .any(|part| matches!(part, IncludePathTemplatePart::Dynamic))
        {
            continue;
        }
        saw_dynamic = true;
        for path in expand_dynamic_include_template(&template, source_dir)? {
            push_unique_string(&mut paths, path);
            if paths.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                return None;
            }
        }
    }
    if saw_dynamic && !paths.is_empty() {
        Some(paths)
    } else {
        None
    }
}

fn bounded_include_path_templates(
    expr: &Expr,
    source_file: &str,
    source_dir: &str,
    path_env: &IncludePathEnv,
) -> Option<Vec<Vec<IncludePathTemplatePart>>> {
    match expr {
        Expr::String(value, _) => Some(vec![vec![IncludePathTemplatePart::Static(value.clone())]]),
        Expr::InterpolatedString(parts, _) => {
            bounded_interpolated_string_templates(parts, path_env)
        }
        Expr::Variable(name, _) => {
            if let Some(paths) = path_env.get(name) {
                Some(
                    paths
                        .iter()
                        .map(|path| vec![IncludePathTemplatePart::Static(path.clone())])
                        .collect(),
                )
            } else {
                Some(vec![vec![IncludePathTemplatePart::Dynamic]])
            }
        }
        Expr::MagicConstant(MagicConstantKind::File, _) => {
            Some(vec![vec![IncludePathTemplatePart::Static(
                source_file.to_string(),
            )]])
        }
        Expr::MagicConstant(MagicConstantKind::Dir, _) => {
            Some(vec![vec![IncludePathTemplatePart::Static(
                source_dir.to_string(),
            )]])
        }
        Expr::Constant(name, _) if name == "DIRECTORY_SEPARATOR" => {
            Some(vec![vec![IncludePathTemplatePart::Static(
                std::path::MAIN_SEPARATOR.to_string(),
            )]])
        }
        Expr::Constant(name, _) if name == "PATH_SEPARATOR" => {
            Some(vec![vec![IncludePathTemplatePart::Static(
                if cfg!(windows) { ";" } else { ":" }.to_string(),
            )]])
        }
        Expr::Call {
            name, arguments, ..
        } if arguments.len() == 1
            && (name.eq_ignore_ascii_case("strtolower")
                || name.eq_ignore_ascii_case("strtoupper")
                || name.eq_ignore_ascii_case("lcfirst")
                || name.eq_ignore_ascii_case("ucfirst")) =>
        {
            let templates =
                bounded_include_path_templates(&arguments[0], source_file, source_dir, path_env)?;
            let mut transformed = Vec::new();
            for template in templates {
                if template
                    .iter()
                    .any(|part| matches!(part, IncludePathTemplatePart::Dynamic))
                {
                    transformed.push(vec![IncludePathTemplatePart::Dynamic]);
                    continue;
                }
                let mut value = template_to_string(&template);
                if name.eq_ignore_ascii_case("strtolower") {
                    value = value.to_ascii_lowercase();
                } else if name.eq_ignore_ascii_case("strtoupper") {
                    value = value.to_ascii_uppercase();
                } else if name.eq_ignore_ascii_case("lcfirst") {
                    if let Some(first) = value.get_mut(0..1) {
                        first.make_ascii_lowercase();
                    }
                } else if let Some(first) = value.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                transformed.push(vec![IncludePathTemplatePart::Static(value)]);
            }
            Some(transformed)
        }
        Expr::Binary {
            op: BinaryOp::Concat,
            left,
            right,
            ..
        } => {
            let left_templates =
                bounded_include_path_templates(left, source_file, source_dir, path_env)?;
            let right_templates =
                bounded_include_path_templates(right, source_file, source_dir, path_env)?;
            concat_include_path_templates(&left_templates, &right_templates)
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            let mut templates = Vec::new();
            let true_expr = if_true.as_deref().unwrap_or(condition);
            for template in
                bounded_include_path_templates(true_expr, source_file, source_dir, path_env)?
            {
                push_unique_template(&mut templates, template);
            }
            for template in
                bounded_include_path_templates(if_false, source_file, source_dir, path_env)?
            {
                push_unique_template(&mut templates, template);
            }
            if templates.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                return None;
            }
            Some(templates)
        }
        Expr::Match { arms, .. } => {
            let mut templates = Vec::new();
            for arm in arms {
                for template in
                    bounded_include_path_templates(&arm.value, source_file, source_dir, path_env)?
                {
                    push_unique_template(&mut templates, template);
                }
                if templates.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
                    return None;
                }
            }
            Some(templates)
        }
        Expr::Grouped { expr, .. } => {
            bounded_include_path_templates(expr, source_file, source_dir, path_env)
        }
        _ => None,
    }
}

fn bounded_interpolated_string_templates(
    parts: &[StringPart],
    path_env: &IncludePathEnv,
) -> Option<Vec<Vec<IncludePathTemplatePart>>> {
    let mut templates = vec![Vec::new()];
    for part in parts {
        match part {
            StringPart::Literal(value) => {
                for template in &mut templates {
                    push_template_static(template, value.clone());
                }
            }
            StringPart::Variable(name) | StringPart::LegacyDollarBraceVariable(name) => {
                if let Some(values) = path_env.get(name) {
                    if templates.len().saturating_mul(values.len()) > MAX_BOUNDED_INCLUDE_CANDIDATES
                    {
                        return None;
                    }
                    let mut expanded = Vec::new();
                    for template in &templates {
                        for value in values {
                            let mut next = template.clone();
                            push_template_static(&mut next, value.clone());
                            push_unique_template(&mut expanded, next);
                        }
                    }
                    templates = expanded;
                } else {
                    for template in &mut templates {
                        template.push(IncludePathTemplatePart::Dynamic);
                    }
                }
            }
            StringPart::PropertyFetch { .. }
            | StringPart::PropertyChain { .. }
            | StringPart::MethodCall { .. }
            | StringPart::ArrayAccess { .. }
            | StringPart::Expression(_)
            | StringPart::LegacyDollarBraceExpression(_)
            | StringPart::DynamicVariableExpression(_) => return None,
        }
    }
    Some(templates)
}

fn concat_include_path_templates(
    left: &[Vec<IncludePathTemplatePart>],
    right: &[Vec<IncludePathTemplatePart>],
) -> Option<Vec<Vec<IncludePathTemplatePart>>> {
    if left.len().saturating_mul(right.len()) > MAX_BOUNDED_INCLUDE_CANDIDATES {
        return None;
    }
    let mut templates = Vec::new();
    for left_template in left {
        for right_template in right {
            let mut template = left_template.clone();
            for part in right_template {
                match part {
                    IncludePathTemplatePart::Static(value) => {
                        push_template_static(&mut template, value.clone());
                    }
                    IncludePathTemplatePart::Dynamic => {
                        template.push(IncludePathTemplatePart::Dynamic);
                    }
                }
            }
            push_unique_template(&mut templates, template);
        }
    }
    Some(templates)
}

fn push_template_static(template: &mut Vec<IncludePathTemplatePart>, value: String) {
    if value.is_empty() {
        return;
    }
    if let Some(IncludePathTemplatePart::Static(previous)) = template.last_mut() {
        previous.push_str(&value);
    } else {
        template.push(IncludePathTemplatePart::Static(value));
    }
}

fn push_unique_template(
    templates: &mut Vec<Vec<IncludePathTemplatePart>>,
    template: Vec<IncludePathTemplatePart>,
) {
    if !templates.contains(&template) {
        templates.push(template);
    }
}

fn template_to_string(template: &[IncludePathTemplatePart]) -> String {
    let mut value = String::new();
    for part in template {
        if let IncludePathTemplatePart::Static(segment) = part {
            value.push_str(segment);
        }
    }
    value
}

fn expand_dynamic_include_template(
    template: &[IncludePathTemplatePart],
    source_dir: &str,
) -> Option<Vec<String>> {
    let pattern = template_to_marker_pattern(template);
    let first_dynamic = pattern.find('\0')?;
    let static_prefix = &pattern[..first_dynamic];
    let separator_index = last_path_separator_before(static_prefix, static_prefix.len());
    let (dir_part, filename_pattern) = if let Some(index) = separator_index {
        let dir = if index == 0 && static_prefix.starts_with('/') {
            "/"
        } else {
            &static_prefix[..index]
        };
        (dir, &pattern[index + 1..])
    } else {
        ("", pattern.as_str())
    };
    if filename_pattern.chars().any(is_path_separator) {
        return None;
    }
    if filename_pattern.split('\0').all(str::is_empty) {
        return None;
    }

    let dir_path = if dir_part.is_empty() {
        PathBuf::from(source_dir)
    } else if Path::new(dir_part).is_absolute() {
        PathBuf::from(dir_part)
    } else {
        Path::new(source_dir).join(dir_part)
    };
    let entries = fs::read_dir(&dir_path).ok()?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.ok()?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !marker_pattern_matches(&file_name, filename_pattern) {
            continue;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        push_unique_string(&mut candidates, path.to_string_lossy().into_owned());
        if candidates.len() > MAX_BOUNDED_INCLUDE_CANDIDATES {
            return None;
        }
    }
    candidates.sort();
    Some(candidates)
}

fn template_to_marker_pattern(template: &[IncludePathTemplatePart]) -> String {
    let mut pattern = String::new();
    for part in template {
        match part {
            IncludePathTemplatePart::Static(value) => pattern.push_str(value),
            IncludePathTemplatePart::Dynamic => pattern.push('\0'),
        }
    }
    pattern
}

fn last_path_separator_before(value: &str, end: usize) -> Option<usize> {
    value[..end]
        .char_indices()
        .filter_map(|(index, ch)| is_path_separator(ch).then_some(index))
        .last()
}

fn is_path_separator(ch: char) -> bool {
    ch == '/' || (cfg!(windows) && ch == '\\')
}

fn marker_pattern_matches(name: &str, pattern: &str) -> bool {
    let chunks: Vec<_> = pattern.split('\0').collect();
    if chunks.len() <= 1 || chunks.iter().all(|chunk| chunk.is_empty()) {
        return false;
    }

    let mut offset = 0;
    if let Some(first) = chunks.first() {
        if !first.is_empty() {
            if !name.starts_with(first) {
                return false;
            }
            offset = first.len();
        }
    }

    for chunk in chunks.iter().skip(1).take(chunks.len().saturating_sub(2)) {
        if chunk.is_empty() {
            continue;
        }
        let Some(found) = name[offset..].find(chunk) else {
            return false;
        };
        offset += found + chunk.len();
    }

    if let Some(last) = chunks.last() {
        if !last.is_empty() && !name[offset..].ends_with(last) {
            return false;
        }
    }
    true
}

fn bounded_interpolated_string_paths(
    parts: &[StringPart],
    path_env: &IncludePathEnv,
) -> Option<Vec<String>> {
    let mut paths = vec![String::new()];
    for part in parts {
        match part {
            StringPart::Literal(value) => {
                for path in &mut paths {
                    path.push_str(value);
                }
            }
            StringPart::Variable(name) | StringPart::LegacyDollarBraceVariable(name) => {
                let values = path_env.get(name)?;
                if paths.len().saturating_mul(values.len()) > MAX_BOUNDED_INCLUDE_CANDIDATES {
                    return None;
                }
                let mut expanded = Vec::new();
                for path in &paths {
                    for value in values {
                        let mut expanded_path = path.clone();
                        expanded_path.push_str(value);
                        push_unique_string(&mut expanded, expanded_path);
                    }
                }
                paths = expanded;
            }
            StringPart::PropertyFetch { .. }
            | StringPart::PropertyChain { .. }
            | StringPart::MethodCall { .. }
            | StringPart::ArrayAccess { .. }
            | StringPart::Expression(_)
            | StringPart::LegacyDollarBraceExpression(_)
            | StringPart::DynamicVariableExpression(_) => return None,
        }
    }
    Some(paths)
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

fn resolve_include_candidate_path(path: &str, source_dir: &str) -> ResolvedIncludeCandidate {
    if let Some(filter_path) = parse_php_filter_include_path(path, source_dir) {
        return filter_path;
    }
    let resource_path = resolve_include_path(path, source_dir);
    ResolvedIncludeCandidate {
        path_aliases: Vec::new(),
        resource_path,
        transform: None,
    }
}

fn parse_php_filter_include_path(path: &str, source_dir: &str) -> Option<ResolvedIncludeCandidate> {
    let rest = path.strip_prefix("php://filter/")?;
    let (filter_spec, resource) = rest.split_once("/resource=")?;
    if resource.is_empty() {
        return None;
    }

    let mut read_filters = Vec::new();
    for segment in filter_spec.split('/') {
        let Some(filters) = segment.strip_prefix("read=") else {
            continue;
        };
        for filter in filters.split('|') {
            let filter = php_filter_read_filter(filter)?;
            read_filters.push(filter);
        }
    }
    if read_filters.is_empty() {
        return None;
    }

    let resource_path = resolve_include_path(resource, source_dir);
    Some(ResolvedIncludeCandidate {
        resource_path,
        path_aliases: vec![path.to_string()],
        transform: Some(IncludeSourceTransform::PhpFilter(read_filters)),
    })
}

fn php_filter_read_filter(name: &str) -> Option<PhpFilterReadFilter> {
    if name.eq_ignore_ascii_case("string.tolower") {
        Some(PhpFilterReadFilter::StringToLower)
    } else if name.eq_ignore_ascii_case("string.toupper") {
        Some(PhpFilterReadFilter::StringToUpper)
    } else if name.eq_ignore_ascii_case("string.rot13") {
        Some(PhpFilterReadFilter::StringRot13)
    } else {
        None
    }
}

fn apply_include_source_transform(
    source_bytes: &mut [u8],
    transform: Option<&IncludeSourceTransform>,
) {
    let Some(transform) = transform else {
        return;
    };
    match transform {
        IncludeSourceTransform::PhpFilter(filters) => {
            for filter in filters {
                apply_php_filter_read_filter(source_bytes, filter);
            }
        }
    }
}

fn apply_php_filter_read_filter(source_bytes: &mut [u8], filter: &PhpFilterReadFilter) {
    match filter {
        PhpFilterReadFilter::StringToLower => source_bytes.make_ascii_lowercase(),
        PhpFilterReadFilter::StringToUpper => source_bytes.make_ascii_uppercase(),
        PhpFilterReadFilter::StringRot13 => {
            for byte in source_bytes {
                *byte = match *byte {
                    b'a'..=b'z' => ((*byte - b'a' + 13) % 26) + b'a',
                    b'A'..=b'Z' => ((*byte - b'A' + 13) % 26) + b'A',
                    _ => *byte,
                };
            }
        }
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
