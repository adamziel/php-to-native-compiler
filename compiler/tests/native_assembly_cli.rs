use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn native_scalar_echo_emit_asm_cli_summary_matches_committed_output() {
    if !has_assembly_backend() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone182/native_scalar_echo_assembly.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone182/native_scalar_echo_assembly_emit_asm.cli"),
    )
    .expect("native scalar echo assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_array_emit_asm_rejection_cli_snapshot_matches_committed_output_without_backend_tools() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone183/native_assembly_rejection.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", "/nonexistent")
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone183/native_assembly_rejection_emit_asm.cli"),
    )
    .expect("native assembly rejection CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_scalar_echo_emit_asm_missing_backend_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone184/native_assembly_no_backend.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", "/nonexistent")
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone184/native_assembly_no_backend_emit_asm.cli"),
    )
    .expect("native assembly missing-backend CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone185/native_assembly_cc_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_cc_only(workspace_root, &cc_path);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone185/native_assembly_cc_fallback_emit_asm.cli"),
    )
    .expect("native assembly cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_backend_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone186/native_assembly_backend_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failing_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone186/native_assembly_backend_failure_emit_asm.cli"),
    )
    .expect("native assembly backend-failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_selection_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone187/native_assembly_llc.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_fake_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone187/native_assembly_llc_emit_asm.cli"),
    )
    .expect("native assembly llc-selection CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone188/native_assembly_llc_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failing_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone188/native_assembly_llc_failure_emit_asm.cli"),
    )
    .expect("native assembly llc-failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_fallback_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone189/native_assembly_cc_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failing_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone189/native_assembly_cc_failure_emit_asm.cli"),
    )
    .expect("native assembly cc-fallback-failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_failed_clang_probe_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone190/native_assembly_probe_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failing_clang_probe_then_fake_llc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone190/native_assembly_probe_fallback_emit_asm.cli"),
    )
    .expect("native assembly discovery-fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_all_backend_probes_fail_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone191/native_assembly_probe_exhaustion.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_all_failing_backend_probes(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone191/native_assembly_probe_exhaustion_emit_asm.cli"),
    )
    .expect("native assembly discovery-exhaustion CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_empty_stderr_backend_failure_cli_snapshot_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone192/native_assembly_empty_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stderr_failing_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone192/native_assembly_empty_stderr_emit_asm.cli"),
    )
    .expect("native assembly empty-stderr failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_empty_stdout_backend_success_cli_snapshot_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone193/native_assembly_empty_stdout.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stdout_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone193/native_assembly_empty_stdout_emit_asm.cli"),
    )
    .expect("native assembly empty-stdout success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_success_with_stderr_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone194/native_assembly_success_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_stderr_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone194/native_assembly_success_stderr_emit_asm.cli"),
    )
    .expect("native assembly success-with-stderr CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_success_with_stderr_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone195/native_assembly_fallback_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_stderr_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone195/native_assembly_llc_success_stderr_emit_asm.cli"),
    )
    .expect("native assembly llc success-with-stderr CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_success_with_stderr_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone195/native_assembly_fallback_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_stderr_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone195/native_assembly_cc_success_stderr_emit_asm.cli"),
    )
    .expect("native assembly cc success-with-stderr CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_empty_stderr_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone196/native_assembly_empty_fallback_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stderr_failing_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone196/native_assembly_llc_empty_stderr_emit_asm.cli"),
    )
    .expect("native assembly llc empty-stderr failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_empty_stderr_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone196/native_assembly_empty_fallback_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stderr_failing_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone196/native_assembly_cc_empty_stderr_emit_asm.cli"),
    )
    .expect("native assembly cc empty-stderr failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_empty_stdout_success_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone197/native_assembly_empty_fallback_stdout.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stdout_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone197/native_assembly_llc_empty_stdout_emit_asm.cli"),
    )
    .expect("native assembly llc empty-stdout success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_empty_stdout_success_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone197/native_assembly_empty_fallback_stdout.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stdout_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone197/native_assembly_cc_empty_stdout_emit_asm.cli"),
    )
    .expect("native assembly cc empty-stdout success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_whitespace_stdout_success_cli_snapshot_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone198/native_assembly_whitespace_fallback_stdout.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_whitespace_stdout_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone198/native_assembly_llc_whitespace_stdout_emit_asm.cli"),
    )
    .expect("native assembly llc whitespace-stdout success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_whitespace_stdout_success_cli_snapshot_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone198/native_assembly_whitespace_fallback_stdout.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_whitespace_stdout_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone198/native_assembly_cc_whitespace_stdout_emit_asm.cli"),
    )
    .expect("native assembly cc whitespace-stdout success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_clang_whitespace_stdout_success_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone199/native_assembly_whitespace_stdout.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_whitespace_stdout_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone199/native_assembly_whitespace_stdout_emit_asm.cli"),
    )
    .expect("native assembly clang whitespace-stdout success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_clang_whitespace_stdout_with_stderr_success_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone200/native_assembly_whitespace_stdout_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_whitespace_stdout_stderr_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join(
            "tests/fixtures/milestone200/native_assembly_whitespace_stdout_stderr_emit_asm.cli",
        ),
    )
    .expect("native assembly clang whitespace-stdout-with-stderr success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_whitespace_stdout_with_stderr_success_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone201/native_assembly_fallback_whitespace_stdout_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_whitespace_stdout_stderr_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone201/native_assembly_llc_whitespace_stdout_stderr_emit_asm.cli",
    ))
    .expect("native assembly llc whitespace-stdout-with-stderr success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_whitespace_stdout_with_stderr_success_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone201/native_assembly_fallback_whitespace_stdout_stderr.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_whitespace_stdout_stderr_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone201/native_assembly_cc_whitespace_stdout_stderr_emit_asm.cli",
    ))
    .expect("native assembly cc whitespace-stdout-with-stderr success CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_clang_validates_ir_stdin_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone202/native_assembly_validating_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone202/native_assembly_validating_clang_emit_asm.cli"),
    )
    .expect("native assembly IR-validating clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_validates_ir_stdin_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone203/native_assembly_validating_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_ir_validating_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone203/native_assembly_validating_llc_emit_asm.cli"),
    )
    .expect("native assembly IR-validating llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_validates_c_stdin_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone203/native_assembly_validating_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone203/native_assembly_validating_cc_emit_asm.cli"),
    )
    .expect("native assembly C-validating cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_clang_validates_arguments_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone204/native_assembly_argument_validation.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_argument_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone204/native_assembly_argument_validating_clang_emit_asm.cli",
    ))
    .expect("native assembly argument-validating clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_validates_arguments_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone204/native_assembly_argument_validation.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_argument_validating_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone204/native_assembly_argument_validating_llc_emit_asm.cli",
        ))
        .expect("native assembly argument-validating llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_validates_arguments_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone204/native_assembly_argument_validation.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_argument_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone204/native_assembly_argument_validating_cc_emit_asm.cli",
        ))
        .expect("native assembly argument-validating cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_clang_validates_probe_arguments_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone205/native_assembly_probe_argument_validation.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_probe_argument_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone205/native_assembly_probe_argument_validating_clang_emit_asm.cli",
    ))
    .expect("native assembly probe-argument-validating clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_validates_probe_arguments_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone205/native_assembly_probe_argument_validation.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_probe_argument_validating_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone205/native_assembly_probe_argument_validating_llc_emit_asm.cli",
    ))
    .expect("native assembly probe-argument-validating llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_validates_probe_arguments_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone205/native_assembly_probe_argument_validation.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_probe_argument_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone205/native_assembly_probe_argument_validating_cc_emit_asm.cli",
    ))
    .expect("native assembly probe-argument-validating cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_clang_ignores_successful_probe_output_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone206/native_assembly_probe_output.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_probe_output_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone206/native_assembly_probe_output_clang_emit_asm.cli"),
    )
    .expect("native assembly probe-output clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_ignores_successful_probe_output_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone206/native_assembly_probe_output.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_probe_output_successful_llc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone206/native_assembly_probe_output_llc_emit_asm.cli"),
    )
    .expect("native assembly probe-output llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_ignores_successful_probe_output_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone206/native_assembly_probe_output.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_probe_output_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone206/native_assembly_probe_output_cc_emit_asm.cli"),
    )
    .expect("native assembly probe-output cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_failed_clang_probe_output_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone207/native_assembly_failed_probe_output.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failed_probe_output_clang_then_llc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone207/native_assembly_failed_probe_output_clang_to_llc_emit_asm.cli",
    ))
    .expect("native assembly failed-probe-output clang-to-llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_failed_llvm_probe_output_before_cc_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone207/native_assembly_failed_probe_output.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failed_probe_output_llvm_then_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone207/native_assembly_failed_probe_output_llvm_to_cc_emit_asm.cli",
    ))
    .expect("native assembly failed-probe-output llvm-to-cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_all_failed_probe_output_missing_backend_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone207/native_assembly_failed_probe_output.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_all_failed_probe_output(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone207/native_assembly_failed_probe_output_exhaustion_emit_asm.cli",
    ))
    .expect("native assembly failed-probe-output exhaustion CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_backend_start_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone208/native_assembly_start_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_start_failing_clang_after_successful_probe(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone208/native_assembly_start_failure_emit_asm.cli"),
    )
    .expect("native assembly backend-start-failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_start_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone209/native_assembly_fallback_start_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_start_failing_llc_after_successful_probe(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone209/native_assembly_llc_start_failure_emit_asm.cli"),
    )
    .expect("native assembly llc-start-failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_start_failure_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone209/native_assembly_fallback_start_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_start_failing_cc_after_successful_probe(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone209/native_assembly_cc_start_failure_emit_asm.cli"),
    )
    .expect("native assembly cc-start-failure CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_prefers_clang_when_all_backends_available_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone210/native_assembly_backend_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_all_backends_available_preferring_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone210/native_assembly_backend_precedence_emit_asm.cli"),
    )
    .expect("native assembly backend-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_prefers_llc_before_cc_when_clang_unavailable_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone211/native_assembly_fallback_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_llc_and_cc_available_preferring_llc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone211/native_assembly_fallback_precedence_emit_asm.cli"),
    )
    .expect("native assembly fallback-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_reports_selected_clang_failure_without_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone212/native_assembly_selected_failure_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failing_clang_and_available_fallbacks(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone212/native_assembly_selected_failure_precedence_emit_asm.cli",
    ))
    .expect("native assembly selected-failure-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_reports_selected_llc_failure_without_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone213/native_assembly_fallback_failure_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_failing_llc_and_available_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone213/native_assembly_fallback_failure_precedence_emit_asm.cli",
    ))
    .expect("native assembly fallback-failure-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_reports_empty_stderr_llc_failure_without_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone214/native_assembly_empty_stderr_fallback_failure_precedence.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stderr_failing_llc_and_available_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone214/native_assembly_empty_stderr_fallback_failure_precedence_emit_asm.cli",
    ))
    .expect("native assembly empty-stderr fallback-failure-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_reports_empty_stderr_clang_failure_without_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone215/native_assembly_empty_stderr_selected_failure_precedence.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_empty_stderr_failing_clang_and_available_fallbacks(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone215/native_assembly_empty_stderr_selected_failure_precedence_emit_asm.cli",
    ))
    .expect("native assembly empty-stderr selected-failure-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_reports_selected_clang_start_failure_without_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone216/native_assembly_selected_start_failure_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_start_failing_clang_after_successful_probe_and_available_fallbacks(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone216/native_assembly_selected_start_failure_precedence_emit_asm.cli",
    ))
    .expect("native assembly selected-start-failure-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn has_assembly_backend() -> bool {
    ["clang", "llc", "cc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok())
}

fn find_command_on_path(command: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|path| path.join(command))
        .find(|path| path.is_file())
}

