use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use php_compiler::codegen::{emit_assembly, emit_llvm_ir};
use php_compiler::error::{CompileResult, Diagnostic, Phase};
use php_compiler::interpreter::{run_program_with_source_file_and_options, RunOptions};
use php_compiler::parser::parse_source;
use php_compiler::test_runner::{
    fixture_manifest, run_fixture_dir_with_options, FixtureManifestCompatibilityProbeExpectation,
    FixtureManifestCompatibilityTarget, FixtureManifestEntry, FixtureManifestOrphanSidecar,
    FixtureManifestSummary, FixtureRunOptions, PhpVersionManifest, PhpVersionManifestEntry,
    TestSummary,
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
        for sidecar in &manifest.unrecognized_sidecars {
            println!("{}", render_fixture_manifest_unrecognized_sidecar(sidecar));
        }
        for target in &manifest.compatibility_targets {
            println!("{}", render_fixture_manifest_compatibility_target(target));
            for probe_expectation in &target.probe_expectations {
                println!(
                    "{}",
                    render_fixture_manifest_compatibility_probe_expectation(probe_expectation)
                );
            }
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
        "summary: php-comparison eligible={}, phpc-only={} expectations stdout={}, stderr={}, exit={}, phpc-only={} phpc-only-reason-gaps={} cli-exercises={} cli-exercise-gaps={} orphan sidecars={} unrecognized sidecars={} bytes source={} stdout={} stderr={} exit={} cli={} phpc-only={} orphan-sidecars={} unrecognized-sidecars={}",
        summary.php_comparison_eligible,
        summary.phpc_only,
        summary.stdout_expectations,
        summary.stderr_expectations,
        summary.exit_expectations,
        summary.phpc_only_markers,
        summary.phpc_only_reason_gaps,
        summary.cli_exercises,
        summary.cli_exercise_gaps,
        summary.orphan_sidecars,
        summary.unrecognized_sidecars,
        summary.source_bytes,
        summary.stdout_bytes,
        summary.stderr_bytes,
        summary.exit_bytes,
        summary.cli_bytes,
        summary.phpc_only_bytes,
        summary.orphan_sidecar_bytes,
        summary.unrecognized_sidecar_bytes
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

    let reason = entry
        .phpc_only_reason
        .as_ref()
        .map(|reason| format!(" phpc-only-reason={}", text_field_value(reason)))
        .unwrap_or_default();

    format!(
        "{} expectations={} cli-exercise={} php-comparison={}{} bytes source={} stdout={} stderr={} exit={} cli={} phpc-only={}",
        entry.path,
        expectations,
        if entry.has_cli { "yes" } else { "no" },
        comparison,
        reason,
        entry.source_bytes,
        text_optional_u64(entry.stdout_bytes),
        text_optional_u64(entry.stderr_bytes),
        text_optional_u64(entry.exit_bytes),
        text_optional_u64(entry.cli_bytes),
        text_optional_u64(entry.phpc_only_bytes)
    )
}

fn render_fixture_manifest_orphan_sidecar(orphan: &FixtureManifestOrphanSidecar) -> String {
    format!(
        "orphan sidecar: {} kind={} expected-fixture={} bytes={} sha256={}",
        orphan.path, orphan.kind, orphan.expected_fixture, orphan.bytes, orphan.sha256
    )
}

fn render_fixture_manifest_unrecognized_sidecar(
    sidecar: &php_compiler::test_runner::FixtureManifestUnrecognizedSidecar,
) -> String {
    format!(
        "unrecognized sidecar: {} extension={} expected-fixture={} bytes={} sha256={}",
        sidecar.path, sidecar.extension, sidecar.expected_fixture, sidecar.bytes, sidecar.sha256
    )
}

fn render_fixture_manifest_compatibility_target(
    target: &FixtureManifestCompatibilityTarget,
) -> String {
    let source_pin = target
        .source_pin
        .as_ref()
        .map(|source_pin| {
            format!(
                " source-pin path={} bytes={} sha256={}",
                source_pin.path, source_pin.bytes, source_pin.sha256
            )
        })
        .unwrap_or_else(|| " source-pin path=- bytes=- sha256=-".to_string());
    let probe_expectation_bytes = target
        .probe_expectations
        .iter()
        .map(|probe_expectation| probe_expectation.bytes)
        .sum::<u64>();

    format!(
        "compatibility target: {} path={} fixtures={} php-comparison eligible={} phpc-only={} expectations stdout={}, stderr={}, exit={}, phpc-only={} phpc-only-reason-gaps={} cli-exercises={} cli-exercise-gaps={} orphan sidecars={} unrecognized sidecars={} bytes source={} stdout={} stderr={} exit={} cli={} phpc-only={} orphan-sidecars={} unrecognized-sidecars={} probe expectations={} bytes={}{}",
        target.target,
        target.path,
        target.summary.total,
        target.summary.php_comparison_eligible,
        target.summary.phpc_only,
        target.summary.stdout_expectations,
        target.summary.stderr_expectations,
        target.summary.exit_expectations,
        target.summary.phpc_only_markers,
        target.summary.phpc_only_reason_gaps,
        target.summary.cli_exercises,
        target.summary.cli_exercise_gaps,
        target.summary.orphan_sidecars,
        target.summary.unrecognized_sidecars,
        target.summary.source_bytes,
        target.summary.stdout_bytes,
        target.summary.stderr_bytes,
        target.summary.exit_bytes,
        target.summary.cli_bytes,
        target.summary.phpc_only_bytes,
        target.summary.orphan_sidecar_bytes,
        target.summary.unrecognized_sidecar_bytes,
        target.probe_expectations.len(),
        probe_expectation_bytes,
        source_pin
    )
}

fn render_fixture_manifest_compatibility_probe_expectation(
    probe_expectation: &FixtureManifestCompatibilityProbeExpectation,
) -> String {
    format!(
        "compatibility probe expectation: {} bytes={} sha256={}",
        probe_expectation.path, probe_expectation.bytes, probe_expectation.sha256
    )
}

fn render_fixture_manifest_json(manifest: &php_compiler::test_runner::FixtureManifest) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"contract_version\": 12,\n");
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
        "    \"cli_exercises\": {},\n",
        manifest.summary.cli_exercises
    ));
    output.push_str(&format!(
        "    \"cli_exercise_gaps\": {},\n",
        manifest.summary.cli_exercise_gaps
    ));
    output.push_str(&format!(
        "    \"phpc_only_reason_gaps\": {},\n",
        manifest.summary.phpc_only_reason_gaps
    ));
    output.push_str(&format!(
        "    \"orphan_sidecars\": {},\n",
        manifest.summary.orphan_sidecars
    ));
    output.push_str(&format!(
        "    \"unrecognized_sidecars\": {},\n",
        manifest.summary.unrecognized_sidecars
    ));
    output.push_str("    \"file_bytes\": {\n");
    output.push_str(&format!(
        "      \"source\": {},\n",
        manifest.summary.source_bytes
    ));
    output.push_str(&format!(
        "      \"stdout\": {},\n",
        manifest.summary.stdout_bytes
    ));
    output.push_str(&format!(
        "      \"stderr\": {},\n",
        manifest.summary.stderr_bytes
    ));
    output.push_str(&format!(
        "      \"exit\": {},\n",
        manifest.summary.exit_bytes
    ));
    output.push_str(&format!("      \"cli\": {},\n", manifest.summary.cli_bytes));
    output.push_str(&format!(
        "      \"phpc_only\": {}\n",
        manifest.summary.phpc_only_bytes
    ));
    output.push_str("    },\n");
    output.push_str(&format!(
        "    \"orphan_sidecar_bytes\": {},\n",
        manifest.summary.orphan_sidecar_bytes
    ));
    output.push_str(&format!(
        "    \"unrecognized_sidecar_bytes\": {}\n",
        manifest.summary.unrecognized_sidecar_bytes
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
        output.push_str("      \"file_bytes\": {\n");
        output.push_str(&format!("        \"source\": {},\n", entry.source_bytes));
        output.push_str(&format!(
            "        \"stdout\": {},\n",
            json_optional_u64(entry.stdout_bytes)
        ));
        output.push_str(&format!(
            "        \"stderr\": {},\n",
            json_optional_u64(entry.stderr_bytes)
        ));
        output.push_str(&format!(
            "        \"exit\": {},\n",
            json_optional_u64(entry.exit_bytes)
        ));
        output.push_str(&format!(
            "        \"cli\": {},\n",
            json_optional_u64(entry.cli_bytes)
        ));
        output.push_str(&format!(
            "        \"phpc_only\": {}\n",
            json_optional_u64(entry.phpc_only_bytes)
        ));
        output.push_str("      },\n");
        output.push_str("      \"file_sha256\": {\n");
        output.push_str(&format!(
            "        \"source\": {},\n",
            json_string_literal(&entry.source_sha256)
        ));
        output.push_str(&format!(
            "        \"stdout\": {},\n",
            json_optional_string_literal(entry.stdout_sha256.as_deref())
        ));
        output.push_str(&format!(
            "        \"stderr\": {},\n",
            json_optional_string_literal(entry.stderr_sha256.as_deref())
        ));
        output.push_str(&format!(
            "        \"exit\": {},\n",
            json_optional_string_literal(entry.exit_sha256.as_deref())
        ));
        output.push_str(&format!(
            "        \"cli\": {},\n",
            json_optional_string_literal(entry.cli_sha256.as_deref())
        ));
        output.push_str(&format!(
            "        \"phpc_only\": {}\n",
            json_optional_string_literal(entry.phpc_only_sha256.as_deref())
        ));
        output.push_str("      },\n");
        let comparison = if entry.phpc_only {
            "phpc-only"
        } else {
            "eligible"
        };
        output.push_str(&format!(
            "      \"php_comparison\": {},\n",
            json_string_literal(comparison)
        ));
        output.push_str("      \"phpc_only_reason\": ");
        match &entry.phpc_only_reason {
            Some(reason) => output.push_str(&json_string_literal(reason)),
            None => output.push_str("null"),
        }
        output.push('\n');
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
            "        \"cli_exercises\": {},\n",
            target.summary.cli_exercises
        ));
        output.push_str(&format!(
            "        \"cli_exercise_gaps\": {},\n",
            target.summary.cli_exercise_gaps
        ));
        output.push_str(&format!(
            "        \"phpc_only_reason_gaps\": {},\n",
            target.summary.phpc_only_reason_gaps
        ));
        output.push_str(&format!(
            "        \"orphan_sidecars\": {},\n",
            target.summary.orphan_sidecars
        ));
        output.push_str(&format!(
            "        \"unrecognized_sidecars\": {},\n",
            target.summary.unrecognized_sidecars
        ));
        output.push_str("        \"file_bytes\": {\n");
        output.push_str(&format!(
            "          \"source\": {},\n",
            target.summary.source_bytes
        ));
        output.push_str(&format!(
            "          \"stdout\": {},\n",
            target.summary.stdout_bytes
        ));
        output.push_str(&format!(
            "          \"stderr\": {},\n",
            target.summary.stderr_bytes
        ));
        output.push_str(&format!(
            "          \"exit\": {},\n",
            target.summary.exit_bytes
        ));
        output.push_str(&format!(
            "          \"cli\": {},\n",
            target.summary.cli_bytes
        ));
        output.push_str(&format!(
            "          \"phpc_only\": {}\n",
            target.summary.phpc_only_bytes
        ));
        output.push_str("        },\n");
        output.push_str(&format!(
            "        \"orphan_sidecar_bytes\": {},\n",
            target.summary.orphan_sidecar_bytes
        ));
        output.push_str(&format!(
            "        \"unrecognized_sidecar_bytes\": {}\n",
            target.summary.unrecognized_sidecar_bytes
        ));
        output.push_str("      },\n");
        output.push_str("      \"source_pin\": ");
        match &target.source_pin {
            Some(source_pin) => {
                output.push_str("{\n");
                output.push_str(&format!(
                    "        \"path\": {},\n",
                    json_string_literal(&source_pin.path)
                ));
                output.push_str(&format!("        \"bytes\": {},\n", source_pin.bytes));
                output.push_str(&format!(
                    "        \"sha256\": {}\n",
                    json_string_literal(&source_pin.sha256)
                ));
                output.push_str("      },\n");
            }
            None => output.push_str("null,\n"),
        }
        output.push_str("      \"probe_expectations\": [\n");
        for (probe_index, probe_expectation) in target.probe_expectations.iter().enumerate() {
            output.push_str("        {\n");
            output.push_str(&format!(
                "          \"path\": {},\n",
                json_string_literal(&probe_expectation.path)
            ));
            output.push_str(&format!(
                "          \"bytes\": {},\n",
                probe_expectation.bytes
            ));
            output.push_str(&format!(
                "          \"sha256\": {}\n",
                json_string_literal(&probe_expectation.sha256)
            ));
            output.push_str("        }");
            if probe_index + 1 < target.probe_expectations.len() {
                output.push(',');
            }
            output.push('\n');
        }
        output.push_str("      ]\n");
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
            "      \"expected_fixture\": {},\n",
            json_string_literal(&orphan.expected_fixture)
        ));
        output.push_str(&format!("      \"bytes\": {},\n", orphan.bytes));
        output.push_str(&format!(
            "      \"sha256\": {}\n",
            json_string_literal(&orphan.sha256)
        ));
        output.push_str("    }");
        if index + 1 < manifest.orphan_sidecars.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ],\n");
    output.push_str("  \"unrecognized_sidecars\": [\n");
    for (index, sidecar) in manifest.unrecognized_sidecars.iter().enumerate() {
        output.push_str("    {\n");
        output.push_str(&format!(
            "      \"path\": {},\n",
            json_string_literal(&sidecar.path)
        ));
        output.push_str(&format!(
            "      \"extension\": {},\n",
            json_string_literal(&sidecar.extension)
        ));
        output.push_str(&format!(
            "      \"expected_fixture\": {},\n",
            json_string_literal(&sidecar.expected_fixture)
        ));
        output.push_str(&format!("      \"bytes\": {},\n", sidecar.bytes));
        output.push_str(&format!(
            "      \"sha256\": {}\n",
            json_string_literal(&sidecar.sha256)
        ));
        output.push_str("    }");
        if index + 1 < manifest.unrecognized_sidecars.len() {
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

fn json_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
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

fn text_field_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn text_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
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
