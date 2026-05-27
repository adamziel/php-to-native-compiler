use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{BinaryOp, Expr, Program, Span, Stmt};
use crate::error::{CompileResult, Diagnostic, Phase};
use crate::parser::parse_source;

const UNSUPPORTED_DYNAMIC_INCLUDE_PATH: &str = "unsupported include/require path: generated C include class discovery requires a literal same-repository path built from string literals and __DIR__";
const UNSUPPORTED_OUT_OF_REPOSITORY_INCLUDE: &str = "unsupported include/require path: generated C include class discovery only supports same-repository files";
const UNSUPPORTED_CYCLIC_INCLUDE: &str = "unsupported cyclic include graph: generated C include class discovery requires an acyclic literal include graph";
const UNSUPPORTED_NON_CLASS_INCLUDE_TOP_LEVEL: &str = "unsupported include/require file: generated C include class discovery only supports class declarations and nested literal includes; top-level executable side effects and autoload registration remain blocked";
const UNSUPPORTED_LATE_INCLUDE: &str = "unsupported include/require placement: generated C include class discovery requires literal includes to appear before executable top-level statements";

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

struct IncludeDiscovery {
    repo_root: PathBuf,
    visiting: HashSet<PathBuf>,
    included_once: HashSet<PathBuf>,
    included_files: Vec<IncludedFileMetadata>,
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
                Stmt::Class(_) => expanded.push(stmt.clone()),
                _ if included_file => {
                    let span = stmt.span();
                    return Err(Diagnostic::new(
                        Phase::Parse,
                        span.line,
                        span.column,
                        UNSUPPORTED_NON_CLASS_INCLUDE_TOP_LEVEL,
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
        self.included_files.push(IncludedFileMetadata {
            path: include_path.clone(),
            class_names,
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
        let Some(raw_path) =
            literal_include_path(path, source_file.parent().unwrap_or(Path::new("")))
        else {
            return Err(Diagnostic::new(
                Phase::Parse,
                span.line,
                span.column,
                UNSUPPORTED_DYNAMIC_INCLUDE_PATH,
            ));
        };
        let candidate = PathBuf::from(raw_path);
        let candidate = if candidate.is_absolute() {
            candidate
        } else {
            source_file
                .parent()
                .unwrap_or(Path::new(""))
                .join(candidate)
        };
        let canonical = canonicalize_existing_path(&candidate, Phase::Io, span)?;
        if !canonical.starts_with(&self.repo_root) {
            return Err(Diagnostic::new(
                Phase::Parse,
                span.line,
                span.column,
                UNSUPPORTED_OUT_OF_REPOSITORY_INCLUDE,
            ));
        }
        Ok(canonical)
    }
}

fn literal_include_path(expr: &Expr, source_dir: &Path) -> Option<String> {
    match expr {
        Expr::String(value, _) => Some(value.clone()),
        Expr::MagicDir { .. } => Some(source_dir.to_string_lossy().into_owned()),
        Expr::Binary {
            left,
            op: BinaryOp::Concat,
            right,
            ..
        } => {
            let mut value = literal_include_path(left, source_dir)?;
            value.push_str(&literal_include_path(right, source_dir)?);
            Some(value)
        }
        _ => None,
    }
}

fn canonicalize_existing_path(path: &Path, phase: Phase, span: Span) -> CompileResult<PathBuf> {
    fs::canonicalize(path).map_err(|error| {
        Diagnostic::new(phase, span.line, span.column, error.to_string()).with_file(path)
    })
}

fn canonicalize_source_file(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
