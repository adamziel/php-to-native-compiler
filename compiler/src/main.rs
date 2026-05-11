use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use php_compiler::codegen::{emit_assembly, emit_llvm_ir};
use php_compiler::error::{CompileResult, Diagnostic, Phase};
use php_compiler::interpreter::run_program;
use php_compiler::parser::parse_source;
use php_compiler::test_runner::run_fixture_dir;

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn real_main() -> CompileResult<u8> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("compile") => command_compile(&args[1..]),
        Some("run") => command_run(&args[1..]),
        Some("test") => command_test(&args[1..]),
        Some("-h") | Some("--help") | None => {
            print_help();
            Ok(0)
        }
        Some(command) => Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            format!("unknown command '{command}'"),
        )),
    }
}

fn command_compile(args: &[String]) -> CompileResult<u8> {
    if args.len() != 2 {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "usage: phpc compile <input.php> (--emit-ir | --emit-asm)",
        ));
    }

    let input = PathBuf::from(&args[0]);
    let flag = args[1].as_str();
    let source = read_source(&input)?;
    let program = parse_source(&source).map_err(|error| error.with_file(&input))?;

    let output = match flag {
        "--emit-ir" => emit_llvm_ir(&program),
        "--emit-asm" => emit_assembly(&program),
        _ => Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "expected --emit-ir or --emit-asm",
        )),
    }
    .map_err(|error| error.with_file(&input))?;

    print!("{output}");
    Ok(0)
}

fn command_run(args: &[String]) -> CompileResult<u8> {
    if args.len() != 1 {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "usage: phpc run <input.php>",
        ));
    }

    let input = PathBuf::from(&args[0]);
    let source = read_source(&input)?;
    let program = parse_source(&source).map_err(|error| error.with_file(&input))?;
    let execution = run_program(&program).map_err(|error| error.with_file(&input))?;
    print!("{}", execution.stdout);
    eprint!("{}", execution.stderr);
    Ok(execution.exit_code as u8)
}

fn command_test(args: &[String]) -> CompileResult<u8> {
    let root = match args {
        [] => PathBuf::from("tests/fixtures"),
        [path] => PathBuf::from(path),
        _ => {
            return Err(Diagnostic::new(
                Phase::Cli,
                0,
                0,
                "usage: phpc test [fixture-dir]",
            ))
        }
    };

    let summary = run_fixture_dir(&root)?;
    for failure in &summary.failures {
        eprintln!("{failure}");
    }
    println!(
        "fixture tests: {} passed, {} failed",
        summary.passed, summary.failed
    );
    Ok(if summary.failed == 0 { 0 } else { 1 })
}

fn read_source(path: &Path) -> CompileResult<String> {
    fs::read_to_string(path).map_err(|error| {
        Diagnostic::new(
            Phase::Io,
            0,
            0,
            format!("failed to read {}: {error}", path.display()),
        )
    })
}

fn print_help() {
    println!("phpc");
    println!();
    println!("Commands:");
    println!("  phpc compile <input.php> --emit-ir");
    println!("  phpc compile <input.php> --emit-asm");
    println!("  phpc run <input.php>");
    println!("  phpc test [fixture-dir]");
}
