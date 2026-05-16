use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use php_compiler::codegen::{emit_assembly, emit_llvm_ir};
use php_compiler::error::{CompileResult, Diagnostic, Phase};
use php_compiler::interpreter::{run_program_with_source_file_and_options, RunOptions};
use php_compiler::parser::parse_source;
use php_compiler::test_runner::{
    fixture_manifest, run_fixture_dir_with_options, FixtureManifestCompatibilityTarget,
    FixtureManifestEntry, FixtureManifestOrphanSidecar, FixtureManifestSummary, FixtureRunOptions,
    PhpVersionManifest, PhpVersionManifestEntry, TestSummary,
};

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
    if flag != "--emit-ir" && flag != "--emit-asm" {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "expected --emit-ir or --emit-asm",
        ));
    }

    let source = read_source(&input)?;
    let program = parse_source(&source).map_err(|error| error.with_file(&input))?;

    let output = match flag {
        "--emit-ir" => emit_llvm_ir(&program),
        "--emit-asm" => emit_assembly(&program),
        _ => unreachable!("compile mode was validated before reading input"),
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
        trace_includes: env::var_os("PHPC_TRACE_INCLUDES").is_some(),
    })
}

fn command_test(args: &[String]) -> CompileResult<u8> {
    let mut compare_php = false;
    let mut compare_php_json = false;
    let mut list_fixtures = false;
    let mut list_fixtures_json = false;
    let mut php_versions_json = false;
    let mut root = None;

    for arg in args {
        if arg == "--compare-php" {
            compare_php = true;
        } else if arg == "--compare-php-json" {
            compare_php_json = true;
        } else if arg == "--list-fixtures" {
            list_fixtures = true;
        } else if arg == "--list-fixtures-json" {
            list_fixtures_json = true;
        } else if arg == "--php-versions-json" {
            php_versions_json = true;
        } else if root.is_none() {
            root = Some(PathBuf::from(arg));
        } else {
            return Err(Diagnostic::new(
                Phase::Cli,
                0,
                0,
                "usage: phpc test [--compare-php] [--list-fixtures | --list-fixtures-json] [fixture-dir] | phpc test --compare-php-json [fixture-dir] | phpc test --php-versions-json",
            ));
        }
    }

    if compare_php && compare_php_json {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "expected at most one PHP comparison output mode",
        ));
    }
    if list_fixtures && list_fixtures_json {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "expected at most one fixture manifest output mode",
        ));
    }
    if compare_php_json && (list_fixtures || list_fixtures_json) {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "expected PHP comparison JSON without fixture manifest output mode",
        ));
    }
    if php_versions_json
        && (list_fixtures
            || list_fixtures_json
            || compare_php
            || compare_php_json
            || root.is_some())
    {
        return Err(Diagnostic::new(
            Phase::Cli,
            0,
            0,
            "usage: phpc test --php-versions-json",
        ));
    }

    if php_versions_json {
        let manifest = php_compiler::test_runner::php_version_manifest_from_env();
        print!("{}", render_php_version_manifest_json(&manifest));
        return Ok(0);
    }

    let root = root.unwrap_or_else(|| PathBuf::from("tests/fixtures"));
    if list_fixtures {
        let manifest = fixture_manifest(&root)?;
        println!("fixture manifest: {} fixtures", manifest.summary.total);
        println!("{}", render_fixture_manifest_summary(&manifest.summary));
        for entry in &manifest.entries {
            println!("{}", render_fixture_manifest_entry(entry));
        }
        for orphan in &manifest.orphan_sidecars {
            println!("{}", render_fixture_manifest_orphan_sidecar(orphan));
        }
        for target in &manifest.compatibility_targets {
            println!("{}", render_fixture_manifest_compatibility_target(target));
        }
        return Ok(0);
    }
    if list_fixtures_json {
        let manifest = fixture_manifest(&root)?;
        print!("{}", render_fixture_manifest_json(&manifest));
        return Ok(0);
    }

    let summary = run_fixture_dir_with_options(
        &root,
        FixtureRunOptions {
            compare_php: compare_php || compare_php_json,
        },
    )?;

    if compare_php_json {
        print!("{}", render_php_comparison_summary_json(&summary));
        return Ok(if summary.failed == 0 { 0 } else { 1 });
    }

    for failure in &summary.failures {
        eprintln!("{failure}");
    }
    println!(
        "fixture tests: {} passed, {} failed",
        summary.passed, summary.failed
    );
    if compare_php {
        println!(
            "system php comparison: {} compared, {} skipped ({} missing php, {} phpc-only)",
            summary.php_compared,
            summary.php_skipped,
            summary.php_skipped_missing,
            summary.php_skipped_by_fixture
        );
    }
    Ok(if summary.failed == 0 { 0 } else { 1 })
}