#[cfg(unix)]
struct TempPath {
    path: PathBuf,
}

#[cfg(unix)]
impl TempPath {
    fn with_cc_only(workspace_root: &Path, cc_path: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-fallback-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary cc-only PATH directory can be created");
        std::os::unix::fs::symlink(cc_path, path.join("cc"))
            .expect("temporary cc symlink can be created");
        Self { path }
    }

    fn with_failing_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-failure-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary failing-clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake clang backend failed after accepting LLVM IR' >&2\n\
exit 42\n",
        )
        .expect("temporary failing clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary failing clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary failing clang script can be made executable");
        Self { path }
    }

    fn with_fake_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-selection-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary fake-llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary fake llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary fake llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary fake llc script can be made executable");
        Self { path }
    }

    fn with_failing_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-failure-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary failing-llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake llc backend failed after accepting LLVM IR' >&2\n\
exit 43\n",
        )
        .expect("temporary failing llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary failing llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary failing llc script can be made executable");
        Self { path }
    }

    fn with_failing_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-failure-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary failing-cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake cc fallback failed after accepting C source' >&2\n\
exit 44\n",
        )
        .expect("temporary failing cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary failing cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary failing cc script can be made executable");
        Self { path }
    }

    fn with_failing_clang_probe_then_fake_llc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-probe-fallback-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary discovery-fallback PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang version probe failed' >&2\n\
  exit 45\n\
fi\n\
printf '%s\\n' 'unexpected clang backend invocation' >&2\n\
exit 46\n",
        )
        .expect("temporary failing clang probe script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect("temporary failing clang probe script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions)
            .expect("temporary failing clang probe script can be made executable");

        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary fallback llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary fallback llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary fallback llc script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
