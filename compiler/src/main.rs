use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use php_compiler::codegen::{emit_assembly, emit_llvm_ir};
use php_compiler::error::{CompileResult, Diagnostic, Phase};
use php_compiler::interpreter::{run_program_with_source_file_and_options, RunOptions};
use php_compiler::parser::parse_source;
use php_compiler::test_runner::{run_fixture_dir_with_options, FixtureRunOptions};

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
    let options = run_options_from_env()?;
    let execution =
        run_program_with_source_file_and_options(&program, input.display().to_string(), options)
            .map_err(|error| error.with_file(&input))?;
    print!("{}", execution.stdout);
    eprint!("{}", execution.stderr);
    Ok(execution.exit_code as u8)
}

fn run_options_from_env() -> CompileResult<RunOptions> {
    let max_execution_steps = match env::var("PHPC_MAX_EXECUTION_STEPS") {
        Ok(value) if !value.trim().is_empty() => {
            let parsed = value.trim().parse::<usize>().map_err(|_| {
                Diagnostic::new(
                    Phase::Cli,
                    0,
                    0,
                    "PHPC_MAX_EXECUTION_STEPS must be a non-negative integer",
                )
            })?;
            Some(parsed)
        }
        _ => None,
    };

    Ok(RunOptions {
        max_execution_steps,
    })
}

fn command_test(args: &[String]) -> CompileResult<u8> {
    let mut compare_php = false;
    let mut root = None;

    for arg in args {
        if arg == "--compare-php" {
            compare_php = true;
        } else if root.is_none() {
            root = Some(PathBuf::from(arg));
        } else {
            return Err(Diagnostic::new(
                Phase::Cli,
                0,
                0,
                "usage: phpc test [--compare-php] [fixture-dir]",
            ));
        }
    }

    let root = root.unwrap_or_else(|| PathBuf::from("tests/fixtures"));
    let summary = run_fixture_dir_with_options(&root, FixtureRunOptions { compare_php })?;

    for failure in &summary.failures {
        eprintln!("{failure}");
    }
    println!(
        "fixture tests: {} passed, {} failed",
        summary.passed, summary.failed
    );
    if compare_php {
        println!(
            "system php comparison: {} compared, {} skipped",
            summary.php_compared, summary.php_skipped
        );
    }
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
    println!("  phpc test [--compare-php] [fixture-dir]");
}