fn render_php_comparison_summary_json(summary: &TestSummary) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"contract_version\": 1,\n");
    output.push_str("  \"summary\": {\n");
    output.push_str(&format!(
        "    \"fixtures\": {{ \"passed\": {}, \"failed\": {}, \"total\": {} }},\n",
        summary.passed,
        summary.failed,
        summary.passed + summary.failed
    ));
    output.push_str("    \"php_comparison\": {\n");
    output.push_str(&format!("      \"compared\": {},\n", summary.php_compared));
    output.push_str(&format!("      \"skipped\": {},\n", summary.php_skipped));
    output.push_str(&format!(
        "      \"missing_system_php\": {},\n",
        summary.php_skipped_missing
    ));
    output.push_str(&format!(
        "      \"phpc_only\": {}\n",
        summary.php_skipped_by_fixture
    ));
    output.push_str("    }\n");
    output.push_str("  },\n");
    output.push_str("  \"failures\": [\n");
    for (index, failure) in summary.failures.iter().enumerate() {
        output.push_str(&format!("    {}", json_string_literal(failure)));
        if index + 1 < summary.failures.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ]\n");
    output.push_str("}\n");
    output
}

fn render_fixture_manifest_summary(summary: &FixtureManifestSummary) -> String {
    format!(
        "summary: php-comparison eligible={}, phpc-only={} expectations stdout={}, stderr={}, exit={}, phpc-only={} orphan sidecars={}",
        summary.php_comparison_eligible,
        summary.phpc_only,
        summary.stdout_expectations,
        summary.stderr_expectations,
        summary.exit_expectations,
        summary.phpc_only_markers,
        summary.orphan_sidecars
    )
}

fn render_fixture_manifest_entry(entry: &FixtureManifestEntry) -> String {
    let mut expectations = Vec::new();
    if entry.has_stdout {
        expectations.push("stdout");
    }
    if entry.has_stderr {
        expectations.push("stderr");
    }
    if entry.has_exit {
        expectations.push("exit");
    }

    let expectations = if expectations.is_empty() {
        "none".to_string()
    } else {
        expectations.join(",")
    };
    let comparison = if entry.phpc_only {
        "phpc-only"
    } else {
        "eligible"
    };

    format!(
        "{} expectations={} php-comparison={}",
        entry.path, expectations, comparison
    )
}

fn render_fixture_manifest_orphan_sidecar(orphan: &FixtureManifestOrphanSidecar) -> String {
    format!(
        "orphan sidecar: {} kind={} expected-fixture={}",
        orphan.path, orphan.kind, orphan.expected_fixture
    )
}

fn render_fixture_manifest_compatibility_target(
    target: &FixtureManifestCompatibilityTarget,
) -> String {
    format!(
        "compatibility target: {} path={} fixtures={} php-comparison eligible={} phpc-only={} expectations stdout={}, stderr={}, exit={}, phpc-only={} orphan sidecars={}",
        target.target,
        target.path,
        target.summary.total,
        target.summary.php_comparison_eligible,
        target.summary.phpc_only,
        target.summary.stdout_expectations,
        target.summary.stderr_expectations,
        target.summary.exit_expectations,
        target.summary.phpc_only_markers,
        target.summary.orphan_sidecars
    )
}

