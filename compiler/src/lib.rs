pub mod ast;
pub mod codegen;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod test_runner;

use ast::Program;
use error::CompileResult;
use interpreter::Execution;
use php_runtime::PhpClassTable;

pub fn parse(source: &str) -> CompileResult<Program> {
    parser::parse_source(source)
}

pub fn run_source(source: &str) -> CompileResult<Execution> {
    let program = parse(source)?;
    interpreter::run_program(&program)
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
