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
