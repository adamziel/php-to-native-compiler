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
fn native_scalar_type_introspection_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone533/native_scalar_type_introspection.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone533/native_scalar_type_introspection_cc_fallback_emit_asm.cli",
    ))
    .expect("native scalar type-introspection cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_is_numeric_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone536/native_scalar_is_numeric.php");
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
        workspace_root
            .join("tests/fixtures/milestone536/native_scalar_is_numeric_cc_fallback_emit_asm.cli"),
    )
    .expect("native scalar is_numeric cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_strlen_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone562/native_strlen.php");
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
        workspace_root.join("tests/fixtures/milestone562/native_strlen_cc_fallback_emit_asm.cli"),
    )
    .expect("native strlen cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone566/native_defined.php");
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
        workspace_root.join("tests/fixtures/milestone566/native_defined_cc_fallback_emit_asm.cli"),
    )
    .expect("native defined cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_sort_regular_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone569/native_defined_sort_regular.php");
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

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone569/native_defined_sort_regular_cc_fallback_emit_asm.cli",
        ))
        .expect("native defined SORT_REGULAR cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_sort_numeric_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone573/native_defined_sort_numeric.php");
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

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone573/native_defined_sort_numeric_cc_fallback_emit_asm.cli",
        ))
        .expect("native defined SORT_NUMERIC cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_countable_iterable_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone537/native_scalar_countable_iterable.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone537/native_scalar_countable_iterable_cc_fallback_emit_asm.cli",
    ))
    .expect("native scalar countable/iterable cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_object_debug_type_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone538/native_scalar_object_debug_type.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone538/native_scalar_object_debug_type_cc_fallback_emit_asm.cli",
    ))
    .expect("native scalar object/debug-type cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_metadata_exists_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone539/native_static_metadata_exists.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone539/native_static_metadata_exists_cc_fallback_emit_asm.cli",
    ))
    .expect("native static metadata-exists cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_member_metadata_exists_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone540/native_static_member_metadata_exists.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone540/native_static_member_metadata_exists_cc_fallback_emit_asm.cli",
    ))
    .expect("native static member metadata-exists cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_relationship_metadata_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone541/native_static_relationship_metadata.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone541/native_static_relationship_metadata_cc_fallback_emit_asm.cli",
    ))
    .expect("native static relationship metadata cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_variable_isset_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone546/native_direct_variable_isset.php");
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

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone546/native_direct_variable_isset_cc_fallback_emit_asm.cli",
        ))
        .expect("native direct-variable isset cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_variable_empty_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone556/native_direct_variable_empty.php");
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

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone556/native_direct_variable_empty_cc_fallback_emit_asm.cli",
        ))
        .expect("native direct-variable empty cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_function_exists_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone547/native_function_exists.php");
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
        workspace_root
            .join("tests/fixtures/milestone547/native_function_exists_cc_fallback_emit_asm.cli"),
    )
    .expect("native function_exists cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_is_callable_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone548/native_is_callable.php");
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
        workspace_root
            .join("tests/fixtures/milestone548/native_is_callable_cc_fallback_emit_asm.cli"),
    )
    .expect("native is_callable cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_change_key_case_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone588/native_array_change_key_case_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone588/native_array_change_key_case_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_change_key_case callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_column_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone591/native_array_column_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone591/native_array_column_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_column callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_count_values_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone596/native_array_count_values_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone596/native_array_count_values_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_count_values callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_sum_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone601/native_array_sum_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone601/native_array_sum_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_sum callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_product_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone606/native_array_product_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone606/native_array_product_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_product callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_reduce_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone611/native_array_reduce_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone611/native_array_reduce_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_reduce callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_filter_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone616/native_array_filter_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone616/native_array_filter_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_filter callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_array_is_list_callable_lookup_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone621/native_array_is_list_callable_lookup.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone621/native_array_is_list_callable_lookup_cc_fallback_emit_asm.cli",
    ))
    .expect("native array_is_list callable lookup cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_is_callable_false_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let Some(cc_path) = find_command_on_path("cc") else {
        return;
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone549/native_is_callable_false.php");
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
        workspace_root
            .join("tests/fixtures/milestone549/native_is_callable_false_cc_fallback_emit_asm.cli"),
    )
    .expect("native scalar is_callable false cc fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_is_callable_false_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone552/native_is_callable_false_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_is_callable_false_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone552/native_is_callable_false_selected_clang_emit_asm.cli",
        ))
        .expect("native scalar is_callable false selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_function_exists_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone558/native_function_exists_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_function_exists_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone558/native_function_exists_selected_clang_emit_asm.cli"),
    )
    .expect("native function_exists selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_variable_empty_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone559/native_empty_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone559/native_empty_selected_clang_emit_asm.cli"),
    )
    .expect("native empty selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_variable_isset_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone564/native_isset_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_isset_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone564/native_isset_selected_clang_emit_asm.cli"),
    )
    .expect("native isset selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_is_numeric_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone565/native_is_numeric_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_is_numeric_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone565/native_is_numeric_selected_clang_emit_asm.cli"),
    )
    .expect("native is_numeric selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_countable_iterable_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone581/native_scalar_countable_iterable_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_countable_iterable_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone581/native_scalar_countable_iterable_selected_clang_emit_asm.cli",
    ))
    .expect("native scalar countable/iterable selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_object_debug_type_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone583/native_scalar_object_debug_type_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_scalar_object_debug_type_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone583/native_scalar_object_debug_type_selected_clang_emit_asm.cli",
    ))
    .expect("native scalar object/debug-type selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_metadata_exists_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone585/native_static_metadata_exists_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_metadata_exists_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone585/native_static_metadata_exists_selected_clang_emit_asm.cli",
    ))
    .expect("native static metadata-exists selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_strlen_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone568/native_strlen_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_strlen_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone568/native_strlen_selected_clang_emit_asm.cli"),
    )
    .expect("native strlen selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_sort_regular_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone572/native_defined_sort_regular_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_defined_sort_regular_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone572/native_defined_sort_regular_selected_clang_emit_asm.cli",
    ))
    .expect("native defined SORT_REGULAR selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_sort_numeric_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone575/native_defined_sort_numeric_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_defined_sort_numeric_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone575/native_defined_sort_numeric_selected_clang_emit_asm.cli",
    ))
    .expect("native defined SORT_NUMERIC selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_sort_string_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone577/native_defined_sort_string_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_defined_sort_string_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone577/native_defined_sort_string_selected_clang_emit_asm.cli",
    ))
    .expect("native defined SORT_STRING selected clang CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_defined_constants_emit_asm_selected_clang_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone579/native_defined_constants_selected_clang.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_defined_constants_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone579/native_defined_constants_selected_clang_emit_asm.cli",
        ))
        .expect("native defined constants selected clang CLI snapshot is readable");
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

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_reports_llc_start_failure_without_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone217/native_assembly_fallback_start_failure_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_start_failing_llc_after_successful_probe_and_available_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone217/native_assembly_fallback_start_failure_precedence_emit_asm.cli",
    ))
    .expect("native assembly fallback-start-failure-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_unstartable_clang_probe_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone218/native_assembly_probe_start_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_unstartable_clang_probe_then_fake_llc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone218/native_assembly_probe_start_failure_clang_to_llc_emit_asm.cli",
    ))
    .expect("native assembly probe-start-failure clang-to-llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_unstartable_llvm_probes_before_cc_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone218/native_assembly_probe_start_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_unstartable_llvm_probes_then_fake_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone218/native_assembly_probe_start_failure_llvm_to_cc_emit_asm.cli",
    ))
    .expect("native assembly probe-start-failure llvm-to-cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_all_unstartable_probes_missing_backend_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone218/native_assembly_probe_start_failure.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_all_unstartable_backend_probes(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone218/native_assembly_probe_start_failure_exhaustion_emit_asm.cli",
    ))
    .expect("native assembly probe-start-failure exhaustion CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_permission_denied_clang_probe_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone219/native_assembly_probe_permission_denied.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_permission_denied_clang_probe_then_fake_llc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone219/native_assembly_probe_permission_denied_clang_to_llc_emit_asm.cli",
    ))
    .expect("native assembly probe-permission-denied clang-to-llc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_skips_permission_denied_llvm_probes_before_cc_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone219/native_assembly_probe_permission_denied.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_permission_denied_llvm_probes_then_fake_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone219/native_assembly_probe_permission_denied_llvm_to_cc_emit_asm.cli",
    ))
    .expect("native assembly probe-permission-denied llvm-to-cc CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_all_permission_denied_probes_missing_backend_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone219/native_assembly_probe_permission_denied.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_all_permission_denied_backend_probes(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone219/native_assembly_probe_permission_denied_exhaustion_emit_asm.cli",
    ))
    .expect("native assembly probe-permission-denied exhaustion CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_selected_backend_permission_denied_after_probe_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone220/native_assembly_selected_permission_denied_emission.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_permission_denied_clang_after_successful_probe_and_available_fallbacks(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone220/native_assembly_selected_permission_denied_emission_emit_asm.cli",
    ))
    .expect("native assembly selected permission-denied emission CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_llc_permission_denied_after_probe_without_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone221/native_assembly_fallback_permission_denied_emission.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_permission_denied_llc_after_successful_probe_and_available_cc(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone221/native_assembly_fallback_permission_denied_emission_llc_emit_asm.cli",
    ))
    .expect("native assembly llc permission-denied emission CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_cc_permission_denied_after_probe_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone221/native_assembly_fallback_permission_denied_emission.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_permission_denied_cc_after_successful_probe(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone221/native_assembly_fallback_permission_denied_emission_cc_emit_asm.cli",
    ))
    .expect("native assembly cc permission-denied emission CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_empty_stdout_selected_clang_does_not_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone590/native_assembly_empty_stdout_selected_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_empty_stdout_successful_clang_and_available_fallbacks(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone590/native_assembly_empty_stdout_selected_precedence_emit_asm.cli",
    ))
    .expect("native assembly empty-stdout selected-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_whitespace_stdout_selected_clang_does_not_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone595/native_assembly_whitespace_stdout_selected_precedence.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_whitespace_stdout_successful_clang_and_available_fallbacks(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone595/native_assembly_whitespace_stdout_selected_precedence_emit_asm.cli",
    ))
    .expect("native assembly whitespace-stdout selected-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_whitespace_stdout_stderr_selected_clang_does_not_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone600/native_assembly_whitespace_stdout_stderr_selected_precedence.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_whitespace_stdout_stderr_successful_clang_and_available_fallbacks(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone600/native_assembly_whitespace_stdout_stderr_selected_precedence_emit_asm.cli",
    ))
    .expect(
        "native assembly whitespace-stdout-with-stderr selected-precedence CLI snapshot is readable",
    );
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_whitespace_stdout_stderr_selected_llc_does_not_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone605/native_assembly_whitespace_stdout_stderr_llc_precedence.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_whitespace_stdout_stderr_successful_llc_and_available_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone605/native_assembly_whitespace_stdout_stderr_llc_precedence_emit_asm.cli",
    ))
    .expect(
        "native assembly whitespace-stdout-with-stderr llc-precedence CLI snapshot is readable",
    );
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_empty_stdout_selected_llc_does_not_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone610/native_assembly_empty_stdout_llc_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_empty_stdout_successful_llc_and_available_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone610/native_assembly_empty_stdout_llc_precedence_emit_asm.cli",
    ))
    .expect("native assembly empty-stdout llc-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_empty_stdout_stderr_selected_llc_does_not_cc_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone620/native_assembly_empty_stdout_stderr_llc_precedence.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_empty_stdout_stderr_successful_llc_and_available_cc(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone620/native_assembly_empty_stdout_stderr_llc_precedence_emit_asm.cli",
    ))
    .expect("native assembly empty-stdout-with-stderr llc-precedence CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_echo_emit_asm_empty_stdout_stderr_selected_clang_does_not_fallback_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone615/native_assembly_empty_stdout_stderr_selected_precedence.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_empty_stdout_stderr_successful_clang_and_available_fallbacks(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone615/native_assembly_empty_stdout_stderr_selected_precedence_emit_asm.cli",
    ))
    .expect(
        "native assembly empty-stdout-with-stderr selected-precedence CLI snapshot is readable",
    );
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_mixed_echo_print_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone222/native_scalar_mixed_echo_print_assembly.php");
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

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone222/native_scalar_mixed_echo_print_assembly_emit_asm.cli",
        ))
        .expect("native scalar mixed echo/print assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_mixed_echo_print_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone223/native_scalar_mixed_echo_print_cc_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_mixed_output_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone223/native_scalar_mixed_echo_print_cc_fallback_emit_asm.cli",
    ))
    .expect("native scalar mixed echo/print C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_reassignment_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone224/native_scalar_reassignment_assembly.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_reassignment_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone224/native_scalar_reassignment_assembly_emit_asm.cli"),
    )
    .expect("native scalar reassignment assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_scalar_reassignment_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone225/native_scalar_reassignment_cc_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_reassignment_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone225/native_scalar_reassignment_cc_fallback_emit_asm.cli",
        ))
        .expect("native scalar reassignment C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_arithmetic_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone233/native_integer_arithmetic_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_arithmetic_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone233/native_integer_arithmetic_assembly_emit_asm.cli"),
    )
    .expect("native integer arithmetic assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_arithmetic_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone233/native_integer_arithmetic_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_arithmetic_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone233/native_integer_arithmetic_cc_fallback_emit_asm.cli"),
    )
    .expect("native integer arithmetic C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_modulo_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone314/native_integer_modulo_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_modulo_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone314/native_integer_modulo_assembly_emit_asm.cli"),
    )
    .expect("native integer modulo assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_modulo_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone314/native_integer_modulo_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_modulo_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone314/native_integer_modulo_cc_fallback_emit_asm.cli"),
    )
    .expect("native integer modulo C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_modulo_by_one_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone452/native_integer_modulo_by_one_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_modulo_by_one_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone452/native_integer_modulo_by_one_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer modulo-by-one folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_integer_modulo_by_one_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone507/native_untracked_integer_modulo_by_one.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_untracked_integer_modulo_by_one_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone507/native_untracked_integer_modulo_by_one_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked integer modulo-by-one C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_bounded_integer_modulo_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone453/native_bounded_integer_modulo_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_bounded_integer_modulo_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone453/native_bounded_integer_modulo_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native bounded integer modulo folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_arithmetic_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone293/native_float_arithmetic_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_float_arithmetic_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone293/native_float_arithmetic_assembly_emit_asm.cli"),
    )
    .expect("native float arithmetic assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_arithmetic_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone293/native_float_arithmetic_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_float_arithmetic_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone293/native_float_arithmetic_cc_fallback_emit_asm.cli"),
    )
    .expect("native float arithmetic C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_numeric_literal_arithmetic_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone451/native_numeric_literal_arithmetic_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_numeric_literal_arithmetic_identity_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone451/native_numeric_literal_arithmetic_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native numeric literal arithmetic identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_integer_arithmetic_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone502/native_untracked_integer_arithmetic_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_integer_arithmetic_identity_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone502/native_untracked_integer_arithmetic_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked integer arithmetic identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_identical_integer_subtraction_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone506/native_untracked_identical_integer_subtraction.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_identical_integer_subtraction_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone506/native_untracked_identical_integer_subtraction_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked identical integer subtraction C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_unary_minus_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone236/native_integer_unary_minus_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_unary_minus_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone236/native_integer_unary_minus_assembly_emit_asm.cli"),
    )
    .expect("native integer unary-minus assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_unary_minus_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone236/native_integer_unary_minus_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_unary_minus_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone236/native_integer_unary_minus_cc_fallback_emit_asm.cli",
        ))
        .expect("native integer unary-minus C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_unary_minus_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone457/native_integer_unary_minus_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_unary_minus_folding_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone457/native_integer_unary_minus_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer unary-minus folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_unary_minus_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone296/native_float_unary_minus_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_float_unary_minus_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone296/native_float_unary_minus_assembly_emit_asm.cli"),
    )
    .expect("native float unary-minus assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_unary_minus_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone296/native_float_unary_minus_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_float_unary_minus_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone296/native_float_unary_minus_cc_fallback_emit_asm.cli"),
    )
    .expect("native float unary-minus C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_unary_minus_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone458/native_float_unary_minus_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_float_unary_minus_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone458/native_float_unary_minus_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native float unary-minus folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_not_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone239/native_boolean_logical_not_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_not_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone239/native_boolean_logical_not_assembly_emit_asm.cli"),
    )
    .expect("native boolean logical-not assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_not_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone239/native_boolean_logical_not_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_not_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone239/native_boolean_logical_not_cc_fallback_emit_asm.cli",
        ))
        .expect("native boolean logical-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_boolean_logical_not_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone269/native_dynamic_boolean_logical_not_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_dynamic_boolean_logical_not_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone269/native_dynamic_boolean_logical_not_assembly_emit_asm.cli",
    ))
    .expect("native dynamic boolean logical-not assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_boolean_logical_not_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone269/native_dynamic_boolean_logical_not_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_dynamic_boolean_logical_not_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone269/native_dynamic_boolean_logical_not_cc_fallback_emit_asm.cli",
    ))
    .expect("native dynamic boolean logical-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_additive_identity_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone461/native_float_additive_identity_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_float_additive_identity_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone461/native_float_additive_identity_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native float additive identity folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_left_zero_subtraction_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone462/native_float_left_zero_subtraction_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_float_left_zero_subtraction_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone462/native_float_left_zero_subtraction_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native float left-zero subtraction folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_positive_float_multiplication_by_zero_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone463/native_positive_float_multiplication_by_zero_folding.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_positive_float_multiplication_by_zero_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone463/native_positive_float_multiplication_by_zero_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native positive float multiplication-by-zero folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_multiplication_by_negative_one_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone464/native_float_multiplication_by_negative_one_folding.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_float_multiplication_by_negative_one_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone464/native_float_multiplication_by_negative_one_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native float multiplication-by-negative-one folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_float_arithmetic_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone465/native_tracked_float_arithmetic_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_tracked_float_arithmetic_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone465/native_tracked_float_arithmetic_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked float arithmetic folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_expression_float_arithmetic_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone480/native_tracked_expression_float_arithmetic_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_tracked_expression_float_arithmetic_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone480/native_tracked_expression_float_arithmetic_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked-expression float arithmetic folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_integer_arithmetic_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone466/native_tracked_integer_arithmetic_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_tracked_integer_arithmetic_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone466/native_tracked_integer_arithmetic_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked integer arithmetic folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_expression_integer_arithmetic_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone477/native_tracked_expression_integer_arithmetic_folding.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_tracked_expression_integer_arithmetic_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone477/native_tracked_expression_integer_arithmetic_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked-expression integer arithmetic folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_not_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone459/native_boolean_logical_not_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_not_folding_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone459/native_boolean_logical_not_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean logical-not folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_not_c_fallback_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone460/native_boolean_logical_not_c_fallback_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_not_folding_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone460/native_boolean_logical_not_c_fallback_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean logical-not C fallback folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_known_string_logical_not_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone487/native_known_string_logical_not.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_known_string_logical_not_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone487/native_known_string_logical_not_cc_fallback_emit_asm.cli",
    ))
    .expect("native known string logical-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_known_numeric_logical_not_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone488/native_known_numeric_logical_not.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_known_numeric_logical_not_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone488/native_known_numeric_logical_not_cc_fallback_emit_asm.cli",
    ))
    .expect("native known numeric logical-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_null_logical_not_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone490/native_null_logical_not.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_null_logical_not_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone490/native_null_logical_not_cc_fallback_emit_asm.cli"),
    )
    .expect("native null logical-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_known_scalar_double_logical_not_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone511/native_known_scalar_double_logical_not.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_known_scalar_double_logical_not_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone511/native_known_scalar_double_logical_not_cc_fallback_emit_asm.cli",
    ))
    .expect("native known scalar double logical-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_operator_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone272/native_boolean_logical_operator_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_operator_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone272/native_boolean_logical_operator_assembly_emit_asm.cli",
        ))
        .expect("native boolean logical operator assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_operator_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone272/native_boolean_logical_operator_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_operator_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone272/native_boolean_logical_operator_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean logical operator C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_logical_known_result_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone476/native_boolean_logical_known_result_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_logical_known_result_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone476/native_boolean_logical_known_result_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean logical known-result folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_known_scalar_logical_truthiness_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone489/native_known_scalar_logical_truthiness.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_known_scalar_logical_truthiness_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone489/native_known_scalar_logical_truthiness_cc_fallback_emit_asm.cli",
    ))
    .expect("native known scalar logical truthiness C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_null_logical_truthiness_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone496/native_null_logical_truthiness.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_null_logical_truthiness_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone496/native_null_logical_truthiness_cc_fallback_emit_asm.cli",
    ))
    .expect("native null logical truthiness C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_logical_short_circuit_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone498/native_static_logical_short_circuit.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_logical_short_circuit_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone498/native_static_logical_short_circuit_cc_fallback_emit_asm.cli",
    ))
    .expect("native static logical short-circuit C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_bitwise_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone275/native_integer_bitwise_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_bitwise_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone275/native_integer_bitwise_assembly_emit_asm.cli"),
    )
    .expect("native integer bitwise assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_bitwise_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone275/native_integer_bitwise_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_bitwise_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone275/native_integer_bitwise_cc_fallback_emit_asm.cli"),
    )
    .expect("native integer bitwise C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_integer_bitwise_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone467/native_tracked_integer_bitwise_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_tracked_integer_bitwise_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone467/native_tracked_integer_bitwise_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked integer bitwise folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_expression_integer_bitwise_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone478/native_tracked_expression_integer_bitwise_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_tracked_expression_integer_bitwise_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone478/native_tracked_expression_integer_bitwise_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked-expression integer bitwise folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_literal_bitwise_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone454/native_integer_literal_bitwise_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_literal_bitwise_identity_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone454/native_integer_literal_bitwise_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer literal bitwise identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_bitwise_or_all_ones_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone499/native_integer_bitwise_or_all_ones.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_bitwise_or_all_ones_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone499/native_integer_bitwise_or_all_ones_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer bitwise OR all-ones C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_bitwise_xor_all_ones_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone500/native_integer_bitwise_xor_all_ones.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_bitwise_xor_all_ones_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone500/native_integer_bitwise_xor_all_ones_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer bitwise XOR all-ones C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_integer_bitwise_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone501/native_untracked_integer_bitwise_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_integer_bitwise_identity_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone501/native_untracked_integer_bitwise_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked integer bitwise identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_identical_integer_bitwise_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone505/native_untracked_identical_integer_bitwise.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_identical_integer_bitwise_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone505/native_untracked_identical_integer_bitwise_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked identical integer bitwise C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_integer_double_bitwise_not_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone510/native_untracked_integer_double_bitwise_not.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_integer_double_bitwise_not_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone510/native_untracked_integer_double_bitwise_not_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked integer double bitwise-not C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_shift_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone278/native_integer_shift_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_shift_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone278/native_integer_shift_assembly_emit_asm.cli"),
    )
    .expect("native integer shift assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_shift_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone278/native_integer_shift_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_shift_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone278/native_integer_shift_cc_fallback_emit_asm.cli"),
    )
    .expect("native integer shift C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_literal_shift_by_zero_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone455/native_integer_literal_shift_by_zero.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_integer_literal_shift_by_zero_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone455/native_integer_literal_shift_by_zero_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer literal shift-by-zero C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_integer_shift_by_zero_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone503/native_untracked_integer_shift_by_zero.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_untracked_integer_shift_by_zero_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone503/native_untracked_integer_shift_by_zero_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked integer shift-by-zero C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_integer_shift_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone468/native_tracked_integer_shift_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_tracked_integer_shift_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone468/native_tracked_integer_shift_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked integer shift folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_integer_shift_count_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone479/native_tracked_integer_shift_count.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_tracked_integer_shift_count_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone479/native_tracked_integer_shift_count_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked integer shift-count C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_integer_bitwise_not_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone456/native_integer_bitwise_not_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_integer_bitwise_not_folding_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone456/native_integer_bitwise_not_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native integer bitwise-not folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_ternary_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone281/native_boolean_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_boolean_ternary_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone281/native_boolean_ternary_assembly_emit_asm.cli"),
    )
    .expect("native boolean ternary assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone281/native_boolean_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_boolean_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone281/native_boolean_ternary_cc_fallback_emit_asm.cli"),
    )
    .expect("native boolean ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone482/native_boolean_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_short_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone482/native_boolean_short_ternary_cc_fallback_emit_asm.cli",
        ))
        .expect("native boolean short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_string_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone515/native_identical_string_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_string_short_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone515/native_identical_string_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical string short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_integer_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone516/native_identical_integer_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_integer_short_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone516/native_identical_integer_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical integer short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_float_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone517/native_identical_float_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_float_short_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone517/native_identical_float_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical float short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_boolean_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone518/native_identical_boolean_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_boolean_short_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone518/native_identical_boolean_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical boolean short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_boolean_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone522/native_identical_boolean_full_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_boolean_full_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone522/native_identical_boolean_full_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical boolean full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_null_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone523/native_identical_null_full_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_identical_null_full_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone523/native_identical_null_full_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical null full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_null_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone526/native_direct_null_full_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_direct_null_full_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone526/native_direct_null_full_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native direct null full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_integer_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone519/native_identical_integer_full_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_integer_full_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone519/native_identical_integer_full_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical integer full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_string_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone521/native_identical_string_full_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_identical_string_full_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone521/native_identical_string_full_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical string full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_float_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone520/native_identical_float_full_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_identical_float_full_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone520/native_identical_float_full_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical float full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_false_short_ternary_scalar_fallback_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone483/native_static_false_short_ternary_scalar_fallback.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_false_short_ternary_scalar_fallback_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone483/native_static_false_short_ternary_scalar_fallback_cc_fallback_emit_asm.cli",
    ))
    .expect("native static false short ternary scalar fallback C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_single_known_integer_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone484/native_single_known_integer_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_single_known_integer_short_ternary_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone484/native_single_known_integer_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native single-known integer short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_single_known_integer_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone492/native_single_known_integer_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_single_known_integer_full_ternary_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone492/native_single_known_integer_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native single-known integer full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_single_known_float_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone485/native_single_known_float_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_single_known_float_short_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone485/native_single_known_float_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native single-known float short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_single_known_float_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone493/native_single_known_float_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_single_known_float_full_ternary_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone493/native_single_known_float_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native single-known float full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_known_string_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone494/native_known_string_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_known_string_full_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone494/native_known_string_ternary_cc_fallback_emit_asm.cli",
        ))
        .expect("native known string full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_null_full_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone495/native_null_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_null_full_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone495/native_null_ternary_cc_fallback_emit_asm.cli"),
    )
    .expect("native null full ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_full_ternary_selected_branch_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone497/native_static_ternary_selected_branch.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_full_ternary_selected_branch_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone497/native_static_ternary_selected_branch_cc_fallback_emit_asm.cli",
    ))
    .expect("native static full ternary selected branch C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_ternary_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone284/native_float_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_float_ternary_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone284/native_float_ternary_assembly_emit_asm.cli"),
    )
    .expect("native float ternary assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_known_string_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone486/native_known_string_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_known_string_short_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone486/native_known_string_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native known string short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_null_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone491/native_null_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_null_short_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone491/native_null_short_ternary_cc_fallback_emit_asm.cli"),
    )
    .expect("native null short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_float_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone284/native_float_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_float_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone284/native_float_ternary_cc_fallback_emit_asm.cli"),
    )
    .expect("native float ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_numeric_literal_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone450/native_identical_numeric_literal_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_identical_numeric_literal_ternary_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone450/native_identical_numeric_literal_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical numeric literal ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_identical_integer_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone508/native_untracked_identical_integer_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_identical_integer_ternary_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone508/native_untracked_identical_integer_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked identical integer ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_identical_float_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone509/native_untracked_identical_float_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_identical_float_ternary_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone509/native_untracked_identical_float_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked identical float ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_single_result_scalar_ternary_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone472/native_single_result_scalar_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_single_result_scalar_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone472/native_single_result_scalar_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native single-result scalar ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_string_ternary_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone287/native_string_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_string_ternary_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone287/native_string_ternary_assembly_emit_asm.cli"),
    )
    .expect("native string ternary assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_string_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone287/native_string_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_string_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone287/native_string_ternary_cc_fallback_emit_asm.cli"),
    )
    .expect("native string ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_mixed_ternary_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone290/native_static_mixed_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_mixed_ternary_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone290/native_static_mixed_ternary_assembly_emit_asm.cli"),
    )
    .expect("native static mixed ternary assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_mixed_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone290/native_static_mixed_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_mixed_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone290/native_static_mixed_ternary_cc_fallback_emit_asm.cli",
        ))
        .expect("native static mixed ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_direct_null_short_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone524/native_direct_null_short_ternary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_direct_null_short_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone524/native_direct_null_short_ternary_cc_fallback_emit_asm.cli",
    ))
    .expect("native direct null short ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_null_ternary_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone311/native_null_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_null_ternary_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone311/native_null_ternary_assembly_emit_asm.cli"),
    )
    .expect("native null ternary assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_null_ternary_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone311/native_null_ternary_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_null_ternary_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone311/native_null_ternary_cc_fallback_emit_asm.cli"),
    )
    .expect("native null ternary C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone242/native_static_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_strict_identity_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone242/native_static_strict_identity_assembly_emit_asm.cli",
        ))
        .expect("native static strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone242/native_static_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_strict_identity_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone242/native_static_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native static strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_string_concat_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone245/native_static_string_concat_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_string_concat_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone245/native_static_string_concat_assembly_emit_asm.cli"),
    )
    .expect("native static string concat assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_string_concat_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone245/native_static_string_concat_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_string_concat_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone245/native_static_string_concat_cc_fallback_emit_asm.cli",
        ))
        .expect("native static string concat C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_single_result_string_ternary_concat_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone481/native_single_result_string_ternary_concat.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_single_result_string_ternary_concat_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone481/native_single_result_string_ternary_concat_cc_fallback_emit_asm.cli",
    ))
    .expect("native single-result string ternary concat C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_empty_string_concat_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone514/native_empty_string_concat_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_empty_string_concat_identity_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone514/native_empty_string_concat_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native empty-string concat identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_string_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone248/native_static_string_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_string_strict_identity_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone248/native_static_string_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native static string strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_string_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone248/native_static_string_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_static_string_strict_identity_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone248/native_static_string_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native static string strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_float_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone251/native_static_float_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_float_strict_identity_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone251/native_static_float_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native static float strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_float_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone251/native_static_float_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_float_strict_identity_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone251/native_static_float_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native static float strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_null_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone254/native_static_null_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_null_strict_identity_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone254/native_static_null_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native static null strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_static_null_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone254/native_static_null_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_static_null_strict_identity_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone254/native_static_null_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native static null strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_scalar_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone257/native_mixed_scalar_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_scalar_strict_identity_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone257/native_mixed_scalar_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native mixed scalar strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_scalar_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone257/native_mixed_scalar_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_scalar_strict_identity_c_validating_successful_cc_only(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone257/native_mixed_scalar_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native mixed scalar strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_integer_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone260/native_dynamic_integer_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_integer_strict_identity_ir_validating_successful_clang(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone260/native_dynamic_integer_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native dynamic integer strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_integer_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone260/native_dynamic_integer_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_integer_strict_identity_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone260/native_dynamic_integer_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native dynamic integer strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_float_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone299/native_dynamic_float_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_dynamic_float_strict_identity_ir_validating_successful_clang(workspace_root);

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone299/native_dynamic_float_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native dynamic float strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_float_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone299/native_dynamic_float_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_float_strict_identity_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone299/native_dynamic_float_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native dynamic float strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_string_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone305/native_dynamic_string_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_string_strict_identity_ir_validating_successful_clang(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone305/native_dynamic_string_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native dynamic string strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_string_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone305/native_dynamic_string_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_string_strict_identity_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone305/native_dynamic_string_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native dynamic string strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_boolean_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone263/native_dynamic_boolean_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_boolean_strict_identity_ir_validating_successful_clang(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone263/native_dynamic_boolean_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native dynamic boolean strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_dynamic_boolean_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone263/native_dynamic_boolean_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_dynamic_boolean_strict_identity_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone263/native_dynamic_boolean_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native dynamic boolean strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_literal_loose_comparison_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone445/native_boolean_literal_loose_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_boolean_literal_loose_comparison_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone445/native_boolean_literal_loose_comparison_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean literal loose-comparison C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_literal_ordering_comparison_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone446/native_boolean_literal_ordering_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_literal_ordering_comparison_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone446/native_boolean_literal_ordering_comparison_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean literal ordering-comparison C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_integer_comparison_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone469/native_tracked_integer_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_tracked_integer_comparison_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone469/native_tracked_integer_comparison_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked integer comparison folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_untracked_reflexive_integer_comparison_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone504/native_untracked_reflexive_integer_comparison.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_untracked_reflexive_integer_comparison_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone504/native_untracked_reflexive_integer_comparison_cc_fallback_emit_asm.cli",
    ))
    .expect("native untracked reflexive integer comparison C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_tracked_float_comparison_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone470/native_tracked_float_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_tracked_float_comparison_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone470/native_tracked_float_comparison_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native tracked float comparison folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_bounded_string_comparison_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone471/native_bounded_string_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_bounded_string_comparison_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone471/native_bounded_string_comparison_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native bounded string comparison folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_boolean_expression_comparison_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone473/native_boolean_expression_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_boolean_expression_comparison_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone473/native_boolean_expression_comparison_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native boolean expression comparison folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_boolean_expression_comparison_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone512/native_identical_boolean_expression_comparison.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_identical_boolean_expression_comparison_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone512/native_identical_boolean_expression_comparison_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical boolean expression comparison C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_identical_string_expression_comparison_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone513/native_identical_string_expression_comparison.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_identical_string_expression_comparison_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone513/native_identical_string_expression_comparison_cc_fallback_emit_asm.cli",
    ))
    .expect("native identical string expression comparison C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_bounded_integer_comparison_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone474/native_bounded_integer_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_bounded_integer_comparison_folding_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone474/native_bounded_integer_comparison_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native bounded integer comparison folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_bounded_float_comparison_folding_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone475/native_bounded_float_comparison_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path = TempPath::with_bounded_float_comparison_folding_c_validating_successful_cc_only(
        workspace_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone475/native_bounded_float_comparison_folding_cc_fallback_emit_asm.cli",
    ))
    .expect("native bounded float comparison folding C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_dynamic_boolean_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone266/native_mixed_dynamic_boolean_strict_identity_emit_ir.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_dynamic_boolean_strict_identity_ir_validating_successful_clang(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone266/native_mixed_dynamic_boolean_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native mixed dynamic boolean strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_dynamic_boolean_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone266/native_mixed_dynamic_boolean_strict_identity_emit_ir.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_dynamic_boolean_strict_identity_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone266/native_mixed_dynamic_boolean_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native mixed dynamic boolean strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_dynamic_float_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone302/native_mixed_dynamic_float_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_dynamic_float_strict_identity_ir_validating_successful_clang(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone302/native_mixed_dynamic_float_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native mixed dynamic float strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_dynamic_float_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone302/native_mixed_dynamic_float_strict_identity_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_dynamic_float_strict_identity_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone302/native_mixed_dynamic_float_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native mixed dynamic float strict-identity C fallback CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_dynamic_string_strict_identity_emit_asm_cli_summary_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone308/native_mixed_dynamic_string_strict_identity_emit_ir.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_dynamic_string_strict_identity_ir_validating_successful_clang(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone308/native_mixed_dynamic_string_strict_identity_assembly_emit_asm.cli",
    ))
    .expect("native mixed dynamic string strict-identity assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(unix)]
fn native_mixed_dynamic_string_strict_identity_emit_asm_cc_fallback_cli_summary_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone308/native_mixed_dynamic_string_strict_identity_emit_ir.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let temp_path =
        TempPath::with_mixed_dynamic_string_strict_identity_c_validating_successful_cc_only(
            workspace_root,
        );

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PATH", temp_path.path())
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone308/native_mixed_dynamic_string_strict_identity_cc_fallback_emit_asm.cli",
    ))
    .expect("native mixed dynamic string strict-identity C fallback CLI snapshot is readable");
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

    fn with_empty_stdout_successful_clang_and_available_fallbacks(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stdout-selected-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty-stdout selected-precedence PATH directory can be created");

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
        .expect("temporary empty-stdout selected-precedence clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect("temporary empty-stdout selected-precedence clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary empty-stdout selected-precedence clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 112), ("cc", 113)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after empty-stdout clang success' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect("temporary empty-stdout selected-precedence fallback script can be written");
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary empty-stdout selected-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary empty-stdout selected-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_empty_stdout_stderr_successful_clang_and_available_fallbacks(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stdout-stderr-selected-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary empty-stdout-with-stderr selected-precedence PATH directory can be created",
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
printf '%s\\n' 'fake clang diagnostic on empty successful assembly' >&2\n\
exit 0\n",
        )
        .expect(
            "temporary empty-stdout-with-stderr selected-precedence clang script can be written",
        );
        let mut clang_permissions = fs::metadata(&clang)
            .expect(
                "temporary empty-stdout-with-stderr selected-precedence clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary empty-stdout-with-stderr selected-precedence clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 114), ("cc", 115)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after empty-stdout clang success with stderr' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect(
                "temporary empty-stdout-with-stderr selected-precedence fallback script can be written",
            );
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary empty-stdout-with-stderr selected-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary empty-stdout-with-stderr selected-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_whitespace_stdout_successful_clang_and_available_fallbacks(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-whitespace-stdout-selected-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary whitespace-stdout selected-precedence PATH directory can be created",
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
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout selected-precedence clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect(
                "temporary whitespace-stdout selected-precedence clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary whitespace-stdout selected-precedence clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 114), ("cc", 115)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after whitespace-stdout clang success' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect("temporary whitespace-stdout selected-precedence fallback script can be written");
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary whitespace-stdout selected-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary whitespace-stdout selected-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_whitespace_stdout_stderr_successful_clang_and_available_fallbacks(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-whitespace-stdout-stderr-selected-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary whitespace-stdout-with-stderr selected-precedence PATH directory can be created",
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
printf '%s\\n' 'fake clang warning before invalid successful output' >&2\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect(
            "temporary whitespace-stdout-with-stderr selected-precedence clang script can be written",
        );
        let mut clang_permissions = fs::metadata(&clang)
            .expect(
                "temporary whitespace-stdout-with-stderr selected-precedence clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary whitespace-stdout-with-stderr selected-precedence clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 116), ("cc", 117)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after whitespace-stdout-with-stderr clang success' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect(
                "temporary whitespace-stdout-with-stderr selected-precedence fallback script can be written",
            );
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary whitespace-stdout-with-stderr selected-precedence fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary whitespace-stdout-with-stderr selected-precedence fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_whitespace_stdout_stderr_successful_llc_and_available_cc(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-whitespace-stdout-stderr-llc-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary whitespace-stdout-with-stderr llc-precedence PATH directory can be created",
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
printf '%s\\n' 'fake llc warning before invalid successful output' >&2\n\
printf ' \\n\\t\\n'\n\
exit 0\n",
        )
        .expect("temporary whitespace-stdout-with-stderr llc-precedence script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect(
                "temporary whitespace-stdout-with-stderr llc-precedence script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions).expect(
            "temporary whitespace-stdout-with-stderr llc-precedence script can be made executable",
        );

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after whitespace-stdout-with-stderr llc success' >&2\n\
exit 118\n",
        )
        .expect("temporary whitespace-stdout-with-stderr llc-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect(
                "temporary whitespace-stdout-with-stderr llc-precedence cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions).expect(
            "temporary whitespace-stdout-with-stderr llc-precedence cc script can be made executable",
        );

        Self { path }
    }

    fn with_empty_stdout_successful_llc_and_available_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stdout-llc-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty-stdout llc-precedence PATH directory can be created");

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
        .expect("temporary empty-stdout llc-precedence script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary empty-stdout llc-precedence script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary empty-stdout llc-precedence script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after empty-stdout llc success' >&2\n\
exit 119\n",
        )
        .expect("temporary empty-stdout llc-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary empty-stdout llc-precedence cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary empty-stdout llc-precedence cc script can be made executable");

        Self { path }
    }

    fn with_empty_stdout_stderr_successful_llc_and_available_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-stdout-stderr-llc-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary empty-stdout-with-stderr llc-precedence PATH directory can be created",
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
printf '%s\\n' 'fake llc diagnostic on empty successful assembly' >&2\n\
exit 0\n",
        )
        .expect("temporary empty-stdout-with-stderr llc-precedence script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary empty-stdout-with-stderr llc-precedence script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions).expect(
            "temporary empty-stdout-with-stderr llc-precedence script can be made executable",
        );

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after empty-stdout llc success with stderr' >&2\n\
exit 120\n",
        )
        .expect("temporary empty-stdout-with-stderr llc-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect(
                "temporary empty-stdout-with-stderr llc-precedence cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions).expect(
            "temporary empty-stdout-with-stderr llc-precedence cc script can be made executable",
        );

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

    fn with_reassignment_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-reassignment-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary reassignment IR-validating clang PATH directory can be created");
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
  exit 119\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 120\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'reassigned:'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing reassigned string output on stdin' >&2\n\
  exit 121\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'i64 224'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing reassigned integer output on stdin' >&2\n\
  exit 122\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'old:'*)\n\
  printf '%s\\n' 'fake clang found overwritten string value on stdin' >&2\n\
  exit 123\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$ir\" in\n\
  *'i64 1)'*)\n\
  printf '%s\\n' 'fake clang found overwritten integer value on stdin' >&2\n\
  exit 124\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary reassignment IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary reassignment IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary reassignment IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_is_callable_false_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-is-callable-false-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary is_callable false IR-validating clang PATH directory can be created",
        );
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
  exit 125\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 126\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.6 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded false output strings on stdin' >&2\n\
  exit 127\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_str, ptr @.str.6)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing final folded false printf on stdin' >&2\n\
  exit 128\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'is_callable'*)\n\
  printf '%s\\n' 'fake clang found unlowered is_callable call on stdin' >&2\n\
  exit 129\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary is_callable false IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary is_callable false IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary is_callable false IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_function_exists_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-function-exists-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary function_exists IR-validating clang PATH directory can be created");
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
  exit 130\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 131\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing first folded true function_exists output string on stdin' >&2\n\
  exit 132\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.3 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded false function_exists output string on stdin' >&2\n\
  exit 134\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.5 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing final folded false function_exists output string on stdin' >&2\n\
  exit 135\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'function_exists'*)\n\
  printf '%s\\n' 'fake clang found unlowered function_exists call on stdin' >&2\n\
  exit 133\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary function_exists IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary function_exists IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary function_exists IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_empty_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-empty-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary empty IR-validating clang PATH directory can be created");
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
  exit 136\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 137\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.5 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.6 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.7 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded empty output strings on stdin' >&2\n\
  exit 138\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'empty'*)\n\
  printf '%s\\n' 'fake clang found unlowered empty call on stdin' >&2\n\
  exit 139\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary empty IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary empty IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary empty IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_isset_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-isset-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary isset IR-validating clang PATH directory can be created");
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
  exit 140\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 141\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded isset output strings on stdin' >&2\n\
  exit 142\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'isset'*)\n\
  printf '%s\\n' 'fake clang found unlowered isset call on stdin' >&2\n\
  exit 143\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary isset IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary isset IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary isset IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_is_numeric_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-is-numeric-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary is_numeric IR-validating clang PATH directory can be created");
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
  exit 144\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 145\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.5 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.6 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.7 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.8 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.9 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.10 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.11 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded is_numeric output strings on stdin' >&2\n\
  exit 146\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'is_numeric'*)\n\
  printf '%s\\n' 'fake clang found unlowered is_numeric call on stdin' >&2\n\
  exit 147\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary is_numeric IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary is_numeric IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary is_numeric IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_countable_iterable_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-countable-iterable-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary countable/iterable IR-validating clang PATH directory can be created",
        );
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
  exit 168\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 169\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.5 = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"'*\
'@.str.6 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.7 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.8 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.9 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.10 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded countable/iterable output strings on stdin' >&2\n\
  exit 170\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'is_countable'*|*'is_iterable'*)\n\
  printf '%s\\n' 'fake clang found unlowered countable/iterable call on stdin' >&2\n\
  exit 171\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary countable/iterable IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary countable/iterable IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary countable/iterable IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_scalar_object_debug_type_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-scalar-object-debug-type-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary scalar object/debug-type IR-validating clang PATH directory can be created",
        );
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
  exit 172\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 173\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.5 = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"'*\
'@.str.6 = private unnamed_addr constant [5 x i8] c\"null\\00\"'*\
'@.str.7 = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"'*\
'@.str.8 = private unnamed_addr constant [5 x i8] c\"bool\\00\"'*\
'@.str.9 = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"'*\
'@.str.10 = private unnamed_addr constant [4 x i8] c\"int\\00\"'*\
'@.str.11 = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"'*\
'@.str.12 = private unnamed_addr constant [6 x i8] c\"float\\00\"'*\
'@.str.13 = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"'*\
'@.str.14 = private unnamed_addr constant [7 x i8] c\"string\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded object/debug-type output strings on stdin' >&2\n\
  exit 174\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'is_object'*|*'get_debug_type'*)\n\
  printf '%s\\n' 'fake clang found unlowered object/debug-type call on stdin' >&2\n\
  exit 175\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary scalar object/debug-type IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary scalar object/debug-type IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary scalar object/debug-type IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_metadata_exists_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-metadata-exists-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static metadata-exists IR-validating clang PATH directory can be created",
        );
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
  exit 176\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 177\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded static metadata-exists output strings on stdin' >&2\n\
  exit 178\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'class_exists'*|*'interface_exists'*|*'trait_exists'*|*'enum_exists'*|*'Box'*|*'I\\00'*|*'T\\00'*|*'E\\00'*)\n\
  printf '%s\\n' 'fake clang found unlowered static metadata-exists marker on stdin' >&2\n\
  exit 179\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static metadata-exists IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static metadata-exists IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static metadata-exists IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_strlen_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-strlen-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary strlen IR-validating clang PATH directory can be created");
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
  exit 148\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 149\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.fmt_int = private unnamed_addr constant [5 x i8] c\"%lld\\00\"'*\
'call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 3)'*\
'call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 6)'*\
'call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded strlen integer outputs on stdin' >&2\n\
  exit 150\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'strlen'*)\n\
  printf '%s\\n' 'fake clang found unlowered strlen call on stdin' >&2\n\
  exit 151\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary strlen IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary strlen IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary strlen IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_defined_sort_regular_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-defined-sort-regular-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary defined SORT_REGULAR IR-validating clang PATH directory can be created",
        );
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
  exit 152\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 153\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded defined SORT_REGULAR output strings on stdin' >&2\n\
  exit 154\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'SORT_REGULAR'*|*'defined'*)\n\
  printf '%s\\n' 'fake clang found unlowered defined SORT_REGULAR marker on stdin' >&2\n\
  exit 155\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary defined SORT_REGULAR IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary defined SORT_REGULAR IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary defined SORT_REGULAR IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_defined_sort_numeric_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-defined-sort-numeric-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary defined SORT_NUMERIC IR-validating clang PATH directory can be created",
        );
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
  exit 156\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 157\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded defined SORT_NUMERIC output strings on stdin' >&2\n\
  exit 158\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'SORT_NUMERIC'*|*'defined'*)\n\
  printf '%s\\n' 'fake clang found unlowered defined SORT_NUMERIC marker on stdin' >&2\n\
  exit 159\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary defined SORT_NUMERIC IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary defined SORT_NUMERIC IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary defined SORT_NUMERIC IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_defined_sort_string_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-defined-sort-string-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary defined SORT_STRING IR-validating clang PATH directory can be created",
        );
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
  exit 160\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 161\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded defined SORT_STRING output strings on stdin' >&2\n\
  exit 162\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'SORT_STRING'*|*'defined'*)\n\
  printf '%s\\n' 'fake clang found unlowered defined SORT_STRING marker on stdin' >&2\n\
  exit 163\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary defined SORT_STRING IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary defined SORT_STRING IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary defined SORT_STRING IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_defined_constants_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-defined-constants-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary defined constants IR-validating clang PATH directory can be created",
        );
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
  exit 164\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'define i32 @main()'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing main definition on stdin' >&2\n\
  exit 165\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@.str.0 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.1 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.2 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.3 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.4 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.5 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*\
'@.str.6 = private unnamed_addr constant [2 x i8] c\"1\\00\"'*\
'@.str.7 = private unnamed_addr constant [2 x i8] c\"0\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded defined constants output strings on stdin' >&2\n\
  exit 166\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'CASE_LOWER'*|*'CASE_UPPER'*|*'ARRAY_FILTER'*|*'SORT_STRING'*|*'defined'*)\n\
  printf '%s\\n' 'fake clang found unlowered defined constants marker on stdin' >&2\n\
  exit 167\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary defined constants IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary defined constants IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary defined constants IR-validating clang script can be made executable",
        );
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

    fn with_mixed_output_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-mixed-output-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary mixed-output C-validating cc PATH directory can be created");
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
  exit 112\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'#include <stdio.h>'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing stdio include on stdin' >&2\n\
  exit 113\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'int main(void)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing main function on stdin' >&2\n\
  exit 114\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"print:\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing print string output on stdin' >&2\n\
  exit 115\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", 223)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing echo integer output on stdin' >&2\n\
  exit 116\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"line\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing literal print output on stdin' >&2\n\
  exit 117\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"done\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing final print output on stdin' >&2\n\
  exit 118\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary mixed-output C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary mixed-output C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary mixed-output C-validating cc script can be made executable");
        Self { path }
    }

    fn with_reassignment_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-reassignment-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary reassignment C-validating cc PATH directory can be created");
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
  exit 125\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'#include <stdio.h>'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing stdio include on stdin' >&2\n\
  exit 126\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'int main(void)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing main function on stdin' >&2\n\
  exit 127\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"fallback:\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reassigned string output on stdin' >&2\n\
  exit 128\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", 225)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reassigned integer output on stdin' >&2\n\
  exit 129\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'old:'*)\n\
  printf '%s\\n' 'fake cc found overwritten string value on stdin' >&2\n\
  exit 130\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", 1)'*)\n\
  printf '%s\\n' 'fake cc found overwritten integer value on stdin' >&2\n\
  exit 131\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary reassignment C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary reassignment C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary reassignment C-validating cc script can be made executable");
        Self { path }
    }

    fn with_integer_arithmetic_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-integer-arithmetic-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer arithmetic IR-validating clang PATH directory can be created",
        );
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
  *'%tmp0 = add i64 10, 5'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer add on stdin' >&2\n\
  exit 132\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 45)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded integer multiply output on stdin' >&2\n\
  exit 133\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'%tmp1 = sub i64 45, 7'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer subtract on stdin' >&2\n\
  exit 134\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 %tmp1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing final integer output on stdin' >&2\n\
  exit 135\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer arithmetic IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary integer arithmetic IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary integer arithmetic IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_integer_arithmetic_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-arithmetic-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer arithmetic C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", (10 + 5))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer add output on stdin' >&2\n\
  exit 136\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", 45)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer multiply output on stdin' >&2\n\
  exit 137\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", (45 - 7))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer subtract output on stdin' >&2\n\
  exit 138\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer arithmetic C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary integer arithmetic C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary integer arithmetic C-validating cc script can be made executable");
        Self { path }
    }

    fn with_integer_modulo_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-integer-modulo-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer modulo IR-validating clang PATH directory can be created");
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
  *'%tmp0 = add i64 10, 5'*'%tmp1 = srem i64 %tmp0, 4'*'%tmp2 = srem i64 17, 5'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer modulo operations' >&2\n\
  exit 245\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 %tmp1)'*'@printf(ptr @.fmt_int, i64 %tmp2)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer modulo outputs' >&2\n\
  exit 246\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' sdiv '*|*' urem '*|*' frem '*|*'@printf(ptr @.fmt_float'*)\n\
  printf '%s\\n' 'fake clang found unsupported modulo lowering output' >&2\n\
  exit 247\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer modulo IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary integer modulo IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary integer modulo IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_integer_modulo_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-modulo-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer modulo C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", ((10 + 5) % 4))'*'printf(\"%lld\", (17 % 5))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer modulo operations' >&2\n\
  exit 248\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*' / '*)\n\
  printf '%s\\n' 'fake cc found unsupported modulo output' >&2\n\
  exit 249\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer modulo C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary integer modulo C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary integer modulo C-validating cc script can be made executable");
        Self { path }
    }

    fn with_integer_modulo_by_one_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-modulo-by-one-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer modulo-by-one C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 5)'*'printf(\"%lld\", 7)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded modulo-by-one outputs' >&2\n\
  exit 301\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' % 1'*|*'% 1)'*)\n\
  printf '%s\\n' 'fake cc found redundant modulo-by-one operation' >&2\n\
  exit 302\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer modulo-by-one C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary integer modulo-by-one C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer modulo-by-one C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_integer_modulo_by_one_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-integer-modulo-by-one-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked integer modulo-by-one C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflow-sensitive shift source' >&2\n\
  exit 423\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' % 1'*|*'% 1)'*)\n\
  printf '%s\\n' 'fake cc found redundant untracked integer modulo-by-one operation' >&2\n\
  exit 424\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary untracked integer modulo-by-one C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked integer modulo-by-one C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked integer modulo-by-one C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_bounded_integer_modulo_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-bounded-integer-modulo-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary bounded integer modulo folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", (1 + 5))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing bounded modulo folding output' >&2\n\
  exit 303\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' % 3'*|*'% 3)'*)\n\
  printf '%s\\n' 'fake cc found redundant bounded modulo operation' >&2\n\
  exit 304\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary bounded integer modulo folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary bounded integer modulo folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary bounded integer modulo folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_float_arithmetic_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-float-arithmetic-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary float arithmetic IR-validating clang PATH directory can be created");
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
  *'%tmp0 = fadd double 1.5, 2.25'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing float add on stdin' >&2\n\
  exit 241\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float, double 9.375)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing folded float multiply output on stdin' >&2\n\
  exit 242\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'%tmp1 = fsub double 9.375, 1.25'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing float subtract on stdin' >&2\n\
  exit 243\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float, double %tmp1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing final float output on stdin' >&2\n\
  exit 244\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' fdiv '*|*' frem '*|*' i64 '*)\n\
  printf '%s\\n' 'fake clang found unsupported float arithmetic lowering' >&2\n\
  exit 245\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float arithmetic IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary float arithmetic IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary float arithmetic IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_float_arithmetic_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-float-arithmetic-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary float arithmetic C-validating cc PATH directory can be created");
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
  *'printf(\"%g\", (1.5 + 2.25))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float add output on stdin' >&2\n\
  exit 246\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\", 9.375)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded float multiply output on stdin' >&2\n\
  exit 247\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\", (9.375 - 1.25))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float subtract output on stdin' >&2\n\
  exit 248\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' / '*|*' % '*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found unsupported float arithmetic output' >&2\n\
  exit 249\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float arithmetic C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary float arithmetic C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary float arithmetic C-validating cc script can be made executable");
        Self { path }
    }

    fn with_numeric_literal_arithmetic_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-numeric-literal-arithmetic-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary numeric literal arithmetic identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 5)'*'printf(\"%lld\", 6)'*'printf(\"%lld\", 7)'*'printf(\"%lld\", 9)'*'printf(\"%lld\", 10)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer literal identity outputs' >&2\n\
  exit 298\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\", 0.0)'*'printf(\"%g\", 3.5)'*'printf(\"%g\", 4.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded float literal identity outputs' >&2\n\
  exit 299\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'5 + 0'*|*'0 + 6'*|*'7 - 0'*|*'8 - 8'*|*'9 * 1'*|*'1 * 10'*|*'11 * 0'*|*'0 * 12'*|*'2.5 - 2.5'*|*'3.5 * 1.0'*|*'1.0 * 4.5'*)\n\
  printf '%s\\n' 'fake cc found redundant numeric literal identity operation' >&2\n\
  exit 300\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary numeric literal arithmetic identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary numeric literal arithmetic identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary numeric literal arithmetic identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_integer_arithmetic_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-integer-arithmetic-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked integer arithmetic identity C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflow-sensitive shift source' >&2\n\
  exit 412\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(4 << 62) + 0'*|*'0 + (4 << 62)'*|*'(4 << 62) - 0'*|*'(4 << 62) * 1'*|*'1 * (4 << 62)'*|*'(4 << 62) * 0'*|*'0 * (4 << 62)'*)\n\
  printf '%s\\n' 'fake cc found redundant untracked integer arithmetic identity operation' >&2\n\
  exit 413\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked integer arithmetic identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked integer arithmetic identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked integer arithmetic identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_identical_integer_subtraction_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-identical-integer-subtraction-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked identical integer subtraction C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflow-sensitive shift source' >&2\n\
  exit 421\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(4 << 62) - (4 << 62)'*)\n\
  printf '%s\\n' 'fake cc found redundant untracked identical integer subtraction operation' >&2\n\
  exit 422\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked identical integer subtraction C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked identical integer subtraction C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked identical integer subtraction C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_unary_minus_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-integer-unary-minus-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer unary-minus IR-validating clang PATH directory can be created",
        );
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
  *'%tmp0 = add i64 10, 2'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing prerequisite integer add on stdin' >&2\n\
  exit 140\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'sub i64 0, 5'*|*'sub i64 0, %tmp0'*)\n\
  printf '%s\\n' 'fake clang found redundant integer unary-minus operation' >&2\n\
  exit 139\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 -5)'*'@printf(ptr @.fmt_int, i64 -12)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing final unary-minus output on stdin' >&2\n\
  exit 142\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer unary-minus IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary integer unary-minus IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary integer unary-minus IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_integer_unary_minus_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-unary-minus-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer unary-minus C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", -5)'*'printf(\"%lld\", -12)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer unary-minus outputs' >&2\n\
  exit 143\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(-5)'*|*'-(10 + 2)'*)\n\
  printf '%s\\n' 'fake cc found redundant integer unary-minus operation' >&2\n\
  exit 144\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer unary-minus C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary integer unary-minus C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary integer unary-minus C-validating cc script can be made executable");
        Self { path }
    }

    fn with_integer_unary_minus_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-unary-minus-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer unary-minus folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", -5)'*'printf(\"%lld\", -12)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer unary-minus outputs' >&2\n\
  exit 311\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(-5)'*|*'-(10 + 2)'*)\n\
  printf '%s\\n' 'fake cc found redundant integer unary-minus operation' >&2\n\
  exit 312\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer unary-minus folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary integer unary-minus folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer unary-minus folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_float_unary_minus_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-float-unary-minus-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary float unary-minus IR-validating clang PATH directory can be created",
        );
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
  *'%tmp0 = fadd double 1.5, 2.25'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing prerequisite float add on stdin' >&2\n\
  exit 251\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'fsub double 0.0, 2.5'*|*'fsub double 0.0, %tmp0'*)\n\
  printf '%s\\n' 'fake clang found redundant float unary-minus operation' >&2\n\
  exit 250\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float, double -2.5)'*'@printf(ptr @.fmt_float, double -3.75)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing final float unary-minus output on stdin' >&2\n\
  exit 253\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' sub i64 '*|*'@fmt_int'*)\n\
  printf '%s\\n' 'fake clang found integer unary-minus output in float slice' >&2\n\
  exit 254\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float unary-minus IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary float unary-minus IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary float unary-minus IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_float_unary_minus_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-float-unary-minus-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary float unary-minus C-validating cc PATH directory can be created");
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
  *'printf(\"%g\", -2.5)'*'printf(\"%g\", -3.75)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded float unary-minus outputs' >&2\n\
  exit 255\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(-2.5)'*|*'-(1.5 + 2.25)'*)\n\
  printf '%s\\n' 'fake cc found redundant float unary-minus operation' >&2\n\
  exit 256\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found integer output in float unary-minus slice' >&2\n\
  exit 257\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float unary-minus C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary float unary-minus C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary float unary-minus C-validating cc script can be made executable");
        Self { path }
    }

    fn with_float_additive_identity_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-float-additive-identity-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary float additive identity folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", (1.5 + 2.5))'*'printf(\"%g\", (1.5 + 2.5))'*'printf(\"%g\", (1.5 + 2.5))'*'printf(\"%g\", (0.0 + 0.0))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded float additive identity outputs' >&2\n\
  exit 321\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1.5 + 2.5) + 0.0'*|*'0.0 + (1.5 + 2.5)'*|*'(1.5 + 2.5) - 0.0'*)\n\
  printf '%s\\n' 'fake cc found redundant nonzero float additive identity operation' >&2\n\
  exit 322\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float additive identity folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary float additive identity folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary float additive identity folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_float_left_zero_subtraction_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-float-left-zero-subtraction-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary float left-zero subtraction folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", -3.75)'*'printf(\"%g\", -2.5)'*'printf(\"%g\", (0.0 - (0.0 + 0.0)))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded float left-zero subtraction outputs' >&2\n\
  exit 323\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'0.0 - (1.5 + 2.25)'*|*'0.0 - 2.5'*)\n\
  printf '%s\\n' 'fake cc found redundant nonzero float left-zero subtraction operation' >&2\n\
  exit 324\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary float left-zero subtraction folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary float left-zero subtraction folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary float left-zero subtraction folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_positive_float_multiplication_by_zero_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-positive-float-multiplication-by-zero-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary positive float multiplication-by-zero folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", 0.0)'*'printf(\"%g\", 0.0)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded positive float multiplication-by-zero outputs' >&2\n\
  exit 325\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1.5 + 2.5) * 0.0'*|*'0.0 * (1.5 + 2.5)'*)\n\
  printf '%s\\n' 'fake cc found redundant positive float multiplication-by-zero operation' >&2\n\
  exit 326\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary positive float multiplication-by-zero folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary positive float multiplication-by-zero folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary positive float multiplication-by-zero folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_float_multiplication_by_negative_one_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-float-multiplication-by-negative-one-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary float multiplication-by-negative-one folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", -3.75)'*'printf(\"%g\", -2.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded float multiplication-by-negative-one outputs' >&2\n\
  exit 327\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1.5 + 2.25) * -1.0'*|*'-1.0 * 2.5'*)\n\
  printf '%s\\n' 'fake cc found redundant float multiplication-by-negative-one operation' >&2\n\
  exit 328\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary float multiplication-by-negative-one folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary float multiplication-by-negative-one folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary float multiplication-by-negative-one folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_float_arithmetic_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-float-arithmetic-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked float arithmetic folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", 5.0)'*'printf(\"%g\", 3.5)'*'printf(\"%g\", 7.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked float arithmetic outputs' >&2\n\
  exit 329\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1.5 + 2.25) + 1.25'*|*'(1.5 + 2.25) - 0.25'*|*'(1.5 + 2.25) * 2.0'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked float arithmetic operation' >&2\n\
  exit 330\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\", (1.5 + 2.25))'*|*'printf(\"%g\", ((0.0 + 0.0) + 0.0))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved literal-only or zero-result float arithmetic' >&2\n\
  exit 331\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary tracked float arithmetic folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked float arithmetic folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked float arithmetic folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_expression_float_arithmetic_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-expression-float-arithmetic-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked-expression float arithmetic folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", 8.25)'*'printf(\"%g\", 0.75)'*'printf(\"%g\", 4.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked-expression float arithmetic outputs' >&2\n\
  exit 344\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1.5 + 2.25) + (4.0 + 0.5)'*|*'(4.0 + 0.5) - (1.5 + 2.25)'*|*'(1.25 + 0.25) * (2.0 + 1.0)'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked-expression float arithmetic operation' >&2\n\
  exit 345\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\", (1.5 + 2.25))'*'((1.5 + 2.25) - (3.0 + 0.75))'*'? (1.25) : (2.25)'*'+'*'? (2.75) : (3.75)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved literal-only, zero-result, or ambiguous tracked-expression float arithmetic' >&2\n\
  exit 346\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked-expression float arithmetic folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked-expression float arithmetic folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked-expression float arithmetic folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_integer_arithmetic_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-integer-arithmetic-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked integer arithmetic folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 7)'*'printf(\"%lld\", 7)'*'printf(\"%lld\", 15)'*'printf(\"%lld\", 12)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked integer arithmetic outputs' >&2\n\
  exit 332\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1 + 2) + 4'*|*'10 - (1 + 2)'*|*'(1 + 2) * 5'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer arithmetic operation' >&2\n\
  exit 333\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", (1 + 2))'*'printf(\"%lld\",'*'? (3) : (4)'*' + '*'? (5) : (6)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved literal-only or tracked-expression integer arithmetic' >&2\n\
  exit 334\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked integer arithmetic folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked integer arithmetic folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked integer arithmetic folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_logical_not_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-boolean-logical-not-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean logical-not IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing logical-not true string on stdin' >&2\n\
  exit 145\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'c\"done\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing logical-not trailing string on stdin' >&2\n\
  exit 146\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found integer output for boolean logical not' >&2\n\
  exit 147\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean logical-not IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary boolean logical-not IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary boolean logical-not IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_logical_not_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-logical-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary boolean logical-not C-validating cc PATH directory can be created");
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
  *'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing logical-not true output on stdin' >&2\n\
  exit 148\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"done\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing logical-not trailing output on stdin' >&2\n\
  exit 149\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found integer output for boolean logical not' >&2\n\
  exit 150\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean logical-not C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary boolean logical-not C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary boolean logical-not C-validating cc script can be made executable");
        Self { path }
    }

    fn with_dynamic_boolean_logical_not_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-dynamic-boolean-logical-not-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic boolean logical-not IR-validating clang PATH directory can be created",
        );
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
  *'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = select i1 %tmp1, i64 3, i64 4'*'%tmp3 = select i1 %tmp1, i64 4, i64 3'*'%tmp4 = icmp eq i64 %tmp0, %tmp2'*'%tmp5 = icmp eq i64 %tmp0, %tmp3'*'%tmp6 = xor i1 %tmp4, true'*'%tmp7 = xor i1 %tmp5, true'*'%tmp12 = xor i1 %tmp7, true'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic boolean logical-not operations' >&2\n\
  exit 187\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp6'*'select i1 %tmp7'*'select i1 %tmp12'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic boolean logical-not echo selects' >&2\n\
  exit 188\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int'*|*'@printf(ptr @.fmt_float'*)\n\
  printf '%s\\n' 'fake clang found numeric output for dynamic boolean logical not' >&2\n\
  exit 189\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary dynamic boolean logical-not IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary dynamic boolean logical-not IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary dynamic boolean logical-not IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_boolean_logical_not_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-dynamic-boolean-logical-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic boolean logical-not C-validating cc PATH directory can be created",
        );
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
  *'(1 + 2)'*'=='*'!'*'if ('*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing dynamic boolean logical-not outputs' >&2\n\
  exit 190\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for dynamic boolean logical not' >&2\n\
  exit 191\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary dynamic boolean logical-not C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary dynamic boolean logical-not C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary dynamic boolean logical-not C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_expression_integer_arithmetic_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-expression-integer-arithmetic-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked-expression integer arithmetic folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 12)'*'printf(\"%lld\", 6)'*'printf(\"%lld\", 30)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked-expression integer arithmetic outputs' >&2\n\
  exit 365\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1 + 2) + (4 + 5)'*|*'(4 + 5) - (1 + 2)'*|*'(2 * 3) * (1 + 4)'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked-expression integer arithmetic operation' >&2\n\
  exit 366\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\",'*'? (3) : (4)'*' + '*'? (5) : (6)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous tracked-expression integer arithmetic' >&2\n\
  exit 367\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked-expression integer arithmetic folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked-expression integer arithmetic folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked-expression integer arithmetic folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_logical_not_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-logical-not-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean logical-not folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded boolean logical-not integer outputs' >&2\n\
  exit 313\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'!'*)\n\
  printf '%s\\n' 'fake cc found redundant boolean logical-not operation' >&2\n\
  exit 314\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean logical-not folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary boolean logical-not folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean logical-not folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_known_string_logical_not_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-known-string-logical-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary known string logical-not C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 1)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded known string logical-not outputs' >&2\n\
  exit 390\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'!'*)\n\
  printf '%s\\n' 'fake cc found redundant known string logical-not operation' >&2\n\
  exit 391\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary known string logical-not C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary known string logical-not C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary known string logical-not C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_known_numeric_logical_not_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-known-numeric-logical-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary known numeric logical-not C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded known numeric logical-not outputs' >&2\n\
  exit 392\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'!'*)\n\
  printf '%s\\n' 'fake cc found redundant known numeric logical-not operation' >&2\n\
  exit 393\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary known numeric logical-not C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary known numeric logical-not C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary known numeric logical-not C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_null_logical_not_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-null-logical-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary null logical-not C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", 1)'*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded null logical-not outputs' >&2\n\
  exit 396\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'!'*|*'?'*)\n\
  printf '%s\\n' 'fake cc found redundant null logical-not operation or conditional' >&2\n\
  exit 397\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary null logical-not C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary null logical-not C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary null logical-not C-validating cc script can be made executable");
        Self { path }
    }

    fn with_known_scalar_double_logical_not_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-known-scalar-double-logical-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary known scalar double logical-not C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded known scalar double logical-not outputs' >&2\n\
  exit 431\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'!'*|*'?'*)\n\
  printf '%s\\n' 'fake cc found redundant known scalar double logical-not operation or conditional' >&2\n\
  exit 432\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary known scalar double logical-not C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary known scalar double logical-not C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary known scalar double logical-not C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_logical_operator_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-boolean-logical-operator-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean logical-operator IR-validating clang PATH directory can be created",
        );
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
  *'%tmp6 = and i1 %tmp3, %tmp4'*'%tmp8 = or i1 %tmp3, %tmp4'*'%tmp10 = xor i1 %tmp3, %tmp4'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing boolean logical operator IR operations' >&2\n\
  exit 192\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp6'*'select i1 %tmp8'*'select i1 %tmp10'*'select i1 %tmp12'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing boolean logical operator echo selects' >&2\n\
  exit 193\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int'*|*'@printf(ptr @.fmt_float'*)\n\
  printf '%s\\n' 'fake clang found numeric output for boolean logical operators' >&2\n\
  exit 194\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean logical-operator IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary boolean logical-operator IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary boolean logical-operator IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_logical_operator_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-logical-operator-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean logical-operator C-validating cc PATH directory can be created",
        );
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
  *'(1 + 2)'*'&&'*'||'*'!='*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing boolean logical operator C output' >&2\n\
  exit 195\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for boolean logical operators' >&2\n\
  exit 196\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean logical-operator C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary boolean logical-operator C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean logical-operator C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_logical_known_result_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-logical-known-result-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean logical known-result folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded boolean logical outputs' >&2\n\
  exit 362\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'== (3)) &&'*'!= (4)'*|*'== (3)) ||'*'== (4)'*|*'== (3)) !='*'!= (4)'*|*'== (4)) &&'*'!= (4)'*)\n\
  printf '%s\\n' 'fake cc found redundant known boolean logical operation' >&2\n\
  exit 363\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*'&&'*'? (3) : (4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous boolean logical operation' >&2\n\
  exit 364\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary boolean logical known-result folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary boolean logical known-result folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean logical known-result folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_known_scalar_logical_truthiness_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-known-scalar-logical-truthiness-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary known scalar logical truthiness C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 1)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded known scalar logical truthiness outputs' >&2\n\
  exit 394\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'&&'*|*'||'*|*' != '*|*'?'*)\n\
  printf '%s\\n' 'fake cc found redundant known scalar logical operation or conditional' >&2\n\
  exit 395\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary known scalar logical truthiness C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary known scalar logical truthiness C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary known scalar logical truthiness C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_null_logical_truthiness_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-null-logical-truthiness-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary null logical truthiness C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded null logical truthiness outputs' >&2\n\
  exit 400\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'&&'*|*'||'*|*' != '*|*'?'*)\n\
  printf '%s\\n' 'fake cc found redundant null logical operation or conditional' >&2\n\
  exit 401\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary null logical truthiness C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary null logical truthiness C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary null logical truthiness C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_static_logical_short_circuit_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-logical-short-circuit-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static logical short-circuit C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static logical short-circuit outputs' >&2\n\
  exit 404\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'[]'*|*'&&'*|*'||'*|*'?'*)\n\
  printf '%s\\n' 'fake cc found unselected static logical operand or redundant operation' >&2\n\
  exit 405\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static logical short-circuit C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary static logical short-circuit C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static logical short-circuit C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_bitwise_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-integer-bitwise-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer bitwise IR-validating clang PATH directory can be created");
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
  *'%tmp0 = add i64 6, 2'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer bitwise prerequisite add' >&2\n\
  exit 197\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 0)'*'@printf(ptr @.fmt_int, i64 11)'*'@printf(ptr @.fmt_int, i64 -4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer bitwise printf calls' >&2\n\
  exit 198\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'and i64 %tmp0, 3'*|*'or i64 %tmp0, 3'*|*'xor i64 %tmp0, 3'*)\n\
  printf '%s\\n' 'fake clang found redundant tracked integer bitwise operation' >&2\n\
  exit 195\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$ir\" in\n\
  *'xor i64 3, -1'*)\n\
  printf '%s\\n' 'fake clang found redundant integer bitwise-not operation' >&2\n\
  exit 196\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$ir\" in\n\
  *' shl '*|*' ashr '*|*' lshr '*)\n\
  printf '%s\\n' 'fake clang found shift operation in integer bitwise slice' >&2\n\
  exit 199\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer bitwise IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary integer bitwise IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary integer bitwise IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_integer_bitwise_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-bitwise-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer bitwise C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 11)'*'printf(\"%lld\", -4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer bitwise outputs' >&2\n\
  exit 200\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(6 + 2) & 3'*|*'(6 + 2) | 3'*|*'(6 + 2) ^ 3'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer bitwise operation' >&2\n\
  exit 202\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", -4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer bitwise-not output' >&2\n\
  exit 205\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'~3'*|*'(~3)'*)\n\
  printf '%s\\n' 'fake cc found redundant integer bitwise-not operation' >&2\n\
  exit 207\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer bitwise printf output' >&2\n\
  exit 206\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'<<'*|*'>>'*)\n\
  printf '%s\\n' 'fake cc found shift operation in integer bitwise slice' >&2\n\
  exit 201\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer bitwise C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary integer bitwise C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary integer bitwise C-validating cc script can be made executable");
        Self { path }
    }

    fn with_tracked_integer_bitwise_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-integer-bitwise-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked integer bitwise folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 9)'*'printf(\"%lld\", 13)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked integer bitwise outputs' >&2\n\
  exit 335\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(6 + 2) & 3'*|*'1 | (6 + 2)'*|*'(6 + 2) ^ 5'*|*'(6 + 2) & (4 + 1)'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer bitwise operation' >&2\n\
  exit 336\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", (6 & 3))'*'? (12) : (10)'*'&'*'? (5) : (3)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved literal-only or tracked-expression integer bitwise' >&2\n\
  exit 337\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked integer bitwise folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked integer bitwise folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked integer bitwise folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_expression_integer_bitwise_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-expression-integer-bitwise-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked-expression integer bitwise folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 0)'*'printf(\"%lld\", 13)'*'printf(\"%lld\", 15)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked-expression integer bitwise outputs' >&2\n\
  exit 335\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(6 + 2) & (4 + 1)'*|*'(6 + 2) | (4 + 1)'*|*'(9 + 3) ^ (1 + 2)'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked-expression integer bitwise operation' >&2\n\
  exit 336\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", (6 & 3))'*'? (12) : (10)'*'&'*'? (5) : (3)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved literal-only or ambiguous tracked-expression integer bitwise' >&2\n\
  exit 337\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked-expression integer bitwise folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked-expression integer bitwise folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked-expression integer bitwise folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_literal_bitwise_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-literal-bitwise-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer literal bitwise identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 5)'*'printf(\"%lld\", 6)'*'printf(\"%lld\", 10)'*'printf(\"%lld\", 11)'*'printf(\"%lld\", 12)'*'printf(\"%lld\", 13)'*'printf(\"%lld\", 14)'*'printf(\"%lld\", 15)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer literal bitwise identity outputs' >&2\n\
  exit 305\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' & '*|*' | '*|*' ^ '*)\n\
  printf '%s\\n' 'fake cc found redundant integer literal bitwise identity operation' >&2\n\
  exit 306\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary integer literal bitwise identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary integer literal bitwise identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer literal bitwise identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_bitwise_or_all_ones_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-bitwise-or-all-ones-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer bitwise OR all-ones C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", -1)'*'printf(\"%lld\", -1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer bitwise OR all-ones outputs' >&2\n\
  exit 406\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' | '*)\n\
  printf '%s\\n' 'fake cc found redundant integer bitwise OR all-ones operation' >&2\n\
  exit 407\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer bitwise OR all-ones C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary integer bitwise OR all-ones C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer bitwise OR all-ones C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_bitwise_xor_all_ones_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-bitwise-xor-all-ones-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer bitwise XOR all-ones C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", -6)'*'printf(\"%lld\", -8)'*'printf(\"%lld\", -9)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer bitwise XOR all-ones outputs' >&2\n\
  exit 408\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ^ '*)\n\
  printf '%s\\n' 'fake cc found redundant integer bitwise XOR all-ones operation' >&2\n\
  exit 409\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer bitwise XOR all-ones C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary integer bitwise XOR all-ones C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer bitwise XOR all-ones C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_integer_bitwise_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-integer-bitwise-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked integer bitwise identity C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing untracked shift source' >&2\n\
  exit 410\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' & '*|*' | '*|*' ^ '*)\n\
  printf '%s\\n' 'fake cc found redundant untracked integer bitwise identity operation' >&2\n\
  exit 411\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked integer bitwise identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked integer bitwise identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked integer bitwise identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_identical_integer_bitwise_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-identical-integer-bitwise-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked identical integer bitwise C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing untracked shift source' >&2\n\
  exit 419\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' & '*|*' | '*|*' ^ '*)\n\
  printf '%s\\n' 'fake cc found redundant untracked identical integer bitwise operation' >&2\n\
  exit 420\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked identical integer bitwise C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked identical integer bitwise C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked identical integer bitwise C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_integer_double_bitwise_not_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-integer-double-bitwise-not-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked integer double bitwise-not C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing untracked shift source' >&2\n\
  exit 429\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'~'*)\n\
  printf '%s\\n' 'fake cc found redundant untracked integer double bitwise-not operation' >&2\n\
  exit 430\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked integer double bitwise-not C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked integer double bitwise-not C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked integer double bitwise-not C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_shift_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-integer-shift-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer shift IR-validating clang PATH directory can be created");
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
  *'%tmp0 = add i64 6, 2'*'%tmp1 = ashr i64 -8, 1'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer shift IR operations' >&2\n\
  exit 202\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 32)'*'@printf(ptr @.fmt_int, i64 4)'*'@printf(ptr @.fmt_int, i64 %tmp1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing integer shift printf calls' >&2\n\
  exit 203\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'shl i64 %tmp0, 2'*|*'ashr i64 %tmp0, 1'*)\n\
  printf '%s\\n' 'fake clang found redundant tracked integer shift operation' >&2\n\
  exit 201\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$ir\" in\n\
  *' lshr '*)\n\
  printf '%s\\n' 'fake clang found logical right shift in integer shift slice' >&2\n\
  exit 204\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer shift IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary integer shift IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary integer shift IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_integer_shift_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-shift-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary integer shift C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", 32)'*'printf(\"%lld\", 4)'*'-8'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer shift inputs' >&2\n\
  exit 205\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(6 + 2) << 2'*|*'(6 + 2) >> 1'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer shift operation' >&2\n\
  exit 206\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *' >> 1'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer right shift output' >&2\n\
  exit 207\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing integer shift printf output' >&2\n\
  exit 208\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer shift C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary integer shift C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary integer shift C-validating cc script can be made executable");
        Self { path }
    }

    fn with_integer_literal_shift_by_zero_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-literal-shift-by-zero-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer literal shift-by-zero C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 8)'*'printf(\"%lld\", 9)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer literal shift-by-zero outputs' >&2\n\
  exit 307\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' << 0'*|*' >> 0'*)\n\
  printf '%s\\n' 'fake cc found redundant integer literal shift-by-zero operation' >&2\n\
  exit 308\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer literal shift-by-zero C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary integer literal shift-by-zero C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer literal shift-by-zero C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_integer_shift_by_zero_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-integer-shift-by-zero-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked integer shift-by-zero C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflow-sensitive shift source' >&2\n\
  exit 414\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' << 0'*|*' >> 0'*)\n\
  printf '%s\\n' 'fake cc found redundant untracked integer shift-by-zero operation' >&2\n\
  exit 415\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary untracked integer shift-by-zero C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked integer shift-by-zero C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked integer shift-by-zero C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_integer_shift_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-integer-shift-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked integer shift folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 32)'*'printf(\"%lld\", 4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked integer shift outputs' >&2\n\
  exit 338\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(6 + 2) << 2'*|*'(6 + 2) >> 1'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer shift operation' >&2\n\
  exit 339\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", 20)'*' << 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded literal-only or preserved bounded integer shift' >&2\n\
  exit 340\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary tracked integer shift folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked integer shift folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked integer shift folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_integer_shift_count_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-integer-shift-count-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked integer shift-count C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 32)'*'printf(\"%lld\", 2)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked integer shift-count outputs' >&2\n\
  exit 341\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(6 + 2) << (1 + 1)'*|*'(6 + 2) >> (1 + 1)'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer shift-count operation' >&2\n\
  exit 342\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\", (8 << 2))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved literal-left shift with tracked count' >&2\n\
  exit 343\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary tracked integer shift-count C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked integer shift-count C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked integer shift-count C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_integer_bitwise_not_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-integer-bitwise-not-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary integer bitwise-not folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", -4)'*'printf(\"%lld\", -4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded integer bitwise-not outputs' >&2\n\
  exit 309\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'~'*)\n\
  printf '%s\\n' 'fake cc found redundant integer bitwise-not operation' >&2\n\
  exit 310\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary integer bitwise-not folding C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary integer bitwise-not folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary integer bitwise-not folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_ternary_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-boolean-ternary-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary boolean ternary IR-validating clang PATH directory can be created");
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
  *'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = icmp eq i64 %tmp0, 4'*'%tmp4 = select i1 %tmp1, i64 %tmp3, i64 99'*'@printf(ptr @.fmt_int, i64 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing boolean ternary select operations' >&2\n\
  exit 209\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 %tmp4)'*'@printf(ptr @.fmt_int, i64 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing boolean ternary printf calls' >&2\n\
  exit 210\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' br i1 '*|*' phi '*)\n\
  printf '%s\\n' 'fake clang found branch/phi in boolean ternary slice' >&2\n\
  exit 211\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean ternary IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary boolean ternary IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary boolean ternary IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_boolean_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary boolean ternary C-validating cc PATH directory can be created");
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
  *'(1 + 2)'*' == '*' ? '*':'*'printf(\"%lld\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing boolean ternary C output' >&2\n\
  exit 212\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found branch statement in boolean ternary slice' >&2\n\
  exit 213\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary boolean ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary boolean ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_boolean_short_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean short ternary C-validating cc PATH directory can be created",
        );
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
  *'(1 + 2)'*' == '*' ? (3) : (4)'*' ? (4) : (3)'*' ? (1) : ('*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing boolean short ternary C output' >&2\n\
  exit 380\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'?:'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found unsupported branch form in boolean short ternary slice' >&2\n\
  exit 381\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary boolean short ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_string_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-string-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical string short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", '* ) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused string short ternary printf call' >&2\n\
  exit 439\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*|*'strlen('*|*' ? \"\" : '* )\n\
  printf '%s\\n' 'fake cc found redundant string short ternary truthiness path' >&2\n\
  exit 440\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical string short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical string short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical string short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_integer_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-integer-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical integer short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", (3 << 62))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused integer short ternary printf call' >&2\n\
  exit 441\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*)\n\
  printf '%s\\n' 'fake cc found redundant integer short ternary truthiness path' >&2\n\
  exit 442\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical integer short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical integer short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical integer short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_float_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-float-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical float short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", (100000000000000000178334994858791836514563642560301392710701527770129502847789953562046870799284296099876897036220978235643807646031628623453753183252563447406133248.0 * 100000000000000000178334994858791836514563642560301392710701527770129502847789953562046870799284296099876897036220978235643807646031628623453753183252563447406133248.0))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused float short ternary printf call' >&2\n\
  exit 443\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*)\n\
  printf '%s\\n' 'fake cc found redundant float short ternary truthiness path' >&2\n\
  exit 444\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical float short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical float short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical float short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_boolean_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-boolean-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical boolean short ternary C-validating cc PATH directory can be created",
        );
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
  *'if ((((3 << 62)) != (0))) { printf(\"%s\", \"1\"); }'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused boolean short ternary echo path' >&2\n\
  exit 445\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*' : (((3 << 62)) != (0))'*)\n\
  printf '%s\\n' 'fake cc found redundant boolean short ternary select path' >&2\n\
  exit 446\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical boolean short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical boolean short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical boolean short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_boolean_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-boolean-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical boolean full ternary C-validating cc PATH directory can be created",
        );
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
  *'if ((((3 << 62)) != (0))) { printf(\"%s\", \"1\"); }'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused boolean full ternary echo path' >&2\n\
  exit 453\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*' : (((3 << 62)) != (0))'*)\n\
  printf '%s\\n' 'fake cc found redundant boolean full ternary select path' >&2\n\
  exit 454\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical boolean full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical boolean full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical boolean full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_null_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-null-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical null full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"a\")'*'printf(\"%s\", \"b\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused null full ternary surrounding output' >&2\n\
  exit 455\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*' : '*|*'NULL'*)\n\
  printf '%s\\n' 'fake cc found redundant null full ternary branch output' >&2\n\
  exit 456\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical null full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical null full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical null full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_direct_null_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-direct-null-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary direct null full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"a\")'*'printf(\"%s\", \"fallback\")'*'printf(\"%s\", \"b\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing direct null full ternary selected fallback output' >&2\n\
  exit 459\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'fail'*|*' ? '*|*' : '*|*'NULL'*|*'if ('*)\n\
  printf '%s\\n' 'fake cc found redundant direct null full ternary branch output' >&2\n\
  exit 460\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary direct null full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary direct null full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary direct null full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_integer_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-integer-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical integer full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", (3 << 62))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused integer full ternary printf call' >&2\n\
  exit 447\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*)\n\
  printf '%s\\n' 'fake cc found redundant integer full ternary truthiness path' >&2\n\
  exit 448\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical integer full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical integer full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical integer full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_string_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-string-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical string full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", '* ) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused string full ternary printf call' >&2\n\
  exit 451\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*|*'strlen('*|*' ? \"\" : '* )\n\
  printf '%s\\n' 'fake cc found redundant string full ternary truthiness path' >&2\n\
  exit 452\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical string full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical string full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical string full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_float_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-float-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical float full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", (100000000000000000178334994858791836514563642560301392710701527770129502847789953562046870799284296099876897036220978235643807646031628623453753183252563447406133248.0 * 100000000000000000178334994858791836514563642560301392710701527770129502847789953562046870799284296099876897036220978235643807646031628623453753183252563447406133248.0))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused float full ternary printf call' >&2\n\
  exit 449\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*)\n\
  printf '%s\\n' 'fake cc found redundant float full ternary truthiness path' >&2\n\
  exit 450\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary identical float full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical float full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical float full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_static_false_short_ternary_scalar_fallback_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-false-short-ternary-scalar-fallback-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static false short ternary scalar fallback C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 42)'*'printf(\"%g\", 2.5)'*'printf(\"%s\", \"fallback\")'*'printf(\"%s\", \"a\")'*'printf(\"%s\", \"b\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static false short ternary scalar fallback output' >&2\n\
  exit 382\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant branch form in static false short ternary scalar fallback slice' >&2\n\
  exit 383\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary static false short ternary scalar fallback C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary static false short ternary scalar fallback C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static false short ternary scalar fallback C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_single_known_integer_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-single-known-integer-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary single-known integer short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 1)'*'printf(\"%s\", \"zero\")'*'printf(\"%lld\", (1 + 2))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing single-known integer short ternary output' >&2\n\
  exit 384\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'\"fallback\"'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant branch or fallback in single-known integer short ternary slice' >&2\n\
  exit 385\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary single-known integer short ternary C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary single-known integer short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary single-known integer short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_single_known_integer_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-single-known-integer-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary single-known integer full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"one\")'*'printf(\"%s\", \"zero\")'*'printf(\"%lld\", 7)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing single-known integer full ternary output' >&2\n\
  exit 392\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant branch form in single-known integer full ternary slice' >&2\n\
  exit 393\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary single-known integer full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary single-known integer full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary single-known integer full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_single_known_float_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-single-known-float-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary single-known float short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%g\", 1.5)'*'printf(\"%s\", \"zero\")'*'printf(\"%g\", (1.25 + 2.5))'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing single-known float short ternary output' >&2\n\
  exit 386\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'\"fallback\"'*|*'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant branch or fallback in single-known float short ternary slice' >&2\n\
  exit 387\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary single-known float short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary single-known float short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary single-known float short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_single_known_float_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-single-known-float-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary single-known float full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"one\")'*'printf(\"%s\", \"zero\")'*'printf(\"%g\", 7.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing single-known float full ternary output' >&2\n\
  exit 394\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant branch form in single-known float full ternary slice' >&2\n\
  exit 395\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary single-known float full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary single-known float full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary single-known float full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_known_string_full_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-known-string-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary known string full ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"one\")'*'printf(\"%s\", \"empty\")'*'printf(\"%s\", \"zero\")'*'printf(\"%lld\", 7)'*'printf(\"%s\", \"falsey\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing known string full ternary output' >&2\n\
  exit 396\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"bad\")'*|*'printf(\"%lld\", 9)'*)\n\
  printf '%s\\n' 'fake cc found unselected known string full ternary output' >&2\n\
  exit 397\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary known string full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary known string full ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary known string full ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_known_string_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-known-string-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary known string short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"literal\")'*'printf(\"%s\", \"empty\")'*'printf(\"%s\", \"zero\")'*'printf(\"%s\", (((((1 + 2)) == (3))) ? (\"left\") : (\"right\")))'*'printf(\"%s\", \"falsey\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing known string short ternary output' >&2\n\
  exit 388\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'\"fallback\"'*|*'[]'*)\n\
  printf '%s\\n' 'fake cc found unsupported fallback in known string short ternary slice' >&2\n\
  exit 389\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary known string short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary known string short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary known string short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_null_full_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-null-full-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary null full ternary C-validating cc PATH directory can be created");
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
  *'printf(\"%s\", \"fallback\")'*'printf(\"%lld\", 7)'*'printf(\"%s\", \"a\")'*'printf(\"%s\", \"b\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing null full ternary false-branch output' >&2\n\
  exit 398\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'\"bad\"'*|*'printf(\"%lld\", 1)'*|*'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found unselected or redundant branch form in null full ternary slice' >&2\n\
  exit 399\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary null full ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary null full ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary null full ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_static_full_ternary_selected_branch_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-full-ternary-selected-branch-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static full ternary selected branch C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"truthy\")'*'printf(\"%s\", \"falsey\")'*'printf(\"%lld\", 7)'*'printf(\"%lld\", 9)'*'printf(\"%s\", \"null\")'*'printf(\"%s\", \"string\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static full ternary selected branch output' >&2\n\
  exit 402\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'[]'*|*'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found unselected or redundant branch form in static full ternary selected branch slice' >&2\n\
  exit 403\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary static full ternary selected branch C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary static full ternary selected branch C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static full ternary selected branch C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_null_short_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-null-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary null short ternary C-validating cc PATH directory can be created");
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
  *'printf(\"%s\", \"fallback\")'*'printf(\"%lld\", 7)'*'printf(\"%s\", \"a\")'*'printf(\"%s\", \"b\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing null short ternary fallback output' >&2\n\
  exit 390\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'?'*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant branch form in null short ternary slice' >&2\n\
  exit 391\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary null short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary null short ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary null short ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_direct_null_short_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-direct-null-short-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary direct null short ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"a\")'*'printf(\"%s\", \"b\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing direct null short ternary surrounding output' >&2\n\
  exit 457\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*' : '*|*'NULL'*)\n\
  printf '%s\\n' 'fake cc found redundant direct null short ternary branch output' >&2\n\
  exit 458\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary direct null short ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary direct null short ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary direct null short ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_float_ternary_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-float-ternary-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary float ternary IR-validating clang PATH directory can be created");
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
  *'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = icmp eq i64 %tmp0, 4'*'%tmp3 = select i1 %tmp1, double 1.5, double 2.5'*'%tmp4 = select i1 %tmp2, double 9.25, double %tmp3'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing float ternary select operations' >&2\n\
  exit 214\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float, double %tmp3)'*'@printf(ptr @.fmt_float, double %tmp4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing float ternary printf calls' >&2\n\
  exit 215\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' br i1 '*|*' phi '*)\n\
  printf '%s\\n' 'fake clang found branch/phi in float ternary slice' >&2\n\
  exit 216\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float ternary IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary float ternary IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary float ternary IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_float_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-float-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary float ternary C-validating cc PATH directory can be created");
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
  *'(1 + 2)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float ternary arithmetic input' >&2\n\
  exit 217\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' == '*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float ternary condition comparison' >&2\n\
  exit 219\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float ternary conditional operator' >&2\n\
  exit 220\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1.5)'*'(2.5)'*'(9.25)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float ternary branch literals' >&2\n\
  exit 221\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing float ternary printf output' >&2\n\
  exit 222\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found branch statement in float ternary slice' >&2\n\
  exit 218\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary float ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary float ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary float ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_identical_numeric_literal_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-numeric-literal-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical numeric literal ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\"'*'(5 + 7)'*'printf(\"%g\"'*'(2.5 + 1.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded identical numeric literal ternary outputs' >&2\n\
  exit 296\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant identical numeric literal ternary branch' >&2\n\
  exit 297\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary identical numeric literal ternary C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical numeric literal ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical numeric literal ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_identical_integer_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-identical-integer-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked identical integer ternary C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflow-sensitive shift source' >&2\n\
  exit 425\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant untracked identical integer ternary branch' >&2\n\
  exit 426\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked identical integer ternary C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked identical integer ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked identical integer ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_identical_float_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-identical-float-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked identical float ternary C-validating cc PATH directory can be created",
        );
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
  *'100000000000000000178334994858791836514563642560301392710701527770129502847789953562046870799284296099876897036220978235643807646031628623453753183252563447406133248.0 * 100000000000000000178334994858791836514563642560301392710701527770129502847789953562046870799284296099876897036220978235643807646031628623453753183252563447406133248.0'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflowing float multiply source' >&2\n\
  exit 427\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found redundant untracked identical float ternary branch' >&2\n\
  exit 428\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked identical float ternary C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked identical float ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked identical float ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_single_result_scalar_ternary_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-single-result-scalar-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary single-result scalar ternary C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", (3 + 4))'*'printf(\"%g\", (3.75 + 1.25))'*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded single-result scalar ternary outputs' >&2\n\
  exit 350\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'? ((1 + 2)) : (3)'*|*'? ((1.25 + 2.5)) : (3.75)'*|*'? (((1 + 2) == 3)) : (1)'*)\n\
  printf '%s\\n' 'fake cc found redundant single-result scalar ternary branch' >&2\n\
  exit 351\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'? (10) : (20)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous scalar ternary' >&2\n\
  exit 352\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary single-result scalar ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary single-result scalar ternary C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary single-result scalar ternary C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_string_ternary_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-string-ternary-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary string ternary IR-validating clang PATH directory can be created");
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
  *'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = icmp eq i64 %tmp0, 4'*'%tmp3 = select i1 %tmp1, ptr @.str.0, ptr @.str.1'*'%tmp4 = select i1 %tmp2, ptr @.str.2, ptr %tmp3'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing string ternary select operations' >&2\n\
  exit 225\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'c\"alpha\\00\"'*'c\"beta\\00\"'*'c\"gamma\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing string ternary branch literals' >&2\n\
  exit 226\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_str, ptr %tmp3)'*'@printf(ptr @.fmt_str, ptr %tmp4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing string ternary printf calls' >&2\n\
  exit 227\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' br i1 '*|*' phi '*)\n\
  printf '%s\\n' 'fake clang found branch/phi in string ternary slice' >&2\n\
  exit 228\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary string ternary IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary string ternary IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary string ternary IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_string_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-string-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary string ternary C-validating cc PATH directory can be created");
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
  *'(1 + 2)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing string ternary arithmetic input' >&2\n\
  exit 229\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' == '*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing string ternary condition comparison' >&2\n\
  exit 230\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing string ternary conditional operator' >&2\n\
  exit 231\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'\"alpha\"'*'\"beta\"'*'\"gamma\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing string ternary branch literals' >&2\n\
  exit 232\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing string ternary printf output' >&2\n\
  exit 233\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*|*'goto '*)\n\
  printf '%s\\n' 'fake cc found branch statement in string ternary slice' >&2\n\
  exit 234\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary string ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary string ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary string ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_static_mixed_ternary_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-mixed-ternary-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static mixed ternary IR-validating clang PATH directory can be created",
        );
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
  *'@printf(ptr @.fmt_int, i64 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static mixed ternary integer true output' >&2\n\
  exit 235\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'picked'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static mixed ternary string output' >&2\n\
  exit 239\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int, i64 7)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static mixed ternary integer false output' >&2\n\
  exit 240\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *' select '*|*'c\"no\\00\"'*|*'9.5'*|*'c\"visible\\00\"'*)\n\
  printf '%s\\n' 'fake clang found unselected static mixed ternary output' >&2\n\
  exit 236\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static mixed ternary IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static mixed ternary IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static mixed ternary IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_mixed_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-mixed-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary static mixed ternary C-validating cc PATH directory can be created");
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
  *'printf(\"%lld\", 1)'*'printf(\"%s\", \"picked\")'*'printf(\"%lld\", 7)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static mixed ternary selected outputs' >&2\n\
  exit 237\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'\"no\"'*|*'9.5'*|*'\"visible\"'*)\n\
  printf '%s\\n' 'fake cc found unselected static mixed ternary output' >&2\n\
  exit 238\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static mixed ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary static mixed ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary static mixed ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_null_ternary_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-null-ternary-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary null ternary IR-validating clang PATH directory can be created");
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
  *'%tmp0 = add i64 1, 2'*'%tmp1 = icmp eq i64 %tmp0, 3'*'@.str.0'*'@.str.1'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing null ternary folded outputs' >&2\n\
  exit 241\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp1'*|*' br i1 '*|*' phi '*|*'@printf(ptr @.fmt_int'*|*'@printf(ptr @.fmt_float'*)\n\
  printf '%s\\n' 'fake clang found runtime null ternary select or numeric output' >&2\n\
  exit 242\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary null ternary IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect("temporary null ternary IR-validating clang script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions)
            .expect("temporary null ternary IR-validating clang script can be made executable");
        Self { path }
    }

    fn with_null_ternary_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-null-ternary-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary null ternary C-validating cc PATH directory can be created");
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
  *'printf(\"%s\", \"a\")'*'printf(\"%s\", \"b\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing null ternary folded outputs' >&2\n\
  exit 243\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? '*|*'if ('*|*'printf(\"%lld\"'*|*'printf(\"%g\"'*)\n\
  printf '%s\\n' 'fake cc found runtime null ternary branch or numeric output' >&2\n\
  exit 244\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary null ternary C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary null ternary C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary null ternary C-validating cc script can be made executable");
        Self { path }
    }

    fn with_static_strict_identity_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing four strict-identity true strings' >&2\n\
  exit 151\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found integer output for static strict identity' >&2\n\
  exit 152\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static strict-identity IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_strict_identity_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing four strict-identity true outputs' >&2\n\
  exit 153\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found integer output for static strict identity' >&2\n\
  exit 154\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary static strict-identity C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_static_string_concat_ir_validating_successful_clang(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-string-concat-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static string-concat IR-validating clang PATH directory can be created",
        );
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
  *'c\"hello world\\00\"'*'c\"say: hello world\\00\"'*'c\"hello world!\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static string concat outputs' >&2\n\
  exit 155\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found integer output for static string concat' >&2\n\
  exit 156\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static string-concat IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static string-concat IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static string-concat IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_string_concat_c_validating_successful_cc_only(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-string-concat-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary static string-concat C-validating cc PATH directory can be created");
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
  *'printf(\"%s\", \"hello world\")'*'printf(\"%s\", \"say: hello world\")'*'printf(\"%s\", \"hello world!\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static string concat outputs' >&2\n\
  exit 157\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found integer output for static string concat' >&2\n\
  exit 158\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static string-concat C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect("temporary static string-concat C-validating cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions)
            .expect("temporary static string-concat C-validating cc script can be made executable");
        Self { path }
    }

    fn with_single_result_string_ternary_concat_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-single-result-string-ternary-concat-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary single-result string ternary concat C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"hello\")'*'printf(\"%s\", \"say yes\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing single-result string ternary concat outputs' >&2\n\
  exit 159\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' ? \"he\" : \"he\"'*|*' ? \"yes\" : \"yes\"'*)\n\
  printf '%s\\n' 'fake cc found redundant single-result string ternary concat expression' >&2\n\
  exit 160\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*|*'printf(\"%g\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for single-result string ternary concat' >&2\n\
  exit 161\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary single-result string ternary concat C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary single-result string ternary concat C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary single-result string ternary concat C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_empty_string_concat_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-empty-string-concat-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary empty-string concat identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", '*'printf(\"%s\", '* ) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing reused string expression printf calls' >&2\n\
  exit 437\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"\")'*|*'strcat('*|*' + \"\"'*|*'\"\" + '* )\n\
  printf '%s\\n' 'fake cc found redundant empty-string concat output' >&2\n\
  exit 438\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary empty-string concat identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc).expect(
            "temporary empty-string concat identity C-validating cc script metadata is readable",
        ).permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary empty-string concat identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_static_string_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-string-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static string strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static string strict-identity outputs' >&2\n\
  exit 159\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found integer output for static string strict identity' >&2\n\
  exit 160\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary static string strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static string strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static string strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_string_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-string-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static string strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static string strict-identity outputs' >&2\n\
  exit 161\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found integer output for static string strict identity' >&2\n\
  exit 162\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static string strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary static string strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static string strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_static_float_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-float-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static float strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static float strict-identity outputs' >&2\n\
  exit 163\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float'*|*'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found numeric output for static float strict identity' >&2\n\
  exit 164\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static float strict-identity IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static float strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static float strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_float_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-float-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static float strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static float strict-identity outputs' >&2\n\
  exit 165\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for static float strict identity' >&2\n\
  exit 166\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static float strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary static float strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static float strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_static_null_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-static-null-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static null strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing static null strict-identity outputs' >&2\n\
  exit 167\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float'*|*'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found numeric output for static null strict identity' >&2\n\
  exit 168\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static null strict-identity IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary static null strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary static null strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_static_null_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-static-null-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary static null strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing static null strict-identity outputs' >&2\n\
  exit 169\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for static null strict identity' >&2\n\
  exit 170\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary static null strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary static null strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary static null strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_scalar_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-mixed-scalar-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed scalar strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*'%tmp0 = add i64 1, 2'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing mixed scalar strict-identity outputs' >&2\n\
  exit 171\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@printf(ptr @.fmt_float'*|*'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found numeric output for mixed scalar strict identity' >&2\n\
  exit 172\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary mixed scalar strict-identity IR-validating clang script can be written");
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary mixed scalar strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary mixed scalar strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_scalar_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-mixed-scalar-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed scalar strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing mixed scalar strict-identity outputs' >&2\n\
  exit 173\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for mixed scalar strict identity' >&2\n\
  exit 174\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary mixed scalar strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary mixed scalar strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary mixed scalar strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_integer_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-dynamic-int-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic integer strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'%tmp0 = add i64 1, 2'*'%tmp1 = mul i64 3, 2'*'%tmp2 = icmp eq i64 %tmp0, 3'*'%tmp4 = icmp ne i64 %tmp1, 6'*'%tmp6 = icmp ne i64 %tmp0, %tmp1'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic integer strict-identity comparisons' >&2\n\
  exit 175\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp2'*'select i1 %tmp4'*'select i1 %tmp6'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic boolean echo selects' >&2\n\
  exit 176\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary dynamic integer strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary dynamic integer strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary dynamic integer strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_integer_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-dynamic-int-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic integer strict-identity C-validating cc PATH directory can be created",
        );
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
  *'if ('*'(1 + 2)'*'=='*'printf(\"%s\", \"1\")'*'(3 * 2)'*'!='*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing dynamic integer strict-identity comparisons' >&2\n\
  exit 177\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for dynamic integer strict identity' >&2\n\
  exit 178\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary dynamic integer strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary dynamic integer strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary dynamic integer strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_float_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-dynamic-float-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic float strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'%tmp0 = add i64 1, 2'*'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = select i1 %tmp1, double 3.75, double 4.25'*'%tmp3 = fcmp oeq double %tmp2, 3.75'*'%tmp5 = fcmp une double %tmp2, 4.25'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic float strict-identity comparisons' >&2\n\
  exit 187\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp3'*'select i1 %tmp5'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic float boolean echo selects' >&2\n\
  exit 188\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary dynamic float strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary dynamic float strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary dynamic float strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_float_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-dynamic-float-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic float strict-identity C-validating cc PATH directory can be created",
        );
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
  *'?'*'3.75'*':'*'4.25'*'=='*'3.75'*'!='*'4.25'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing dynamic float strict-identity comparisons' >&2\n\
  exit 189\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for dynamic float strict identity' >&2\n\
  exit 190\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary dynamic float strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary dynamic float strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary dynamic float strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_string_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-dynamic-string-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic string strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'declare i32 @strcmp(ptr, ptr)'*'%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1'*'%tmp3 = call i32 @strcmp(ptr %tmp2, ptr @.str.2)'*'%tmp4 = icmp eq i32 %tmp3, 0'*'call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @.str.6)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic string strict-identity comparison or folded output' >&2\n\
  exit 195\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp4'*'gamma'*)\n\
  printf '%s\\n' 'fake clang found unfolded bounded string strict-identity comparison' >&2\n\
  exit 196\n\
  ;;\n\
  *'select i1 %tmp4'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic string boolean echo selects' >&2\n\
  exit 199\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary dynamic string strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary dynamic string strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary dynamic string strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_string_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-dynamic-string-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic string strict-identity C-validating cc PATH directory can be created",
        );
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
  *'#include <string.h>'*'strcmp('*'alpha'*'== 0'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing dynamic string strict-identity comparison or folded output' >&2\n\
  exit 197\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'gamma'*)\n\
  printf '%s\\n' 'fake cc found unfolded bounded string strict-identity comparison' >&2\n\
  exit 199\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%lld\"'*|*'printf(\"%g\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for dynamic string strict identity' >&2\n\
  exit 198\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary dynamic string strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary dynamic string strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary dynamic string strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_boolean_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-dynamic-bool-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic boolean strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = select i1 %tmp1, i64 3, i64 4'*'%tmp3 = select i1 %tmp1, i64 4, i64 3'*'%tmp4 = icmp eq i64 %tmp0, %tmp2'*'%tmp5 = icmp eq i64 %tmp0, %tmp3'*'%tmp7 = xor i1 %tmp5, true'*'%tmp9 = icmp ne i1 %tmp4, %tmp5'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic boolean strict-identity comparisons' >&2\n\
  exit 179\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'select i1 %tmp4'*'select i1 %tmp7'*'select i1 %tmp9'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing dynamic boolean echo selects' >&2\n\
  exit 180\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary dynamic boolean strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary dynamic boolean strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary dynamic boolean strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_dynamic_boolean_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-dynamic-bool-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary dynamic boolean strict-identity C-validating cc PATH directory can be created",
        );
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
  *'(1 + 2)'*'=='*'if ('*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing dynamic boolean strict-identity comparisons' >&2\n\
  exit 181\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for dynamic boolean strict identity' >&2\n\
  exit 182\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary dynamic boolean strict-identity C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary dynamic boolean strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary dynamic boolean strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_literal_loose_comparison_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-bool-literal-loose-comparison-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean literal loose-comparison C-validating cc PATH directory can be created",
        );
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
  *'(1 + 2)'*'? (\"T\") : (\"F\")'*'!('*'? (\"T\") : (\"F\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing boolean literal loose-comparison folded ternaries' >&2\n\
  exit 183\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'== (1)'*|*'== (0)'*|*'!= (1)'*|*'!= (0)'*)\n\
  printf '%s\\n' 'fake cc found redundant boolean literal loose comparison' >&2\n\
  exit 184\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect("temporary boolean literal loose-comparison C-validating cc script can be written");
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary boolean literal loose-comparison C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean literal loose-comparison C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_literal_ordering_comparison_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-bool-literal-ordering-comparison-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean literal ordering-comparison C-validating cc PATH directory can be created",
        );
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
  *'!('*'? (\"T\") : (\"F\")'*'? (\"T\") : (\"F\")'*'printf(\"%s\", \"T\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing boolean literal ordering-comparison folded outputs' >&2\n\
  exit 185\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' < (1)'*|*' > (0)'*|*' <= (1)'*|*' >= (0)'*|*'(0) < '*|*'(1) > '*|*'(0) <= '*|*'(1) >= '*)\n\
  printf '%s\\n' 'fake cc found redundant boolean literal ordering comparison' >&2\n\
  exit 186\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary boolean literal ordering-comparison C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary boolean literal ordering-comparison C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean literal ordering-comparison C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_integer_comparison_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-integer-comparison-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked integer comparison folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked integer comparison outputs' >&2\n\
  exit 341\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'(1 + 2) == 3'*|*'(1 + 2) != 4'*|*'2 < (1 + 2)'*|*'4 <= (1 + 2)'*|*'(1 + 2) > 1'*|*'(1 + 2) >= 4'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked integer comparison operation' >&2\n\
  exit 342\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'? (3) : (4)'*'== (3)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved non-single tracked integer comparison' >&2\n\
  exit 343\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked integer comparison folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked integer comparison folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked integer comparison folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_untracked_reflexive_integer_comparison_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-untracked-reflexive-integer-comparison-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary untracked reflexive integer comparison C-validating cc PATH directory can be created",
        );
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
  *'(4 << 62)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing overflow-sensitive shift source' >&2\n\
  exit 416\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded reflexive true comparison outputs' >&2\n\
  exit 417\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' == '*|*' != '*|*' < '*|*' <= '*|*' > '*|*' >= '*)\n\
  printf '%s\\n' 'fake cc found redundant untracked reflexive integer comparison operation' >&2\n\
  exit 418\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary untracked reflexive integer comparison C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary untracked reflexive integer comparison C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary untracked reflexive integer comparison C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_tracked_float_comparison_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-tracked-float-comparison-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary tracked float comparison folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded tracked float comparison outputs' >&2\n\
  exit 344\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'1.25 + 2.5'*'== (3.75)'*|*'1.25 + 2.5'*'!= (4.25)'*|*'2.5 <'*'1.25 + 2.5'*|*'3.5 <='*'1.25 + 2.5'*|*'1.25 + 2.5'*'> (1.25)'*|*'1.25 + 2.5'*'>= (4.0)'*)\n\
  printf '%s\\n' 'fake cc found redundant tracked float comparison operation' >&2\n\
  exit 345\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'? (3.75) : (4.75)'*'== (3.75)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved non-single tracked float comparison' >&2\n\
  exit 346\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary tracked float comparison folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary tracked float comparison folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary tracked float comparison folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_bounded_string_comparison_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-bounded-string-comparison-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary bounded string comparison folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded bounded string comparison outputs' >&2\n\
  exit 347\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'strcmp('*'gamma\") != 0'*|*'strcmp('*'gamma\") < 0'*|*'strcmp(\"aardvark\"'*|*'strcmp('*'alpha\") >= 0'*|*'strcmp(\"zeta\"'*)\n\
  printf '%s\\n' 'fake cc found redundant bounded string comparison operation' >&2\n\
  exit 348\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'strcmp('*'? (\"alpha\") : (\"beta\")'*'? (\"alpha\") : (\"gamma\")'*'== 0'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous bounded string comparison' >&2\n\
  exit 349\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary bounded string comparison folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary bounded string comparison folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary bounded string comparison folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_boolean_expression_comparison_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-boolean-expression-comparison-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary boolean expression comparison folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded boolean expression comparison outputs' >&2\n\
  exit 353\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'== (3)) =='*'!= (4)'*|*'!= (4)) =='*|*'== (3)) !='*|*'!= (4)) !='*)\n\
  printf '%s\\n' 'fake cc found redundant known boolean expression comparison' >&2\n\
  exit 354\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'if ('*'== (3)'*'? (3) : (4)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous boolean expression comparison' >&2\n\
  exit 355\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary boolean expression comparison folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary boolean expression comparison folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary boolean expression comparison folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_boolean_expression_comparison_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-boolean-expression-comparison-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical boolean expression comparison C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded identical boolean expression comparison outputs' >&2\n\
  exit 433\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' == '*|*' != '*|*' < '*|*' <= '*|*' > '*|*' >= '*|*' ? '*)\n\
  printf '%s\\n' 'fake cc found redundant identical boolean expression comparison' >&2\n\
  exit 434\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary identical boolean expression comparison C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical boolean expression comparison C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical boolean expression comparison C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_identical_string_expression_comparison_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-identical-string-expression-comparison-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary identical string expression comparison C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*'printf(\"%lld\", 0)'*'printf(\"%lld\", 1)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded identical string expression comparison outputs' >&2\n\
  exit 435\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'strcmp('*|*' == '*|*' != '*|*' < '*|*' <= '*|*' > '*|*' >= '*|*' ? '*)\n\
  printf '%s\\n' 'fake cc found redundant identical string expression comparison' >&2\n\
  exit 436\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary identical string expression comparison C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary identical string expression comparison C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary identical string expression comparison C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_bounded_integer_comparison_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-bounded-integer-comparison-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary bounded integer comparison folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded bounded integer comparison outputs' >&2\n\
  exit 356\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'> (6)'*|*'< (10)'*|*'(1) <'*|*'>='*)\n\
  printf '%s\\n' 'fake cc found redundant bounded integer comparison operation' >&2\n\
  exit 357\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'? (7) : (8)'*'== (7)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous bounded integer comparison' >&2\n\
  exit 358\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary bounded integer comparison folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary bounded integer comparison folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary bounded integer comparison folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_bounded_float_comparison_folding_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-bounded-float-comparison-folding-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary bounded float comparison folding C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing folded bounded float comparison outputs' >&2\n\
  exit 359\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'> (6.5)'*|*'< (10.5)'*|*'(1.5) <'*|*'>='*)\n\
  printf '%s\\n' 'fake cc found redundant bounded float comparison operation' >&2\n\
  exit 360\n\
  ;;\n\
  *) : ;;\n\
esac\n\
case \"$source\" in\n\
  *'? (7.5) : (8.5)'*'== (7.5)'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing preserved ambiguous bounded float comparison' >&2\n\
  exit 361\n\
  ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary bounded float comparison folding C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary bounded float comparison folding C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary bounded float comparison folding C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_dynamic_boolean_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-mixed-dynamic-bool-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed dynamic boolean strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*'%tmp0 = add i64 1, 2'*'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = select i1 %tmp1'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing mixed dynamic boolean strict-identity outputs' >&2\n\
  exit 183\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'icmp eq i1'*|*'icmp ne i1'*|*'@printf(ptr @.fmt_float'*|*'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found runtime boolean comparison or numeric output for mixed dynamic boolean strict identity' >&2\n\
  exit 184\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary mixed dynamic boolean strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary mixed dynamic boolean strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary mixed dynamic boolean strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_dynamic_boolean_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-mixed-dynamic-bool-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed dynamic boolean strict-identity C-validating cc PATH directory can be created",
        );
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
  *'if ('*'(1 + 2)'*'=='*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing mixed dynamic boolean strict-identity outputs' >&2\n\
  exit 185\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found numeric output for mixed dynamic boolean strict identity' >&2\n\
  exit 186\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary mixed dynamic boolean strict-identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary mixed dynamic boolean strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary mixed dynamic boolean strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_dynamic_float_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-mixed-dynamic-float-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed dynamic float strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*'%tmp0 = fadd double 1.5, 2.25'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing mixed dynamic float strict-identity outputs' >&2\n\
  exit 191\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'fcmp '*|*'select i1'*|*'@printf(ptr @.fmt_float'*|*'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found runtime comparison or numeric output for mixed dynamic float strict identity' >&2\n\
  exit 192\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary mixed dynamic float strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary mixed dynamic float strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary mixed dynamic float strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_dynamic_float_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-mixed-dynamic-float-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed dynamic float strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing mixed dynamic float strict-identity outputs' >&2\n\
  exit 193\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *' == '*|*' != '*|*'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found runtime comparison or numeric output for mixed dynamic float strict identity' >&2\n\
  exit 194\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary mixed dynamic float strict-identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary mixed dynamic float strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary mixed dynamic float strict-identity C-validating cc script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_dynamic_string_strict_identity_ir_validating_successful_clang(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-mixed-dynamic-string-strict-identity-clang-validate-ir-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed dynamic string strict-identity IR-validating clang PATH directory can be created",
        );
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
  *'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"1\\00\"'*'c\"x\\00\"'*'c\"y\\00\"'*'%tmp0 = add i64 1, 2'*'%tmp1 = icmp eq i64 %tmp0, 3'*'%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake clang missing mixed dynamic string strict-identity outputs' >&2\n\
  exit 199\n\
  ;;\n\
esac\n\
case \"$ir\" in\n\
  *'@strcmp'*|*'call i32 @strcmp'*|*'@printf(ptr @.fmt_float'*|*'@printf(ptr @.fmt_int'*)\n\
  printf '%s\\n' 'fake clang found runtime comparison or numeric output for mixed dynamic string strict identity' >&2\n\
  exit 200\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary mixed dynamic string strict-identity IR-validating clang script can be written",
        );
        let mut permissions = fs::metadata(&clang)
            .expect(
                "temporary mixed dynamic string strict-identity IR-validating clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&clang, permissions).expect(
            "temporary mixed dynamic string strict-identity IR-validating clang script can be made executable",
        );
        Self { path }
    }

    fn with_mixed_dynamic_string_strict_identity_c_validating_successful_cc_only(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-cc-mixed-dynamic-string-strict-identity-validate-c-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary mixed dynamic string strict-identity C-validating cc PATH directory can be created",
        );
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
  *'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"1\")'*'printf(\"%s\", \"x\")'*'printf(\"%s\", \"y\")'*) : ;;\n\
  *)\n\
  printf '%s\\n' 'fake cc missing mixed dynamic string strict-identity outputs' >&2\n\
  exit 201\n\
  ;;\n\
