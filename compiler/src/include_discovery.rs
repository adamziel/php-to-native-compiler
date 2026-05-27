use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{BinaryOp, Expr, Program, Span, Stmt};
use crate::error::{CompileResult, Diagnostic, Phase};
use crate::parser::parse_source;

const UNSUPPORTED_DYNAMIC_INCLUDE_PATH: &str = "unsupported include/require path: generated C include class discovery requires a literal same-repository path built from string literals and __DIR__";
const UNSUPPORTED_OUT_OF_REPOSITORY_INCLUDE: &str = "unsupported include/require path: generated C include class discovery only supports same-repository files";
const UNSUPPORTED_CYCLIC_INCLUDE: &str = "unsupported cyclic include graph: generated C include class discovery requires an acyclic literal include graph";
const UNSUPPORTED_NON_DECLARATION_INCLUDE_TOP_LEVEL: &str = "unsupported include/require file: generated C include declaration discovery only supports class, interface, and trait declarations plus nested literal includes; top-level executable side effects and autoload registration remain blocked";
const UNSUPPORTED_LATE_INCLUDE: &str = "unsupported include/require placement: generated C include class discovery requires literal includes to appear before executable top-level statements";
const UNSUPPORTED_DYNAMIC_INCLUDE_PATH_MUTATION: &str = "unsupported set_include_path(): generated C include/require include_path search requires a literal compile-time path string";

