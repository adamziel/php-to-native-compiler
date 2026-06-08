use std::path::PathBuf;

use ptn::{compile_file, CompileOptions};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("ptn: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage());
    };
    match command.as_str() {
        "compile" => {
            let input = args.next().ok_or_else(usage)?;
            let mut output = None;
            let mut emit_c = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-o" | "--output" => output = args.next().map(PathBuf::from),
                    "--emit-c" => emit_c = true,
                    _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
                }
            }
            let output = output.ok_or_else(|| format!("missing -o/--output\n{}", usage()))?;
            compile_file(&PathBuf::from(input), &output, CompileOptions { emit_c })
                .map_err(|error| error.to_string())?;
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: ptn compile <input.php> -o <native-binary> [--emit-c]".to_string()
}