esac\n\
case \"$source\" in\n\
  *'strcmp('*|*'#include <string.h>'*|*'printf(\"%g\"'*|*'printf(\"%lld\"'*)\n\
  printf '%s\\n' 'fake cc found runtime comparison or numeric output for mixed dynamic string strict identity' >&2\n\
  exit 202\n\
  ;;\n\
  *) : ;;\n\
esac\n\
printf '%s\\n' '.text'\n\
printf '%s\\n' '.globl main'\n\
printf '%s\\n' 'main:'\n\
printf '%s\\n' '  call printf'\n\
exit 0\n",
        )
        .expect(
            "temporary mixed dynamic string strict-identity C-validating cc script can be written",
        );
        let mut permissions = fs::metadata(&cc)
            .expect(
                "temporary mixed dynamic string strict-identity C-validating cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&cc, permissions).expect(
            "temporary mixed dynamic string strict-identity C-validating cc script can be made executable",
        );
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

    fn with_permission_denied_clang_after_successful_probe_and_available_fallbacks(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-selected-permission-denied-emission-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary selected-permission-denied-emission PATH directory can be created");
        let chmod = find_command_on_path("chmod")
            .expect("host chmod command is available for permission-denied emission test");

        let clang = path.join("clang");
        fs::write(
            &clang,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  '{}' 0644 '{}'\n\
  printf '%s\\n' 'fake clang 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected clang backend invocation after permission removal' >&2\n\
exit 109\n",
                chmod.display(),
                clang.display()
            ),
        )
        .expect("temporary selected-permission-denied-emission clang script can be written");
        let mut clang_permissions = fs::metadata(&clang)
            .expect(
                "temporary selected-permission-denied-emission clang script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut clang_permissions, 0o755);
        fs::set_permissions(&clang, clang_permissions).expect(
            "temporary selected-permission-denied-emission clang script can be made executable",
        );

        for (command, exit_code) in [("llc", 110), ("cc", 111)] {
            let script = path.join(command);
            fs::write(
                &script,
                format!(
                    "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake {command} 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected {command} fallback invocation after selected clang permission-denied start' >&2\n\
exit {exit_code}\n"
                ),
            )
            .expect(
                "temporary selected-permission-denied-emission fallback script can be written",
            );
            let mut permissions = fs::metadata(&script)
                .expect(
                    "temporary selected-permission-denied-emission fallback script metadata is readable",
                )
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&script, permissions).expect(
                "temporary selected-permission-denied-emission fallback script can be made executable",
            );
        }

        Self { path }
    }

    fn with_start_failing_llc_after_successful_probe_and_available_cc(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-fallback-start-failure-precedence-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary fallback-start-failure-precedence PATH directory can be created");

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
exit 107\n",
                llc.display()
            ),
        )
        .expect("temporary fallback-start-failure-precedence llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary fallback-start-failure-precedence llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions).expect(
            "temporary fallback-start-failure-precedence llc script can be made executable",
        );

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after selected llc start failure' >&2\n\
exit 108\n",
        )
        .expect("temporary fallback-start-failure-precedence cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary fallback-start-failure-precedence cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary fallback-start-failure-precedence cc script can be made executable");

        Self { path }
    }

    fn with_permission_denied_llc_after_successful_probe_and_available_cc(
        workspace_root: &Path,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-fallback-llc-permission-denied-emission-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary fallback-llc-permission-denied-emission PATH directory can be created",
        );
        let chmod = find_command_on_path("chmod")
            .expect("host chmod command is available for permission-denied emission test");

        let llc = path.join("llc");
        fs::write(
            &llc,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  '{}' 0644 '{}'\n\
  printf '%s\\n' 'fake llc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected llc backend invocation after permission removal' >&2\n\
exit 112\n",
                chmod.display(),
                llc.display()
            ),
        )
        .expect("temporary fallback-llc-permission-denied-emission llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect(
                "temporary fallback-llc-permission-denied-emission llc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions).expect(
            "temporary fallback-llc-permission-denied-emission llc script can be made executable",
        );

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc fallback invocation after selected llc permission-denied start' >&2\n\
exit 113\n",
        )
        .expect("temporary fallback-llc-permission-denied-emission cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect(
                "temporary fallback-llc-permission-denied-emission cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions).expect(
            "temporary fallback-llc-permission-denied-emission cc script can be made executable",
        );

        Self { path }
    }

    fn with_permission_denied_cc_after_successful_probe(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-fallback-cc-permission-denied-emission-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect(
            "temporary fallback-cc-permission-denied-emission PATH directory can be created",
        );
        let chmod = find_command_on_path("chmod")
            .expect("host chmod command is available for permission-denied emission test");

        let cc = path.join("cc");
        fs::write(
            &cc,
            format!(
                "#!/bin/sh\n\
if [ \"$1\" = \"--version\" ]; then\n\
  '{}' 0644 '{}'\n\
  printf '%s\\n' 'fake cc 0.0'\n\
  exit 0\n\
fi\n\
printf '%s\\n' 'unexpected cc backend invocation after permission removal' >&2\n\
exit 114\n",
                chmod.display(),
                cc.display()
            ),
        )
        .expect("temporary fallback-cc-permission-denied-emission cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect(
                "temporary fallback-cc-permission-denied-emission cc script metadata is readable",
            )
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions).expect(
            "temporary fallback-cc-permission-denied-emission cc script can be made executable",
        );

        Self { path }
    }

    fn with_unstartable_clang_probe_then_fake_llc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-unstartable-clang-probe-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary unstartable-clang-probe PATH directory can be created");

        let clang = path.join("clang");
        write_unstartable_backend_probe(&clang);

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
        .expect("temporary unstartable-clang-probe llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary unstartable-clang-probe llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary unstartable-clang-probe llc script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
printf '%s\\n' 'unexpected cc fallback invocation after unstartable clang probe' >&2\n\
exit 109\n",
        )
        .expect("temporary unstartable-clang-probe unused cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary unstartable-clang-probe unused cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary unstartable-clang-probe unused cc script can be made executable");

        Self { path }
    }

    fn with_unstartable_llvm_probes_then_fake_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-unstartable-llvm-probes-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary unstartable-llvm-probes PATH directory can be created");

        write_unstartable_backend_probe(&path.join("clang"));
        write_unstartable_backend_probe(&path.join("llc"));

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
        .expect("temporary unstartable-llvm-probes cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary unstartable-llvm-probes cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary unstartable-llvm-probes cc script can be made executable");

        Self { path }
    }

    fn with_all_unstartable_backend_probes(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-all-unstartable-probes-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary all-unstartable-probes PATH directory can be created");

        for command in ["clang", "llc", "cc"] {
            write_unstartable_backend_probe(&path.join(command));
        }

        Self { path }
    }

    fn with_permission_denied_clang_probe_then_fake_llc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-permission-denied-clang-probe-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary permission-denied-clang-probe PATH directory can be created");

        write_permission_denied_backend_probe(&path.join("clang"));

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
        .expect("temporary permission-denied-clang-probe llc script can be written");
        let mut llc_permissions = fs::metadata(&llc)
            .expect("temporary permission-denied-clang-probe llc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut llc_permissions, 0o755);
        fs::set_permissions(&llc, llc_permissions)
            .expect("temporary permission-denied-clang-probe llc script can be made executable");

        let cc = path.join("cc");
        fs::write(
            &cc,
            "#!/bin/sh\n\
printf '%s\\n' 'unexpected cc fallback invocation after permission-denied clang probe' >&2\n\
exit 109\n",
        )
        .expect("temporary permission-denied-clang-probe unused cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary permission-denied-clang-probe unused cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions).expect(
            "temporary permission-denied-clang-probe unused cc script can be made executable",
        );

        Self { path }
    }

    fn with_permission_denied_llvm_probes_then_fake_cc(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-permission-denied-llvm-probes-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary permission-denied-llvm-probes PATH directory can be created");

        write_permission_denied_backend_probe(&path.join("clang"));
        write_permission_denied_backend_probe(&path.join("llc"));

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
        .expect("temporary permission-denied-llvm-probes cc script can be written");
        let mut cc_permissions = fs::metadata(&cc)
            .expect("temporary permission-denied-llvm-probes cc script metadata is readable")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut cc_permissions, 0o755);
        fs::set_permissions(&cc, cc_permissions)
            .expect("temporary permission-denied-llvm-probes cc script can be made executable");

        Self { path }
    }

    fn with_all_permission_denied_backend_probes(workspace_root: &Path) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = workspace_root.join("target").join(format!(
            "native-assembly-all-permission-denied-probes-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .expect("temporary all-permission-denied-probes PATH directory can be created");

        for command in ["clang", "llc", "cc"] {
            write_permission_denied_backend_probe(&path.join(command));
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

#[cfg(unix)]
fn write_unstartable_backend_probe(path: &Path) {
    fs::write(path, "#!/nonexistent/phpc-probe-start-failure\n")
        .expect("temporary unstartable backend probe script can be written");
    let mut permissions = fs::metadata(path)
        .expect("temporary unstartable backend probe script metadata is readable")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(path, permissions)
        .expect("temporary unstartable backend probe script can be made executable");
}

#[cfg(unix)]
fn write_permission_denied_backend_probe(path: &Path) {
    fs::write(
        path,
        "#!/bin/sh\n\
printf '%s\\n' 'unexpected permission-denied backend probe execution' >&2\n\
exit 109\n",
    )
    .expect("temporary permission-denied backend probe script can be written");
    let mut permissions = fs::metadata(path)
        .expect("temporary permission-denied backend probe script metadata is readable")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o644);
    fs::set_permissions(path, permissions)
        .expect("temporary permission-denied backend probe script can be made non-executable");
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
