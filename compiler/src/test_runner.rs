use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CompileResult, Diagnostic, Phase};
use crate::run_source;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

pub fn run_fixture_dir(root: &Path) -> CompileResult<TestSummary> {
    let mut files = Vec::new();
    collect_php_files(root, &mut files)?;
    files.sort();

    let mut summary = TestSummary {
        passed: 0,
        failed: 0,
        failures: Vec::new(),
    };

    for path in files {
        let outcome = run_fixture(&path);
        match outcome {
            Ok(()) => summary.passed += 1,
            Err(message) => {
                summary.failed += 1;
                summary.failures.push(message);
            }
        }
    }

    Ok(summary)
}

fn run_fixture(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("{}: failed to read source: {error}", path.display()))?;
    let expected_stdout =
        strip_fixture_editor_newline(read_optional(path.with_extension("stdout"))?);
    let expected_stderr =
        strip_fixture_editor_newline(read_optional(path.with_extension("stderr"))?);
    let expected_exit = read_optional(path.with_extension("exit"))?
        .trim()
        .parse::<i32>()
        .unwrap_or(0);

    let (actual_stdout, actual_stderr, actual_exit) = match run_source(&source) {
        Ok(execution) => (execution.stdout, execution.stderr, execution.exit_code),
        Err(error) => (String::new(), format!("{error}\n"), 1),
    };

    let mut differences = Vec::new();
    if actual_stdout != expected_stdout {
        differences.push(format!(
            "stdout mismatch\nexpected: {:?}\nactual:   {:?}",
            expected_stdout, actual_stdout
        ));
    }
    if actual_stderr != expected_stderr {
        differences.push(format!(
            "stderr mismatch\nexpected: {:?}\nactual:   {:?}",
            expected_stderr, actual_stderr
        ));
    }
    if actual_exit != expected_exit {
        differences.push(format!(
            "exit mismatch\nexpected: {expected_exit}\nactual:   {actual_exit}"
        ));
    }

    if differences.is_empty() {
        Ok(())
    } else {
        Err(format!("{}\n{}", path.display(), differences.join("\n")))
    }
}

fn collect_php_files(root: &Path, out: &mut Vec<PathBuf>) -> CompileResult<()> {
    let entries = fs::read_dir(root).map_err(|error| {
        Diagnostic::new(
            Phase::Test,
            0,
            0,
            format!("failed to read test directory {}: {error}", root.display()),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::new(
                Phase::Test,
                0,
                0,
                format!("failed to read test entry: {error}"),
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_php_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("php") {
            out.push(path);
        }
    }

    Ok(())
}

fn read_optional(path: PathBuf) -> Result<String, String> {
    match fs::read_to_string(&path) {
        Ok(value) => Ok(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!(
            "{}: failed to read fixture: {error}",
            path.display()
        )),
    }
}

fn strip_fixture_editor_newline(mut value: String) -> String {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    value
}