printf '%s\\n' 'unexpected cc fallback invocation' >&2\n\
exit 47\n",
        )
        .expect("temporary unused cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary unused cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary unused cc script can be made executable");

        Self { path }
    }

    fn with_all_failing_backend_probes(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-probe-exhaustion-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary discovery-exhaustion PATH directory can be created");

        for (command, exit_code) in [("clang", 48), ("llc", 49), ("cc", 50)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} version probe failed' >&2\n\
  exit {exit_code}\n\
fi\n\
printf '%s\\n' 'unexpected {command} backend invocation' >&2\n\
exit 51\n"
                ),
            )
            .expect("temporary failing backend probe script can be written");
            let mut permissions = fs::metadata(&script)
                .expect("temporary failing backend probe script metadata is readable")
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions)
                .expect("temporary failing backend probe script can be made executable");
        }

        Self { path }
    }

    fn with_empty_stderr_failing_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty-stderr clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 52\n",
        )
        .expect("temporary empty-stderr clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary empty-stderr clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary empty-stderr clang script can be made executable");
        Self { path }
    }

    fn with_empty_stdout_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stdout-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty-stdout clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 0\n",
        )
        .expect("temporary empty-stdout clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary empty-stdout clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary empty-stdout clang script can be made executable");
        Self { path }
    }

    fn with_stderr_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-success-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary success-with-stderr clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake clang warning on successful assembly' >&2\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary success-with-stderr clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary success-with-stderr clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary success-with-stderr clang script can be made executable");
        Self { path }
    }

    fn with_whitespace_stdout_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-whitespace-stdout-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary whitespace-stdout clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary whitespace-stdout clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary whitespace-stdout clang script can be made executable");
        Self { path }
    }

    fn with_whitespace_stdout_stderr_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-whitespace-stdout-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary whitespace-stdout-with-stderr clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake clang warning before invalid successful output' >&2\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout-with-stderr clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary whitespace-stdout-with-stderr clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary whitespace-stdout-with-stderr clang script can be made executable");
        Self { path }
    }

    fn with_whitespace_stdout_stderr_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-whitespace-stdout-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary whitespace-stdout-with-stderr llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake llc warning before invalid successful output' >&2\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout-with-stderr llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary whitespace-stdout-with-stderr llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary whitespace-stdout-with-stderr llc script can be made executable");
        Self { path }
    }

    fn with_whitespace_stdout_stderr_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-whitespace-stdout-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary whitespace-stdout-with-stderr cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake cc warning before invalid successful output' >&2\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout-with-stderr cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary whitespace-stdout-with-stderr cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary whitespace-stdout-with-stderr cc script can be made executable");
        Self { path }
    }

    fn with_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary IR-validating clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
