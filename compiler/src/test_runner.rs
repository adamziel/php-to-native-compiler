use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::error::{CompileResult, Diagnostic, Phase};
use crate::run_source_with_source_file;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub php_compared: usize,
    pub php_skipped: usize,
    pub php_skipped_missing: usize,
    pub php_skipped_by_fixture: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifest {
    pub entries: Vec<FixtureManifestEntry>,
    pub summary: FixtureManifestSummary,
    pub orphan_sidecars: Vec<FixtureManifestOrphanSidecar>,
    pub compatibility_targets: Vec<FixtureManifestCompatibilityTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixtureManifestSummary {
    pub total: usize,
    pub php_comparison_eligible: usize,
    pub phpc_only: usize,
    pub stdout_expectations: usize,
    pub stderr_expectations: usize,
    pub exit_expectations: usize,
    pub cli_exercises: usize,
    pub phpc_only_markers: usize,
    pub orphan_sidecars: usize,
    pub source_bytes: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub exit_bytes: u64,
    pub cli_bytes: u64,
    pub phpc_only_bytes: u64,
    pub orphan_sidecar_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifestEntry {
    pub path: String,
    pub source_bytes: u64,
    pub source_sha256: String,
    pub has_stdout: bool,
    pub has_stderr: bool,
    pub has_exit: bool,
    pub has_cli: bool,
    pub phpc_only: bool,
    pub stdout_bytes: Option<u64>,
    pub stderr_bytes: Option<u64>,
    pub exit_bytes: Option<u64>,
    pub cli_bytes: Option<u64>,
    pub phpc_only_bytes: Option<u64>,
    pub stdout_sha256: Option<String>,
    pub stderr_sha256: Option<String>,
    pub exit_sha256: Option<String>,
    pub cli_sha256: Option<String>,
    pub phpc_only_sha256: Option<String>,
    pub phpc_only_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifestOrphanSidecar {
    pub path: String,
    pub kind: String,
    pub expected_fixture: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifestCompatibilityTarget {
    pub target: String,
    pub path: String,
    pub summary: FixtureManifestSummary,
    pub source_pin: Option<FixtureManifestCompatibilitySourcePin>,
    pub probe_expectations: Vec<FixtureManifestCompatibilityProbeExpectation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifestCompatibilitySourcePin {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifestCompatibilityProbeExpectation {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpVersionManifest {
    pub tracked_php_branches: Vec<String>,
    pub php_binaries: Vec<PhpVersionManifestEntry>,
    pub summary: PhpVersionManifestSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpVersionManifestEntry {
    pub command: String,
    pub available: bool,
    pub version: Option<String>,
    pub branch: Option<String>,
    pub tracked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpVersionManifestSummary {
    pub requested: usize,
    pub available: usize,
    pub tracked_available: usize,
    pub missing_tracked_branches: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FixtureRunOptions {
    pub compare_php: bool,
}

pub fn fixture_manifest(root: &Path) -> CompileResult<FixtureManifest> {
    let mut files = Vec::new();
    collect_php_files(root, &mut files)?;
    files.sort();

    let mut entries = Vec::new();
    for path in files {
        let phpc_only_reason = read_phpc_only_reason(&path)?;
        let stdout_bytes = optional_file_size(&path.with_extension("stdout"))?;
        let stderr_bytes = optional_file_size(&path.with_extension("stderr"))?;
        let exit_bytes = optional_file_size(&path.with_extension("exit"))?;
        let cli_bytes = optional_file_size(&path.with_extension("cli"))?;
        let phpc_only_bytes = optional_file_size(&path.with_extension("phpc-only"))?;
        entries.push(FixtureManifestEntry {
            path: fixture_manifest_path(root, &path),
            source_bytes: file_size(&path)?,
            source_sha256: file_sha256(&path)?,
            has_stdout: stdout_bytes.is_some(),
            has_stderr: stderr_bytes.is_some(),
            has_exit: exit_bytes.is_some(),
            has_cli: cli_bytes.is_some(),
            phpc_only: phpc_only_reason.is_some(),
            stdout_bytes,
            stderr_bytes,
            exit_bytes,
            cli_bytes,
            phpc_only_bytes,
            stdout_sha256: optional_file_sha256(&path.with_extension("stdout"))?,
            stderr_sha256: optional_file_sha256(&path.with_extension("stderr"))?,
            exit_sha256: optional_file_sha256(&path.with_extension("exit"))?,
            cli_sha256: optional_file_sha256(&path.with_extension("cli"))?,
            phpc_only_sha256: optional_file_sha256(&path.with_extension("phpc-only"))?,
            phpc_only_reason,
        });
    }
    let orphan_sidecars = collect_orphan_sidecars(root)?;
    let summary = FixtureManifestSummary::from_entries(&entries, &orphan_sidecars);
    let compatibility_targets = collect_compatibility_targets(root, &entries, &orphan_sidecars)?;

    Ok(FixtureManifest {
        entries,
        summary,
        orphan_sidecars,
        compatibility_targets,
    })
}

impl FixtureManifestSummary {
    fn from_entries(
        entries: &[FixtureManifestEntry],
        orphan_sidecars: &[FixtureManifestOrphanSidecar],
    ) -> Self {
        let mut summary = Self {
            total: entries.len(),
            orphan_sidecars: orphan_sidecars.len(),
            orphan_sidecar_bytes: orphan_sidecars.iter().map(|sidecar| sidecar.bytes).sum(),
            ..Self::default()
        };

        for entry in entries {
            if entry.phpc_only {
                summary.phpc_only += 1;
                summary.phpc_only_markers += 1;
            } else {
                summary.php_comparison_eligible += 1;
            }
            if entry.has_stdout {
                summary.stdout_expectations += 1;
            }
            if entry.has_stderr {
                summary.stderr_expectations += 1;
            }
            if entry.has_exit {
                summary.exit_expectations += 1;
            }
            if entry.has_cli {
                summary.cli_exercises += 1;
            }
            summary.source_bytes += entry.source_bytes;
            summary.stdout_bytes += entry.stdout_bytes.unwrap_or(0);
            summary.stderr_bytes += entry.stderr_bytes.unwrap_or(0);
            summary.exit_bytes += entry.exit_bytes.unwrap_or(0);
            summary.cli_bytes += entry.cli_bytes.unwrap_or(0);
            summary.phpc_only_bytes += entry.phpc_only_bytes.unwrap_or(0);
        }

        summary
    }
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
        php_skipped_missing: 0,
        php_skipped_by_fixture: 0,
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
            PhpComparison::Missing => {
                summary.php_skipped += 1;
                summary.php_skipped_missing += 1;
            }
            PhpComparison::SkippedByFixture => {
                summary.php_skipped += 1;
                summary.php_skipped_by_fixture += 1;
            }
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

pub fn php_version_manifest_from_env() -> PhpVersionManifest {
    let commands = configured_php_binary_commands();
    php_version_manifest_for_commands(commands)
}

pub fn php_version_manifest_for_commands(commands: Vec<String>) -> PhpVersionManifest {
    let tracked_php_branches = tracked_php_branches();
    let tracked_branch_set = tracked_php_branches
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    let php_binaries = commands
        .into_iter()
        .map(|command| {
            let version = php_binary_version(&command);
            let branch = version.as_deref().and_then(php_branch_from_version);
            let tracked = branch
                .as_ref()
                .map_or(false, |branch| tracked_branch_set.contains(branch));

            PhpVersionManifestEntry {
                command,
                available: version.is_some(),
                version,
                branch,
                tracked,
            }
        })
        .collect::<Vec<_>>();

    let available_tracked_branches = php_binaries
        .iter()
        .filter(|entry| entry.available && entry.tracked)
        .filter_map(|entry| entry.branch.clone())
        .collect::<BTreeSet<_>>();
    let missing_tracked_branches = tracked_php_branches
        .iter()
        .filter(|branch| !available_tracked_branches.contains(*branch))
        .cloned()
        .collect::<Vec<_>>();

    let summary = PhpVersionManifestSummary {
        requested: php_binaries.len(),
        available: php_binaries.iter().filter(|entry| entry.available).count(),
        tracked_available: available_tracked_branches.len(),
        missing_tracked_branches,
    };

    PhpVersionManifest {
        tracked_php_branches,
        php_binaries,
        summary,
    }
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

fn fixture_manifest_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    normalize_path_for_display(relative)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
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

fn configured_php_binary_commands() -> Vec<String> {
    let commands = env::var("PHPC_PHP_BINARIES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|command| !command.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|commands| !commands.is_empty())
        .unwrap_or_else(|| vec!["php".to_string()]);

    commands
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn tracked_php_branches() -> Vec<String> {
    ["8.2", "8.3", "8.4", "8.5"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn php_binary_version(command: &str) -> Option<String> {
    let output = Command::new(command)
        .args(["-r", "echo PHP_VERSION;"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn php_branch_from_version(version: &str) -> Option<String> {
    let mut parts = version.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    if major.chars().all(|character| character.is_ascii_digit())
        && minor.chars().all(|character| character.is_ascii_digit())
    {
        Some(format!("{major}.{minor}"))
    } else {
        None
    }
}

fn collect_orphan_sidecars(root: &Path) -> CompileResult<Vec<FixtureManifestOrphanSidecar>> {
    let mut sidecars = Vec::new();
    collect_sidecar_files(root, &mut sidecars)?;

    let mut orphans = Vec::new();
    for path in sidecars {
        let Some(kind) = recognized_sidecar_kind(&path) else {
            continue;
        };
        let expected_fixture = path.with_extension("php");
        if expected_fixture.exists() {
            continue;
        }

        orphans.push(FixtureManifestOrphanSidecar {
            path: fixture_manifest_path(root, &path),
            kind: kind.to_string(),
            expected_fixture: fixture_manifest_path(root, &expected_fixture),
            bytes: file_size(&path)?,
            sha256: file_sha256(&path)?,
        });
    }
    orphans.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.expected_fixture.cmp(&right.expected_fixture))
    });

    Ok(orphans)
}

fn collect_compatibility_targets(
    root: &Path,
    entries: &[FixtureManifestEntry],
    orphan_sidecars: &[FixtureManifestOrphanSidecar],
) -> CompileResult<Vec<FixtureManifestCompatibilityTarget>> {
    let mut targets = BTreeSet::new();
    let compat_dir = root.join("compat");
    if compat_dir.is_dir() {
        let entries = fs::read_dir(&compat_dir).map_err(|error| {
            Diagnostic::new(
                Phase::Test,
                0,
                0,
                format!(
                    "failed to read compatibility fixture directory {}: {error}",
                    compat_dir.display()
                ),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|error| {
                Diagnostic::new(
                    Phase::Test,
                    0,
                    0,
                    format!("failed to read compatibility fixture entry: {error}"),
                )
            })?;
            if entry.path().is_dir() {
                targets.insert(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }

    for entry in entries {
        if let Some(target) = compatibility_target_from_manifest_path(&entry.path) {
            targets.insert(target.to_string());
        }
    }
    for orphan in orphan_sidecars {
        if let Some(target) = compatibility_target_from_manifest_path(&orphan.path) {
            targets.insert(target.to_string());
        }
    }

    targets
        .into_iter()
        .map(|target| {
            let prefix = format!("compat/{target}/");
            let target_path = root.join("compat").join(&target);
            let source_pin_path = target_path.join("source-pin.md");
            let target_entries = entries
                .iter()
                .filter(|entry| entry.path.starts_with(&prefix))
                .cloned()
                .collect::<Vec<_>>();
            let target_orphans = orphan_sidecars
                .iter()
                .filter(|orphan| orphan.path.starts_with(&prefix))
                .cloned()
                .collect::<Vec<_>>();
            let summary = FixtureManifestSummary::from_entries(&target_entries, &target_orphans);

            Ok(FixtureManifestCompatibilityTarget {
                path: format!("compat/{target}"),
                target,
                summary,
                source_pin: compatibility_target_source_pin(root, &source_pin_path)?,
                probe_expectations: collect_compatibility_probe_expectations(root, &target_path)?,
            })
        })
        .collect::<CompileResult<Vec<_>>>()
}

fn compatibility_target_source_pin(
    root: &Path,
    path: &Path,
) -> CompileResult<Option<FixtureManifestCompatibilitySourcePin>> {
    match (optional_file_size(path)?, optional_file_sha256(path)?) {
        (Some(bytes), Some(sha256)) => Ok(Some(FixtureManifestCompatibilitySourcePin {
            path: fixture_manifest_path(root, path),
            bytes,
            sha256,
        })),
        _ => Ok(None),
    }
}

fn collect_compatibility_probe_expectations(
    root: &Path,
    target_path: &Path,
) -> CompileResult<Vec<FixtureManifestCompatibilityProbeExpectation>> {
    let mut files = Vec::new();
    collect_expected_files(target_path, &mut files)?;
    files.sort();

    files
        .into_iter()
        .map(|path| {
            Ok(FixtureManifestCompatibilityProbeExpectation {
                path: fixture_manifest_path(root, &path),
                bytes: file_size(&path)?,
                sha256: file_sha256(&path)?,
            })
        })
        .collect::<CompileResult<Vec<_>>>()
}

fn collect_expected_files(root: &Path, out: &mut Vec<PathBuf>) -> CompileResult<()> {
    let entries = fs::read_dir(root).map_err(|error| {
        Diagnostic::new(
            Phase::Test,
            0,
            0,
            format!(
                "failed to read compatibility expectation directory {}: {error}",
                root.display()
            ),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::new(
                Phase::Test,
                0,
                0,
                format!("failed to read compatibility expectation entry: {error}"),
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_expected_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("expected") {
            out.push(path);
        }
    }

    Ok(())
}

fn compatibility_target_from_manifest_path(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("compat/")?;
    let (target, _) = rest.split_once('/')?;
    if target.is_empty() {
        None
    } else {
        Some(target)
    }
}

fn collect_sidecar_files(root: &Path, out: &mut Vec<PathBuf>) -> CompileResult<()> {
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
            collect_sidecar_files(&path, out)?;
        } else if recognized_sidecar_kind(&path).is_some() {
            out.push(path);
        }
    }

    Ok(())
}

fn recognized_sidecar_kind(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("stdout") => Some("stdout"),
        Some("stderr") => Some("stderr"),
        Some("exit") => Some("exit"),
        Some("cli") => Some("cli"),
        Some("phpc-only") => Some("phpc-only"),
        _ => None,
    }
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

fn optional_file_size(path: &Path) -> CompileResult<Option<u64>> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata.len())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Diagnostic::new(
            Phase::Test,
            0,
            0,
            format!("failed to stat fixture sidecar {}: {error}", path.display()),
        )),
    }
}

fn file_size(path: &Path) -> CompileResult<u64> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|error| {
            Diagnostic::new(
                Phase::Test,
                0,
                0,
                format!("failed to stat fixture file {}: {error}", path.display()),
            )
        })
}

fn optional_file_sha256(path: &Path) -> CompileResult<Option<String>> {
    match fs::read(path) {
        Ok(contents) => Ok(Some(sha256_hex(&contents))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Diagnostic::new(
            Phase::Test,
            0,
            0,
            format!("failed to read fixture sidecar {}: {error}", path.display()),
        )),
    }
}

fn file_sha256(path: &Path) -> CompileResult<String> {
    fs::read(path)
        .map(|contents| sha256_hex(&contents))
        .map_err(|error| {
            Diagnostic::new(
                Phase::Test,
                0,
                0,
                format!("failed to read fixture file {}: {error}", path.display()),
            )
        })
}

fn sha256_hex(contents: &[u8]) -> String {
    let digest = Sha256::digest(contents);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn read_phpc_only_reason(path: &Path) -> CompileResult<Option<String>> {
    let marker = path.with_extension("phpc-only");
    match fs::read_to_string(&marker) {
        Ok(value) => Ok(Some(strip_fixture_editor_newline(value))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Diagnostic::new(
            Phase::Test,
            0,
            0,
            format!(
                "failed to read phpc-only marker {}: {error}",
                marker.display()
            ),
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
