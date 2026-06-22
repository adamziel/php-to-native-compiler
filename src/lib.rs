pub mod ast;
pub mod backend;
pub mod compiler;
pub mod diagnostic;
pub mod ir;
pub mod lexer;
pub mod parser;

pub use compiler::{
    compile_file, compile_file_with_preloads, compile_file_with_preloads_and_source_options,
    CompileOptions, CompileOutput, CompileSourceOptions,
};
pub use diagnostic::{Diagnostic, DiagnosticKind, Result};