ir=''\n\
while IFS= read -r line; do\n\
  ir=\"${ir}${line}\"\n\
done\n\
case \"$ir\" in\n\
  *'declare i32 @printf(ptr, ...)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing printf declaration on stdin' >&2\n\
  exit 60\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 61\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'call i32 (ptr, ...) @printf'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing printf call on stdin' >&2\n\
  exit 62\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_ir_validating_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary IR-validating llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
ir=''\n\
while IFS= read -r line; do\n\
  ir=\"${ir}${line}\"\n\
done\n\
case \"$ir\" in\n\
  *'declare i32 @printf(ptr, ...)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake llc missing printf declaration on stdin' >&2\n\
  exit 63\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake llc missing main definition on stdin' >&2\n\
  exit 64\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'call i32 (ptr, ...) @printf'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake llc missing printf call on stdin' >&2\n\
  exit 65\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary IR-validating llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary IR-validating llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary IR-validating llc script can be made executable");
        Self { path }
    }

    fn with_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary C-validating cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
source=''\n\
while IFS= read -r line; do\n\
  source=\"${source}${line}\"\n\
done\n\
case \"$source\" in\n\
  *'/* generated by phpc milestone 1 C assembly fallback */'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing generated-source marker on stdin' >&2\n\
  exit 66\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'#include <stdio.h>'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing stdio include on stdin' >&2\n\
  exit 67\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'int main(void)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing main function on stdin' >&2\n\
  exit 68\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"fallback stdin validation\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing string printf on stdin' >&2\n\
  exit 69\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", 203)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer printf on stdin' >&2\n\
  exit 70\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_argument_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-validate-args-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary argument-validating clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
