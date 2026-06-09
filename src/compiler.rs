use std::fs;
use std::path::{Path, PathBuf};

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
