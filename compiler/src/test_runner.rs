use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::error::{CompileResult, Diagnostic, Phase};
use crate::run_source_with_source_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub php_compared: usize,
    pub php_skipped: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FixtureRunOptions {
    pub compare_php: bool,
}

pub fn run_fixture_dir(root: &Path) -> CompileResult<TestSummary> {
    run_fixture_dir_with_options(root, FixtureRunOptions::default())
}

pub fn run_fixture_dir_with_options(
    root: &Path,
    options: FixtureRunOptions,
) -> CompileResult<TestSummary> {
    let mut files = Vec::new();
    collect_php_files(root, &mut files)?;
    files.sort();

    let php_comparison = if options.compare_php {
        PhpComparison::for_system_php()
    } else {
        PhpComparison::Disabled
    };

    let mut summary = TestSummary {
        passed: 0,
        failed: 0,
        php_compared: 0,
        php_skipped: 0,
        failures: Vec::new(),
    };

    for path in files {
        let fixture_php_comparison = if php_comparison == PhpComparison::Enabled
            && path.with_extension("phpc-only").exists()
        {
            PhpComparison::SkippedByFixture
        } else {
            php_comparison
        };

        match fixture_php_comparison {
            PhpComparison::Enabled => summary.php_compared += 1,
            PhpComparison::Missing | PhpComparison::SkippedByFixture => summary.php_skipped += 1,
            PhpComparison::Disabled => {}
        }

        let outcome = run_fixture(&path, fixture_php_comparison);
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

pub fn system_php_available() -> bool {
    Command::new("php").arg("-v").output().is_ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhpComparison {
    Disabled,
    Enabled,
    Missing,
    SkippedByFixture,
}

impl PhpComparison {
    fn for_system_php() -> Self {
        if system_php_available() {
            Self::Enabled
        } else {
            Self::Missing
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

fn run_fixture(path: &Path, php_comparison: PhpComparison) -> Result<(), String> {
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

    let actual = match run_source_with_source_file(&source, fixture_source_name(path)) {
        Ok(execution) => FixtureOutput {
            stdout: execution.stdout,
            stderr: execution.stderr,
            exit_code: execution.exit_code,
        },
        Err(error) => FixtureOutput {
            stdout: String::new(),
            stderr: format!("{error}\n"),
            exit_code: 1,
        },
    };

    let mut differences = Vec::new();
    if actual.stdout != expected_stdout {
        differences.push(format!(
            "stdout mismatch\nexpected: {:?}\nactual:   {:?}",
            expected_stdout, actual.stdout
        ));
    }
    if actual.stderr != expected_stderr {
        differences.push(format!(
            "stderr mismatch\nexpected: {:?}\nactual:   {:?}",
            expected_stderr, actual.stderr
        ));
    }
    if actual.exit_code != expected_exit {
        differences.push(format!(
            "exit mismatch\nexpected: {expected_exit}\nactual:   {}",
            actual.exit_code
        ));
    }

    if php_comparison == PhpComparison::Enabled {
        let system_php = run_system_php(path)?;
        if actual.stdout != system_php.stdout {
            differences.push(format!(
                "system php stdout mismatch\nphp:  {:?}\nphpc: {:?}",
                system_php.stdout, actual.stdout
            ));
        }
        if actual.stderr != system_php.stderr {
            differences.push(format!(
                "system php stderr mismatch\nphp:  {:?}\nphpc: {:?}",
                system_php.stderr, actual.stderr
            ));
        }
        if actual.exit_code != system_php.exit_code {
            differences.push(format!(
                "system php exit mismatch\nphp:  {}\nphpc: {}",
                system_php.exit_code, actual.exit_code
            ));
        }
    }

    if differences.is_empty() {
        Ok(())
    } else {
        Err(format!("{}\n{}", path.display(), differences.join("\n")))
    }
}

fn fixture_source_name(path: &Path) -> String {
    let normalized = normalize_path_for_display(path);
    path_from_tests_fixtures(&normalized)
        .unwrap_or(normalized)
        .to_string_lossy()
        .into_owned()
}

fn normalize_path_for_display(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    normalized
}

fn path_from_tests_fixtures(path: &Path) -> Option<PathBuf> {
    let parts = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part),
            _ => None,
        })
        .collect::<Vec<_>>();

    let start = parts.windows(2).position(|window| {
        window[0] == OsStr::new("tests") && window[1] == OsStr::new("fixtures")
    })?;

    let mut relative = PathBuf::new();
    for part in &parts[start..] {
        relative.push(part);
    }
    Some(relative)
}

fn run_system_php(path: &Path) -> Result<FixtureOutput, String> {
    let output = Command::new("php")
        .arg(path)
        .output()
        .map_err(|error| format!("{}: failed to run system php: {error}", path.display()))?;

    Ok(FixtureOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(1),
    })
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