if [ \"$#\" -ne 6 ] || [ \"$1\" != \"-x\" ] || [ \"$2\" != \"ir\" ] || [ \"$3\" != \"-S\" ] || [ \"$4\" != \"-o\" ] || [ \"$5\" != \"-\" ] || [ \"$6\" != \"-\" ]; then\n\
  printf '%s\\n' \"fake clang unexpected arguments: $*\" >&2\n\
  exit 71\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary argument-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary argument-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary argument-validating clang script can be made executable");
        Self { path }
    }

    fn with_argument_validating_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-validate-args-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary argument-validating llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
if [ \"$#\" -ne 3 ] || [ \"$1\" != \"-o\" ] || [ \"$2\" != \"-\" ] || [ \"$3\" != \"-\" ]; then\n\
  printf '%s\\n' \"fake llc unexpected arguments: $*\" >&2\n\
  exit 72\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary argument-validating llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary argument-validating llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary argument-validating llc script can be made executable");
        Self { path }
    }

    fn with_argument_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-validate-args-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary argument-validating cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
if [ \"$#\" -ne 6 ] || [ \"$1\" != \"-x\" ] || [ \"$2\" != \"c\" ] || [ \"$3\" != \"-S\" ] || [ \"$4\" != \"-o\" ] || [ \"$5\" != \"-\" ] || [ \"$6\" != \"-\" ]; then\n\
  printf '%s\\n' \"fake cc unexpected arguments: $*\" >&2\n\
  exit 73\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary argument-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary argument-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary argument-validating cc script can be made executable");
        Self { path }
    }

    fn with_probe_argument_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-validate-probe-args-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary probe-argument-validating clang PATH directory can be created");
        let marker = path.join("clang.version-probed");
        let clang = path.join("clang");
        fs::write(
            &clang,
            format!(
                "#!/bin/sh\n\
if [ \"$#\" -eq 1 ] && [ \"$1\" = \"--version\" ]; then\n\
  : > '{}'\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
if [ ! -f '{}' ]; then\n\
  printf '%s\\n' 'fake clang assembly invoked before exact --version probe' >&2\n\
  exit 74\n\
fi\n\
if [ \"$#\" -ne 6 ] || [ \"$1\" != \"-x\" ] || [ \"$2\" != \"ir\" ] || [ \"$3\" != \"-S\" ] || [ \"$4\" != \"-o\" ] || [ \"$5\" != \"-\" ] || [ \"$6\" != \"-\" ]; then\n\
  printf '%s\\n' \"fake clang unexpected arguments: $*\" >&2\n\
  exit 75\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
                marker.display(),
                marker.display()
            ),
        )
        .expect("temporary probe-argument-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary probe-argument-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary probe-argument-validating clang script can be made executable");
        Self { path }
    }

    fn with_probe_argument_validating_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-validate-probe-args-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary probe-argument-validating llc PATH directory can be created");
        let marker = path.join("llc.version-probed");
        let llc = path.join("llc");
        fs::write(
            &llc,
            format!(
                "#!/bin/sh\n\
if [ \"$#\" -eq 1 ] && [ \"$1\" = \"--version\" ]; then\n\
  : > '{}'\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
if [ ! -f '{}' ]; then\n\
  printf '%s\\n' 'fake llc assembly invoked before exact --version probe' >&2\n\
  exit 76\n\
fi\n\
if [ \"$#\" -ne 3 ] || [ \"$1\" != \"-o\" ] || [ \"$2\" != \"-\" ] || [ \"$3\" != \"-\" ]; then\n\
  printf '%s\\n' \"fake llc unexpected arguments: $*\" >&2\n\
  exit 77\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
                marker.display(),
                marker.display()
            ),
        )
        .expect("temporary probe-argument-validating llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary probe-argument-validating llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary probe-argument-validating llc script can be made executable");
        Self { path }
    }

    fn with_probe_argument_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-validate-probe-args-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary probe-argument-validating cc PATH directory can be created");
        let marker = path.join("cc.version-probed");
        let cc = path.join("cc");
        fs::write(
            &cc,
            format!(
                "#!/bin/sh\n\
if [ \"$#\" -eq 1 ] && [ \"$1\" = \"--version\" ]; then\n\
  : > '{}'\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
if [ ! -f '{}' ]; then\n\
  printf '%s\\n' 'fake cc assembly invoked before exact --version probe' >&2\n\
  exit 78\n\
fi\n\
if [ \"$#\" -ne 6 ] || [ \"$1\" != \"-x\" ] || [ \"$2\" != \"c\" ] || [ \"$3\" != \"-S\" ] || [ \"$4\" != \"-o\" ] || [ \"$5\" != \"-\" ] || [ \"$6\" != \"-\" ]; then\n\
  printf '%s\\n' \"fake cc unexpected arguments: $*\" >&2\n\
  exit 79\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
                marker.display(),
                marker.display()
            ),
        )
        .expect("temporary probe-argument-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary probe-argument-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary probe-argument-validating cc script can be made executable");
        Self { path }
    }

    fn with_probe_output_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-probe-output-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary probe-output clang PATH directory can be created");
        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang version stdout'\n\
  printf '%s\\n' 'fake clang version stderr' >&2\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary probe-output clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary probe-output clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary probe-output clang script can be made executable");
        Self { path }
    }

    fn with_probe_output_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-probe-output-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary probe-output llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc version stdout'\n\
  printf '%s\\n' 'fake llc version stderr' >&2\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary probe-output llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary probe-output llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary probe-output llc script can be made executable");
        Self { path }
    }

    fn with_probe_output_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-probe-output-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary probe-output cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc version stdout'\n\
  printf '%s\\n' 'fake cc version stderr' >&2\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary probe-output cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary probe-output cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary probe-output cc script can be made executable");
        Self { path }
    }

    fn with_failed_probe_output_clang_then_llc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-failed-clang-probe-output-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary failed-probe-output clang PATH directory can be created");

        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang failed probe stdout'\n\
  printf '%s\\n' 'fake clang failed probe stderr' >&2\n\
  exit 80\n\
fi\n\
printf '%s\\n' 'unexpected clang backend invocation after failed probe' >&2\n\
exit 81\n",
        )
        .expect("temporary failed-probe-output clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect("temporary failed-probe-output clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions)
            .expect("temporary failed-probe-output clang script can be made executable");

        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary failed-probe-output llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary failed-probe-output llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary failed-probe-output llc script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
