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
