pub mod ast;
pub mod call_arguments;
pub mod codegen;
pub mod error;
pub mod include_discovery;
pub mod interpreter;
mod legacy_hash_tables;
mod legacy_hashes;
pub mod lexer;
pub mod parser;
pub mod php_tokenizer;
pub mod test_runner;
pub mod trait_semantics;

use ast::Program;
use error::CompileResult;
pub use include_discovery::{
    compilation_unit_with_literal_include_metadata,
    executable_compilation_unit_with_literal_include_units,
    parse_source_with_literal_include_metadata, CompilationUnit, ExecutableCompilationUnit,
    ExecutableIncludeUnit, IncludeGraphMetadata, IncludedFileMetadata,
};
use interpreter::{Execution, RunOptions};
use php_runtime::PhpClassTable;

pub use codegen::{
    native_runtime_scalar_echo_probe_ir, native_runtime_scalar_echo_probe_ir_for_target,
    NativeRuntimeIrTarget,
};

pub fn parse(source: &str) -> CompileResult<Program> {
    parser::parse_source(source)
}

pub fn run_source(source: &str) -> CompileResult<Execution> {
    let program = parse(source)?;
    interpreter::run_program(&program)
}

pub fn run_source_with_source_file(
    source: &str,
    source_file: impl Into<String>,
) -> CompileResult<Execution> {
    let program = parse(source)?;
    interpreter::run_program_with_source_file(&program, source_file)
}

pub fn run_source_with_execution_step_limit(
    source: &str,
    max_execution_steps: usize,
) -> CompileResult<Execution> {
    let program = parse(source)?;
    interpreter::run_program_with_options(
        &program,
        RunOptions {
            max_execution_steps: Some(max_execution_steps),
            ..RunOptions::default()
        },
    )
}

pub fn class_metadata_source(source: &str) -> CompileResult<PhpClassTable> {
    let program = parse(source)?;
    interpreter::class_metadata(&program)
}

pub fn emit_ir_source(source: &str) -> CompileResult<String> {
    let program = parse(source)?;
    codegen::emit_llvm_ir(&program)
}

pub fn emit_asm_source(source: &str) -> CompileResult<String> {
    let program = parse(source)?;
    codegen::emit_assembly(&program)
}