printf '%s\\n' 'unexpected cc fallback invocation' >&2\n\
exit 82\n",
        )
        .expect("temporary failed-probe-output unused cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary failed-probe-output unused cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary failed-probe-output unused cc script can be made executable");

        Self { path }
    }

    fn with_failed_probe_output_llvm_then_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-failed-llvm-probe-output-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary failed-probe-output llvm PATH directory can be created");

        for (command, exit_code) in [("clang", 83), ("llc", 84)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} failed probe stdout'\n\
  printf '%s\\n' 'fake {command} failed probe stderr' >&2\n\
  exit {exit_code}\n\
fi\n\
printf '%s\\n' 'unexpected {command} backend invocation after failed probe' >&2\n\
exit 85\n"
                ),
            )
            .expect("temporary failed-probe-output LLVM script can be written");
            let mut permissions = fs::metadata(&script)
                .expect("temporary failed-probe-output LLVM script metadata is readable")
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions)
                .expect("temporary failed-probe-output LLVM script can be made executable");
        }

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary failed-probe-output cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary failed-probe-output cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary failed-probe-output cc script can be made executable");

        Self { path }
    }

    fn with_all_failed_probe_output(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-all-failed-probe-output-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary all-failed-probe-output PATH directory can be created");

        for (command, exit_code) in [("clang", 86), ("llc", 87), ("cc", 88)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} failed probe stdout'\n\
  printf '%s\\n' 'fake {command} failed probe stderr' >&2\n\
  exit {exit_code}\n\
fi\n\
printf '%s\\n' 'unexpected {command} backend invocation after failed probe' >&2\n\
exit 89\n"
                ),
            )
            .expect("temporary all-failed-probe-output script can be written");
            let mut permissions = fs::metadata(&script)
                .expect("temporary all-failed-probe-output script metadata is readable")
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions)
                .expect("temporary all-failed-probe-output script can be made executable");
        }

        Self { path }
    }

    fn with_start_failing_clang_after_successful_probe(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-clang-start-failure-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary backend-start-failure PATH directory can be created");

        let clang = path.join("clang");
        fs::write(
            &clang,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' '#!/nonexistent/phpc-start-failure' > '{}'\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected clang backend invocation after interpreter removal' >&2\n\
exit 90\n",
                clang.display()
            ),
        )
        .expect("temporary start-failing clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary start-failing clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary start-failing clang script can be made executable");

        Self { path }
    }

    fn with_start_failing_llc_after_successful_probe(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-start-failure-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary llc-start-failure PATH directory can be created");

        let llc = path.join("llc");
        fs::write(
            &llc,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' '#!/nonexistent/phpc-start-failure' > '{}'\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected llc backend invocation after interpreter removal' >&2\n\
exit 90\n",
                llc.display()
            ),
        )
        .expect("temporary start-failing llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary start-failing llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary start-failing llc script can be made executable");

        Self { path }
    }

    fn with_start_failing_cc_after_successful_probe(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-start-failure-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary cc-start-failure PATH directory can be created");

        let cc = path.join("cc");
        fs::write(
            &cc,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' '#!/nonexistent/phpc-start-failure' > '{}'\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc backend invocation after interpreter removal' >&2\n\
exit 90\n",
                cc.display()
            ),
        )
        .expect("temporary start-failing cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary start-failing cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary start-failing cc script can be made executable");

        Self { path }
    }

    fn with_all_backends_available_preferring_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-backend-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary backend-precedence PATH directory can be created");

        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