fn render_fixture_manifest_json(manifest: &php_compiler::test_runner::FixtureManifest) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"contract_version\": 2,\n");
    output.push_str(&format!(
        "  \"fixture_count\": {},\n",
        manifest.summary.total
    ));
    output.push_str("  \"summary\": {\n");
    output.push_str(&format!("    \"total\": {},\n", manifest.summary.total));
    output.push_str(&format!(
        "    \"php_comparison_eligible\": {},\n",
        manifest.summary.php_comparison_eligible
    ));
    output.push_str(&format!(
        "    \"phpc_only\": {},\n",
        manifest.summary.phpc_only
    ));
    output.push_str("    \"expectations\": {\n");
    output.push_str(&format!(
        "      \"stdout\": {},\n",
        manifest.summary.stdout_expectations
    ));
    output.push_str(&format!(
        "      \"stderr\": {},\n",
        manifest.summary.stderr_expectations
    ));
    output.push_str(&format!(
        "      \"exit\": {},\n",
        manifest.summary.exit_expectations
    ));
    output.push_str(&format!(
        "      \"phpc_only\": {}\n",
        manifest.summary.phpc_only_markers
    ));
    output.push_str("    },\n");
    output.push_str(&format!(
        "    \"orphan_sidecars\": {}\n",
        manifest.summary.orphan_sidecars
    ));
    output.push_str("  },\n");
    output.push_str("  \"fixtures\": [\n");
    for (index, entry) in manifest.entries.iter().enumerate() {
        output.push_str("    {\n");
        output.push_str(&format!(
            "      \"path\": {},\n",
            json_string_literal(&entry.path)
        ));
        output.push_str("      \"expectations\": [");
        let expectations = fixture_manifest_entry_expectations(entry);
        for (expectation_index, expectation) in expectations.iter().enumerate() {
            if expectation_index > 0 {
                output.push_str(", ");
            }
            output.push_str(&json_string_literal(expectation));
        }
        output.push_str("],\n");
        let comparison = if entry.phpc_only {
            "phpc-only"
        } else {
            "eligible"
        };
        output.push_str(&format!(
            "      \"php_comparison\": {}\n",
            json_string_literal(comparison)
        ));
        output.push_str("    }");
        if index + 1 < manifest.entries.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ],\n");
    output.push_str("  \"compatibility_targets\": [\n");
    for (index, target) in manifest.compatibility_targets.iter().enumerate() {
        output.push_str("    {\n");
        output.push_str(&format!(
            "      \"target\": {},\n",
            json_string_literal(&target.target)
        ));
        output.push_str(&format!(
            "      \"path\": {},\n",
            json_string_literal(&target.path)
        ));
        output.push_str("      \"summary\": {\n");
        output.push_str(&format!("        \"total\": {},\n", target.summary.total));
        output.push_str(&format!(
            "        \"php_comparison_eligible\": {},\n",
            target.summary.php_comparison_eligible
        ));
        output.push_str(&format!(
            "        \"phpc_only\": {},\n",
            target.summary.phpc_only
        ));
        output.push_str("        \"expectations\": {\n");
        output.push_str(&format!(
            "          \"stdout\": {},\n",
            target.summary.stdout_expectations
        ));
        output.push_str(&format!(
            "          \"stderr\": {},\n",
            target.summary.stderr_expectations
        ));
        output.push_str(&format!(
            "          \"exit\": {},\n",
            target.summary.exit_expectations
        ));
        output.push_str(&format!(
            "          \"phpc_only\": {}\n",
            target.summary.phpc_only_markers
        ));
        output.push_str("        },\n");
        output.push_str(&format!(
            "        \"orphan_sidecars\": {}\n",
            target.summary.orphan_sidecars
        ));
        output.push_str("      }\n");
        output.push_str("    }");
        if index + 1 < manifest.compatibility_targets.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ],\n");
    output.push_str("  \"orphan_sidecars\": [\n");
    for (index, orphan) in manifest.orphan_sidecars.iter().enumerate() {
        output.push_str("    {\n");
        output.push_str(&format!(
            "      \"path\": {},\n",
            json_string_literal(&orphan.path)
        ));
        output.push_str(&format!(
            "      \"kind\": {},\n",
            json_string_literal(&orphan.kind)
        ));
        output.push_str(&format!(
            "      \"expected_fixture\": {}\n",
            json_string_literal(&orphan.expected_fixture)
        ));
        output.push_str("    }");
        if index + 1 < manifest.orphan_sidecars.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ]\n");
    output.push_str("}\n");
    output
}

fn render_php_version_manifest_json(manifest: &PhpVersionManifest) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"contract_version\": 1,\n");
    output.push_str("  \"tracked_php_branches\": [");
    for (index, branch) in manifest.tracked_php_branches.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&json_string_literal(branch));
    }
    output.push_str("],\n");
    output.push_str("  \"summary\": {\n");
    output.push_str(&format!(
        "    \"requested\": {},\n",
        manifest.summary.requested
    ));
    output.push_str(&format!(
        "    \"available\": {},\n",
        manifest.summary.available
    ));
    output.push_str(&format!(
        "    \"tracked_available\": {},\n",
        manifest.summary.tracked_available
    ));
    output.push_str("    \"missing_tracked_branches\": [");
    for (index, branch) in manifest.summary.missing_tracked_branches.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&json_string_literal(branch));
    }
    output.push_str("]\n");
    output.push_str("  },\n");
    output.push_str("  \"php_binaries\": [\n");
    for (index, entry) in manifest.php_binaries.iter().enumerate() {
        output.push_str(&render_php_version_manifest_entry_json(entry));
        if index + 1 < manifest.php_binaries.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ]\n");
    output.push_str("}\n");
    output
}

fn render_php_version_manifest_entry_json(entry: &PhpVersionManifestEntry) -> String {
    let mut output = String::new();
    output.push_str("    {\n");
    output.push_str(&format!(
        "      \"command\": {},\n",
        json_string_literal(&entry.command)
    ));
    output.push_str(&format!("      \"available\": {},\n", entry.available));
    output.push_str(&format!(
        "      \"version\": {},\n",
        json_optional_string_literal(entry.version.as_deref())
    ));
    output.push_str(&format!(
        "      \"branch\": {},\n",
        json_optional_string_literal(entry.branch.as_deref())
    ));
    output.push_str(&format!("      \"tracked\": {}\n", entry.tracked));
    output.push_str("    }");
    output
}

fn fixture_manifest_entry_expectations(entry: &FixtureManifestEntry) -> Vec<&'static str> {
    let mut expectations = Vec::new();
    if entry.has_stdout {
        expectations.push("stdout");
    }
    if entry.has_stderr {
        expectations.push("stderr");
    }
    if entry.has_exit {
        expectations.push("exit");
    }
    expectations
}

fn json_optional_string_literal(value: Option<&str>) -> String {
    value
        .map(json_string_literal)
        .unwrap_or_else(|| "null".to_string())
}

fn json_string_literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character <= '\u{1f}' => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
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
    println!("  phpc test [--compare-php] [--list-fixtures | --list-fixtures-json] [fixture-dir]");
    println!("  phpc test --compare-php-json [fixture-dir]");
    println!("  phpc test --php-versions-json");
}
