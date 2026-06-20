pub mod ast;
pub mod backend;
pub mod compiler;
pub mod diagnostic;
pub mod ir;
pub(crate) mod json_metadata;
pub mod lexer;
pub mod parser;

pub use compiler::{compile_file, CompileOptions, CompileOutput};
pub use diagnostic::{Diagnostic, DiagnosticKind, Result};