printf '%s\\n' '# selected clang backend'\n\
exit 0\n",
        )
        .expect("temporary backend-precedence clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect("temporary backend-precedence clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions)
            .expect("temporary backend-precedence clang script can be made executable");

        for (command, exit_code) in [("llc", 91), ("cc", 92)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation when clang is available' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect("temporary backend-precedence fallback script can be written");
            let mut permissions = fs::metadata(&script)
                .expect("temporary backend-precedence fallback script metadata is readable")
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions)
                .expect("temporary backend-precedence fallback script can be made executable");
        }

        Self { path }
    }

    fn with_llc_and_cc_available_preferring_llc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-fallback-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary fallback-precedence PATH directory can be created");

        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
printf '%s\\n' '# selected llc backend before cc fallback'\n\
exit 0\n",
        )
        .expect("temporary fallback-precedence llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary fallback-precedence llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary fallback-precedence llc script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation when llc is available' >&2\n\
exit 93\n",
        )
        .expect("temporary fallback-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary fallback-precedence cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary fallback-precedence cc script can be made executable");

        Self { path }
    }

    fn with_failing_clang_and_available_fallbacks(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-selected-failure-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary selected-failure-precedence PATH directory can be created");

        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake clang selected backend failed before fallback selection' >&2\n\
exit 94\n",
        )
        .expect("temporary selected-failure-precedence clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect("temporary selected-failure-precedence clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions)
            .expect("temporary selected-failure-precedence clang script can be made executable");

        for (command, exit_code) in [("llc", 95), ("cc", 96)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after selected clang failure' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect("temporary selected-failure-precedence fallback script can be written");
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary selected-failure-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary selected-failure-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_failing_llc_and_available_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-fallback-failure-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary fallback-failure-precedence PATH directory can be created");

        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake llc fallback backend failed before cc fallback selection' >&2\n\
exit 97\n",
        )
        .expect("temporary fallback-failure-precedence llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary fallback-failure-precedence llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary fallback-failure-precedence llc script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after selected llc failure' >&2\n\
exit 98\n",
        )
        .expect("temporary fallback-failure-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary fallback-failure-precedence cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary fallback-failure-precedence cc script can be made executable");

        Self { path }
    }

    fn with_empty_stderr_failing_llc_and_available_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stderr-fallback-failure-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary empty-stderr fallback-failure-precedence PATH directory can be created",
        );

        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 99\n",
        )
        .expect("temporary empty-stderr fallback-failure-precedence llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect(
                "temporary empty-stderr fallback-failure-precedence llc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions).expect(
            "temporary empty-stderr fallback-failure-precedence llc script can be made executable",
        );

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after empty-stderr llc failure' >&2\n\
exit 100\n",
        )
        .expect("temporary empty-stderr fallback-failure-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect(
                "temporary empty-stderr fallback-failure-precedence cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions).expect(
            "temporary empty-stderr fallback-failure-precedence cc script can be made executable",
        );

        Self { path }
    }

    fn with_empty_stderr_failing_clang_and_available_fallbacks(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stderr-selected-failure-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary empty-stderr selected-failure-precedence PATH directory can be created",
        );

        let clang = path.join("clang");
        fs::write(
            &clang,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 101\n",
        )
        .expect("temporary empty-stderr selected-failure-precedence clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect(
                "temporary empty-stderr selected-failure-precedence clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary empty-stderr selected-failure-precedence clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 102), ("cc", 103)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after empty-stderr clang failure' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect(
                "temporary empty-stderr selected-failure-precedence fallback script can be written",
            );
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary empty-stderr selected-failure-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary empty-stderr selected-failure-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_start_failing_clang_after_successful_probe_and_available_fallbacks(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-selected-start-failure-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary selected-start-failure-precedence PATH directory can be created");

        let clang = path.join("clang");
        fs::write(
            &clang,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' '#!/nonexistent/phpc-start-failure' > '{}'\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected clang backend invocation after interpreter removal' >&2\n\
exit 104\n",
                clang.display()
            ),
        )
        .expect("temporary selected-start-failure-precedence clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect("temporary selected-start-failure-precedence clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary selected-start-failure-precedence clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 105), ("cc", 106)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after selected clang start failure' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect("temporary selected-start-failure-precedence fallback script can be written");
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary selected-start-failure-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary selected-start-failure-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_stderr_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-success-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary success-with-stderr llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake llc warning on successful assembly' >&2\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary success-with-stderr llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary success-with-stderr llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary success-with-stderr llc script can be made executable");
        Self { path }
    }

    fn with_stderr_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-success-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary success-with-stderr cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf '%s\\n' 'fake cc warning on successful assembly' >&2\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary success-with-stderr cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary success-with-stderr cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary success-with-stderr cc script can be made executable");
        Self { path }
    }

    fn with_empty_stderr_failing_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-empty-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty-stderr llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 53\n",
        )
        .expect("temporary empty-stderr llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary empty-stderr llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary empty-stderr llc script can be made executable");
        Self { path }
    }

    fn with_empty_stderr_failing_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-empty-stderr-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary empty-stderr cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 54\n",
        )
        .expect("temporary empty-stderr cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary empty-stderr cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary empty-stderr cc script can be made executable");
        Self { path }
    }

    fn with_empty_stdout_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-empty-stdout-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty-stdout llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 0\n",
        )
        .expect("temporary empty-stdout llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary empty-stdout llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary empty-stdout llc script can be made executable");
        Self { path }
    }

    fn with_empty_stdout_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-empty-stdout-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary empty-stdout cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
exit 0\n",
        )
        .expect("temporary empty-stdout cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary empty-stdout cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary empty-stdout cc script can be made executable");
        Self { path }
    }

    fn with_whitespace_stdout_successful_llc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-llc-whitespace-stdout-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary whitespace-stdout llc PATH directory can be created");
        let llc = path.join("llc");
        fs::write(
            &llc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout llc script can be written");
        let mut permissions = fs::metadata(&llc)
            .expect("temporary whitespace-stdout llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&llc, permissions)
            .expect("temporary whitespace-stdout llc script can be made executable");
        Self { path }
    }

    fn with_whitespace_stdout_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-whitespace-stdout-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary whitespace-stdout cc PATH directory can be created");
        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
while IFS= read -r _line; do\n\
  :\n\
done\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary whitespace-stdout cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary whitespace-stdout cc script can be made executable");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(unix)]
impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn render_asm_cli_summary(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\nnonempty: {}\ncontains_main: {}\ncontains_printf: {}\n--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n",
        !stdout.is_empty(),
        stdout.contains("main"),
        stdout.contains("printf"),
    )
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