const INCLUDE_PATH_SEPARATOR: char = if cfg!(windows) { ';' } else { ':' };
const MAX_STATIC_INCLUDE_PATH_VALUES: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    pub program: Program,
    pub include_metadata: IncludeGraphMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeGraphMetadata {
    pub root_file: PathBuf,
    pub included_files: Vec<IncludedFileMetadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludedFileMetadata {
    pub path: PathBuf,
    pub class_names: Vec<String>,
    pub interface_names: Vec<String>,
    pub trait_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableCompilationUnit {
    pub program: Program,
    pub root_file: PathBuf,
    pub include_units: Vec<ExecutableIncludeUnit>,
    pub include_resolutions: Vec<ExecutableIncludeResolution>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableIncludeUnit {
    pub path: PathBuf,
    pub program: Program,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableIncludeResolution {
    pub source_file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub path: PathBuf,
    pub found: bool,
    pub requested_path: String,
    pub include_path: String,
}

#[derive(Debug, Clone, PartialEq)]
struct IncludePathResolution {
    path: PathBuf,
    found: bool,
    requested_path: String,
    include_path: String,
}

pub fn parse_source_with_literal_include_metadata(
    source: &str,
    source_file: impl AsRef<Path>,
    repo_root: impl AsRef<Path>,
) -> CompileResult<Program> {
    Ok(compilation_unit_with_literal_include_metadata(source, source_file, repo_root)?.program)
}

pub fn compilation_unit_with_literal_include_metadata(
    source: &str,
    source_file: impl AsRef<Path>,
    repo_root: impl AsRef<Path>,
) -> CompileResult<CompilationUnit> {
    let source_file = source_file.as_ref();
    let repo_root = canonicalize_existing_path(repo_root.as_ref(), Phase::Io, Span::new(0, 0))?;
    let root_file = canonicalize_source_file(source_file);
    let program = parse_source(source)?;

    let mut discovery = IncludeDiscovery {
        repo_root,
        visiting: HashSet::new(),
        included_once: HashSet::new(),
        included_files: Vec::new(),
        include_path: ".".to_string(),
    };
    discovery.visiting.insert(root_file.clone());
    let statements = discovery.expand_statements(&program.statements, source_file, false)?;
    discovery.visiting.remove(&root_file);

    Ok(CompilationUnit {
        program: Program { statements },
        include_metadata: IncludeGraphMetadata {
            root_file,
            included_files: discovery.included_files,
        },
    })
}

pub fn executable_compilation_unit_with_literal_include_units(
    source: &str,
    source_file: impl AsRef<Path>,
    repo_root: impl AsRef<Path>,
) -> CompileResult<ExecutableCompilationUnit> {
    let source_file = source_file.as_ref();
    let repo_root = canonicalize_existing_path(repo_root.as_ref(), Phase::Io, Span::new(0, 0))?;
    let root_file = canonicalize_source_file(source_file);
    let program = parse_source(source)?;

    let mut discovery = IncludeUnitDiscovery {
        repo_root,
        visiting: HashSet::new(),
        discovered: HashSet::new(),
        include_units: Vec::new(),
        include_path: ".".to_string(),
        include_resolutions: Vec::new(),
        known_strings: HashMap::new(),
    };
    discovery.visiting.insert(root_file);
    discovery.discover_program(&program, source_file)?;

    Ok(ExecutableCompilationUnit {
        program,
        root_file: canonicalize_source_file(source_file),
        include_units: discovery.include_units,
        include_resolutions: discovery.include_resolutions,
    })
}

struct IncludeDiscovery {
    repo_root: PathBuf,
    visiting: HashSet<PathBuf>,
    included_once: HashSet<PathBuf>,
    included_files: Vec<IncludedFileMetadata>,
    include_path: String,
}

impl IncludeDiscovery {
    fn expand_statements(
        &mut self,
        statements: &[Stmt],
        source_file: &Path,
        included_file: bool,
    ) -> CompileResult<Vec<Stmt>> {
        let mut expanded = Vec::new();
        let mut saw_executable = false;

        for stmt in statements {
            match stmt {
                Stmt::Expr { expr, span } if is_set_include_path_call(expr) => {
                    if included_file {
                        return Err(Diagnostic::new(
                            Phase::Parse,
                            span.line,
                            span.column,
                            UNSUPPORTED_NON_DECLARATION_INCLUDE_TOP_LEVEL,
                        ));
                    }
                    self.include_path = literal_set_include_path(expr, source_file, *span)?;
                    expanded.push(stmt.clone());
                }
                Stmt::Require { path, once, span } | Stmt::Include { path, once, span } => {
                    if saw_executable {
                        return Err(Diagnostic::new(
                            Phase::Parse,
                            span.line,
                            span.column,
                            UNSUPPORTED_LATE_INCLUDE,
                        ));
                    }
                    expanded.extend(self.expand_include(path, *once, *span, source_file)?);
                }
                Stmt::Class(_) | Stmt::Interface(_) | Stmt::Trait(_) => expanded.push(stmt.clone()),
                _ if included_file => {
                    let span = stmt.span();
                    return Err(Diagnostic::new(
                        Phase::Parse,
                        span.line,
                        span.column,
                        UNSUPPORTED_NON_DECLARATION_INCLUDE_TOP_LEVEL,
                    ));
                }
                _ => {
                    saw_executable = true;
                    expanded.push(stmt.clone());
                }
            }
        }

        Ok(expanded)
    }

    fn expand_include(
        &mut self,
        path: &Expr,
        once: bool,
        span: Span,
        source_file: &Path,
    ) -> CompileResult<Vec<Stmt>> {
        let include_path = self.resolve_include_path(path, source_file, span)?;
        if once && !self.included_once.insert(include_path.clone()) {
            return Ok(Vec::new());
        }
        if !self.visiting.insert(include_path.clone()) {
            return Err(Diagnostic::new(
                Phase::Parse,
                span.line,
                span.column,
                UNSUPPORTED_CYCLIC_INCLUDE,
            ));
        }

        let source = fs::read_to_string(&include_path).map_err(|error| {
            Diagnostic::new(Phase::Io, span.line, span.column, error.to_string())
                .with_file(&include_path)
        })?;
        let program = parse_source(&source).map_err(|error| error.with_file(&include_path))?;
        let statements = self.expand_statements(&program.statements, &include_path, true)?;
        let class_names = statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Class(class) => Some(class.name.clone()),
                _ => None,
            })
            .collect();
        let interface_names = statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Interface(interface) => Some(interface.name.clone()),
                _ => None,
            })
            .collect();
        let trait_names = statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Trait(trait_decl) => Some(trait_decl.name.clone()),
                _ => None,
            })
            .collect();
        self.included_files.push(IncludedFileMetadata {
            path: include_path.clone(),
            class_names,
            interface_names,
            trait_names,
        });

        self.visiting.remove(&include_path);
        Ok(statements)
    }

    fn resolve_include_path(
        &self,
        path: &Expr,
        source_file: &Path,
        span: Span,
    ) -> CompileResult<PathBuf> {
        resolve_literal_include_path_with_include_path(
            path,
            source_file,
            &self.repo_root,
            &self.include_path,
            span,
        )
    }
}

struct IncludeUnitDiscovery {
    repo_root: PathBuf,
    visiting: HashSet<PathBuf>,
    discovered: HashSet<PathBuf>,
    include_units: Vec<ExecutableIncludeUnit>,
    include_path: String,
    include_resolutions: Vec<ExecutableIncludeResolution>,
    known_strings: HashMap<String, Vec<String>>,
}

impl IncludeUnitDiscovery {
    fn discover_program(&mut self, program: &Program, source_file: &Path) -> CompileResult<()> {
        for stmt in &program.statements {
            self.discover_statement(stmt, source_file)?;
        }
        Ok(())
    }

    fn discover_statement(&mut self, stmt: &Stmt, source_file: &Path) -> CompileResult<()> {
        match stmt {
            Stmt::Expr { expr, span } if is_set_include_path_call(expr) => {
                self.include_path = literal_set_include_path(expr, source_file, *span)?;
                Ok(())
            }
            Stmt::Assign { target, expr, .. } => {
                self.discover_expression(expr, source_file)?;
                self.record_assignment_target_string(target, expr, source_file);
                Ok(())
            }
            Stmt::Require {
                path, once, span, ..
            }
            | Stmt::Include {
                path, once, span, ..
            } => self.discover_include(path, *once, *span, source_file),
            Stmt::Echo { exprs, .. } => self.discover_expressions(exprs, source_file),
            Stmt::Print { expr, .. }
            | Stmt::CompoundAssign { expr, .. }
            | Stmt::NullCoalesceAssign { expr, .. }
            | Stmt::Expr { expr, .. }
            | Stmt::Throw { expr, .. } => self.discover_expression(expr, source_file),
            Stmt::ReferenceAssign { .. } => Ok(()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.discover_expression(condition, source_file)?;
                self.discover_statements(then_branch, source_file)?;
                self.discover_statements(else_branch, source_file)
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.discover_expression(condition, source_file)?;
                self.discover_statements(body, source_file)
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                self.discover_statements(body, source_file)?;
                self.discover_expression(condition, source_file)
            }
            Stmt::For {
                initializers,
                conditions,
                increments,
                body,
                ..
            } => {
                self.discover_for_actions(initializers, source_file)?;
                self.discover_expressions(conditions, source_file)?;
                self.discover_for_actions(increments, source_file)?;
                self.discover_statements(body, source_file)
            }
            Stmt::Switch { value, cases, .. } => {
                self.discover_expression(value, source_file)?;
                for case in cases {
                    if let Some(value) = &case.condition {
                        self.discover_expression(value, source_file)?;
                    }
                    self.discover_statements(&case.body, source_file)?;
                }
                Ok(())
            }
            Stmt::Foreach { iterable, body, .. } => {
                self.discover_expression(iterable, source_file)?;
                self.discover_statements(body, source_file)
            }
            Stmt::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                self.discover_statements(body, source_file)?;
                for catch in catches {
                    self.discover_statements(&catch.body, source_file)?;
                }
                if let Some(finally_body) = finally_body {
                    self.discover_statements(finally_body, source_file)?;
                }
                Ok(())
            }
            Stmt::Function(function) => self.discover_statements(&function.body, source_file),
            Stmt::Class(class) => {
                for member in &class.members {
                    if let crate::ast::ClassMember::Method(method) = member {
                        self.discover_statements(&method.function.body, source_file)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn discover_statements(
        &mut self,
        statements: &[Stmt],
        source_file: &Path,
    ) -> CompileResult<()> {
        for stmt in statements {
            self.discover_statement(stmt, source_file)?;
        }
        Ok(())
    }

    fn discover_for_actions(
        &mut self,
        actions: &[crate::ast::ForAction],
        source_file: &Path,
    ) -> CompileResult<()> {
        for action in actions {
            match action {
                crate::ast::ForAction::Assign { expr, .. }
                | crate::ast::ForAction::CompoundAssign { expr, .. }
                | crate::ast::ForAction::Expr { expr } => {
                    self.discover_expression(expr, source_file)?;
                }
                crate::ast::ForAction::IncrementDecrement { .. } => {}
            }
        }
        Ok(())
    }

    fn discover_expressions(&mut self, exprs: &[Expr], source_file: &Path) -> CompileResult<()> {
        for expr in exprs {
            self.discover_expression(expr, source_file)?;
        }
        Ok(())
    }

    fn discover_expression(&mut self, expr: &Expr, source_file: &Path) -> CompileResult<()> {
        match expr {
            Expr::Call { span, .. } if is_set_include_path_call(expr) => {
                self.include_path = literal_set_include_path(expr, source_file, *span)?;
                Ok(())
            }
            Expr::Include {
                path, once, span, ..
            }
            | Expr::Require {
                path, once, span, ..
            } => self.discover_include(path, *once, *span, source_file),
            Expr::Binary { left, right, .. } => {
                self.discover_expression(left, source_file)?;
                self.discover_expression(right, source_file)
            }
            Expr::Unary { expr, .. }
            | Expr::ErrorControl { expr, .. }
            | Expr::Cast { expr, .. }
            | Expr::Clone { expr, .. } => self.discover_expression(expr, source_file),
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                self.discover_expression(condition, source_file)?;
                self.discover_expression(if_true, source_file)?;
                self.discover_expression(if_false, source_file)
            }
            Expr::ShortTernary {
                condition,
                if_false,
                ..
            } => {
                self.discover_expression(condition, source_file)?;
                self.discover_expression(if_false, source_file)
            }
            Expr::Assign { expr, .. } | Expr::CompoundAssign { expr, .. } => {
                self.discover_expression(expr, source_file)
            }
            Expr::Call { args, .. } | Expr::DynamicCall { args, .. } | Expr::New { args, .. } => {
                self.discover_expressions(args, source_file)
            }
            Expr::Array { items, .. } => {
                for item in items {
                    if let Some(key) = &item.key {
                        self.discover_expression(key, source_file)?;
                    }
                    self.discover_expression(&item.value, source_file)?;
                }
                Ok(())
            }
            Expr::Closure { body, .. } => self.discover_statements(body, source_file),
            _ => Ok(()),
        }
    }

    fn discover_include(
        &mut self,
        path: &Expr,
        once: bool,
        span: Span,
        source_file: &Path,
    ) -> CompileResult<()> {
        self.discover_expression(path, source_file)?;
        let source_dir = source_file.parent().unwrap_or(Path::new(""));
        let Some(requested_paths) =
            static_include_path_values(path, source_dir, false, &self.known_strings)
        else {
            return Ok(());
        };
        for requested_path in requested_paths {
            let resolution = resolve_raw_include_path_result_with_include_path(
                requested_path,
                source_file,
                &self.repo_root,
                &self.include_path,
                span,
            )?;
            let include_path = resolution.path.clone();
            self.include_resolutions.push(ExecutableIncludeResolution {
                source_file: canonicalize_source_file(source_file),
                line: span.line,
                column: span.column,
                path: include_path.clone(),
                found: resolution.found,
                requested_path: resolution.requested_path,
                include_path: resolution.include_path,
            });
            if !resolution.found {
                continue;
            }
            if self.discovered.contains(&include_path) {
                continue;
            }
            if once && self.visiting.contains(&include_path) {
                continue;
            }
            if !self.visiting.insert(include_path.clone()) {
                return Err(Diagnostic::new(
                    Phase::Parse,
                    span.line,
                    span.column,
                    UNSUPPORTED_CYCLIC_INCLUDE,
                ));
            }
            let source = fs::read_to_string(&include_path).map_err(|error| {
                Diagnostic::new(Phase::Io, span.line, span.column, error.to_string())
                    .with_file(&include_path)
            })?;
            let program = parse_source(&source).map_err(|error| error.with_file(&include_path))?;
            self.discover_program(&program, &include_path)?;
            self.visiting.remove(&include_path);
            self.discovered.insert(include_path.clone());
            self.include_units.push(ExecutableIncludeUnit {
                path: include_path,
                program,
            });
        }
        Ok(())
    }

    fn record_assignment_target_string(
        &mut self,
        target: &crate::ast::AssignTarget,
        expr: &Expr,
        source_file: &Path,
    ) {
        let crate::ast::AssignTarget::Variable { name, .. } = target else {
            return;
        };
        let source_dir = source_file.parent().unwrap_or(Path::new(""));
        if let Some(values) =
            static_include_path_values(expr, source_dir, false, &self.known_strings)
        {
            self.known_strings.insert(name.clone(), values);
        } else {
            self.known_strings.remove(name);
        }
    }
}

fn literal_include_path(expr: &Expr, source_dir: &Path) -> Option<String> {
    literal_path_string(expr, source_dir, false)
}

fn literal_include_path_setting(expr: &Expr, source_dir: &Path) -> Option<String> {
    literal_path_string(expr, source_dir, true)
}

fn literal_path_string(
    expr: &Expr,
    source_dir: &Path,
    allow_path_separator: bool,
) -> Option<String> {
    match expr {
        Expr::String(value, _) => Some(value.clone()),
        Expr::MagicDir { .. } => Some(source_dir.to_string_lossy().into_owned()),
        Expr::GlobalConstant { name, .. }
            if allow_path_separator && name.eq_ignore_ascii_case("PATH_SEPARATOR") =>
        {
            Some(INCLUDE_PATH_SEPARATOR.to_string())
        }
        Expr::Binary {
            left,
            op: BinaryOp::Concat,
            right,
            ..
        } => {
            let mut value = literal_path_string(left, source_dir, allow_path_separator)?;
            value.push_str(&literal_path_string(
                right,
                source_dir,
                allow_path_separator,
            )?);
            Some(value)
        }
        _ => None,
    }
}

fn static_include_path_values(
    expr: &Expr,
    source_dir: &Path,
    allow_path_separator: bool,
    known_strings: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    match expr {
        Expr::String(value, _) => Some(vec![value.clone()]),
        Expr::MagicDir { .. } => Some(vec![source_dir.to_string_lossy().into_owned()]),
        Expr::GlobalConstant { name, .. }
            if allow_path_separator && name.eq_ignore_ascii_case("PATH_SEPARATOR") =>
        {
            Some(vec![INCLUDE_PATH_SEPARATOR.to_string()])
        }
        Expr::Variable(name, _) => known_strings.get(name).cloned(),
        Expr::Binary {
            left,
            op: BinaryOp::Concat,
            right,
            ..
        } => {
            let left_values =
                static_include_path_values(left, source_dir, allow_path_separator, known_strings)?;
            let right_values =
                static_include_path_values(right, source_dir, allow_path_separator, known_strings)?;
            let mut values = Vec::new();
            for left in &left_values {
                for right in &right_values {
                    values.push(format!("{left}{right}"));
                }
            }
            dedupe_static_include_path_values(values)
        }
        Expr::Ternary {
            if_true, if_false, ..
        } => {
            let mut values = static_include_path_values(
                if_true,
                source_dir,
                allow_path_separator,
                known_strings,
            )?;
            values.extend(static_include_path_values(
                if_false,
                source_dir,
                allow_path_separator,
                known_strings,
            )?);
            dedupe_static_include_path_values(values)
        }
        Expr::ShortTernary {
            condition,
            if_false,
            ..
        } => {
            let mut values = static_include_path_values(
                condition,
                source_dir,
                allow_path_separator,
                known_strings,
            )?;
            values.extend(static_include_path_values(
                if_false,
                source_dir,
                allow_path_separator,
                known_strings,
            )?);
            dedupe_static_include_path_values(values)
        }
        _ => None,
    }
}

fn dedupe_static_include_path_values(values: Vec<String>) -> Option<Vec<String>> {
    let mut unique = Vec::new();
    for value in values {
        if unique.contains(&value) {
            continue;
        }
        unique.push(value);
        if unique.len() > MAX_STATIC_INCLUDE_PATH_VALUES {
            return None;
        }
    }
    if unique.is_empty() {
        None
    } else {
        Some(unique)
    }
}

fn is_set_include_path_call(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { name, args, .. } if name.eq_ignore_ascii_case("set_include_path") && !args.is_empty())
}

fn literal_set_include_path(expr: &Expr, source_file: &Path, span: Span) -> CompileResult<String> {
    let Expr::Call { args, .. } = expr else {
        unreachable!("set_include_path guard passed non-call expression");
    };
    let [path] = args.as_slice() else {
        return Err(Diagnostic::new(
            Phase::Parse,
            span.line,
            span.column,
            UNSUPPORTED_DYNAMIC_INCLUDE_PATH_MUTATION,
        ));
    };
    literal_include_path_setting(path, source_file.parent().unwrap_or(Path::new(""))).ok_or_else(
        || {
            Diagnostic::new(
                Phase::Parse,
                span.line,
                span.column,
                UNSUPPORTED_DYNAMIC_INCLUDE_PATH_MUTATION,
            )
        },
    )
}

pub fn resolve_literal_include_path(
    path: &Expr,
    source_file: &Path,
    repo_root: &Path,
    span: Span,
) -> CompileResult<PathBuf> {
    resolve_literal_include_path_with_include_path(path, source_file, repo_root, ".", span)
}

pub fn resolve_literal_include_path_with_include_path(
    path: &Expr,
    source_file: &Path,
    repo_root: &Path,
    include_path: &str,
    span: Span,
) -> CompileResult<PathBuf> {
    resolve_literal_include_path_result_with_include_path(
        path,
        source_file,
        repo_root,
        include_path,
        span,
    )
    .and_then(|resolution| {
        if resolution.found {
            Ok(resolution.path)
        } else {
            Err(Diagnostic::new(
                Phase::Io,
                span.line,
                span.column,
                "No such file or directory",
            )
            .with_file(&resolution.path))
        }
    })
}

fn resolve_literal_include_path_result_with_include_path(
    path: &Expr,
    source_file: &Path,
    repo_root: &Path,
    include_path: &str,
    span: Span,
) -> CompileResult<IncludePathResolution> {
    let Some(raw_path_value) =
        literal_include_path(path, source_file.parent().unwrap_or(Path::new("")))
    else {
        return Err(Diagnostic::new(
            Phase::Parse,
            span.line,
            span.column,
            UNSUPPORTED_DYNAMIC_INCLUDE_PATH,
        ));
    };
    resolve_raw_include_path_result_with_include_path(
        raw_path_value,
        source_file,
        repo_root,
        include_path,
        span,
    )
}

fn resolve_raw_include_path_result_with_include_path(
    raw_path_value: String,
    source_file: &Path,
    repo_root: &Path,
    include_path: &str,
    span: Span,
) -> CompileResult<IncludePathResolution> {
    let raw_path = PathBuf::from(&raw_path_value);
    let (path, found) = if raw_path.is_absolute() {
        if raw_path.exists() {
            (
                canonicalize_existing_path(&raw_path, Phase::Io, span)?,
                true,
            )
        } else {
            (raw_path, false)
        }
    } else {
        let source_relative = source_file
            .parent()
            .unwrap_or(Path::new(""))
            .join(&raw_path);
        let mut existing = None;
        for candidate in include_path_candidates(&source_relative, &raw_path, include_path) {
            if candidate.exists() {
                existing = Some(candidate);
                break;
            }
        }
        let candidate = existing.unwrap_or(source_relative);
        if candidate.exists() {
            (
                canonicalize_existing_path(&candidate, Phase::Io, span)?,
                true,
            )
        } else {
            (candidate, false)
        }
    };
    if found && !path.starts_with(repo_root) {
        return Err(Diagnostic::new(
            Phase::Parse,
            span.line,
            span.column,
            UNSUPPORTED_OUT_OF_REPOSITORY_INCLUDE,
        ));
    }
    Ok(IncludePathResolution {
        path,
        found,
        requested_path: raw_path_value,
        include_path: include_path.to_string(),
    })
}

fn include_path_candidates(
    source_relative_path: &Path,
    requested_path: &Path,
    include_path: &str,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    for entry in include_path.split(INCLUDE_PATH_SEPARATOR) {
        let entry = if entry.is_empty() { "." } else { entry };
        candidates.push(PathBuf::from(entry).join(requested_path));
    }

    candidates.push(source_relative_path.to_path_buf());
    candidates
}

fn canonicalize_existing_path(path: &Path, phase: Phase, span: Span) -> CompileResult<PathBuf> {
    fs::canonicalize(path).map_err(|error| {
        Diagnostic::new(phase, span.line, span.column, error.to_string()).with_file(path)
    })
}

fn canonicalize_source_file(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
