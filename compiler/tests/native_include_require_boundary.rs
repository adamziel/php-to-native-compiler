use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use php_compiler::codegen::emit_native_executable_c_source;
use php_compiler::error::Phase;
use php_compiler::{
    compilation_unit_with_literal_include_metadata, emit_asm_source, emit_ir_source,
};

const LLVM_REQUIRE_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles the current narrow include/require behavior";
const LLVM_REQUIRE_EXPRESSION_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution for expression forms with include return values, _once de-duplication results, and caller-scope side effects until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles current include/require expression behavior";

#[test]
fn emit_ir_rejects_statement_include_require_with_multifile_boundary() {
    for source in [
        "<?php\nrequire 'bootstrap.php';\n",
        "<?php\nrequire_once 'bootstrap.php';\n",
        "<?php\ninclude 'bootstrap.php';\n",
        "<?php\ninclude_once 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REQUIRE_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_expression_include_require_with_return_value_boundary() {
    for source in [
        "<?php\n$ok = require 'bootstrap.php';\n",
        "<?php\n$ok = require_once 'bootstrap.php';\n",
        "<?php\n$ok = include 'bootstrap.php';\n",
        "<?php\n$ok = include_once 'bootstrap.php';\n",
        "<?php\necho include 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REQUIRE_EXPRESSION_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_expression_include_require_before_backend_execution() {
    let error = emit_asm_source("<?php\n$ok = include 'bootstrap.php';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_REQUIRE_EXPRESSION_REJECTION);
}

#[test]
fn literal_include_graph_metadata_expands_declaration_only_class_files() {
    let dir = include_discovery_fixture_dir("metadata");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        "<?php\nclass IncludedMeta { public const TOKEN = 'T'; public static function mark() { return 'M'; } }\n",
    )
    .expect("write included metadata fixture");
    fs::write(
        &root,
        "<?php\nrequire __DIR__ . '/included.php';\necho class_exists('IncludedMeta') ? 'Y' : 'N';\n",
    )
    .expect("write root metadata fixture");

    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read root metadata fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    assert_eq!(unit.include_metadata.included_files.len(), 1);
    assert_eq!(
        unit.include_metadata.included_files[0].class_names,
        vec!["IncludedMeta".to_string()]
    );
    assert!(
        unit.program
            .statements
            .iter()
            .any(|stmt| matches!(stmt, php_compiler::ast::Stmt::Class(class) if class.name == "IncludedMeta")),
        "literal include should expand to reusable class metadata statements"
    );
}

#[test]
fn literal_include_graph_metadata_expands_declaration_only_trait_and_interface_files() {
    let dir = include_discovery_fixture_dir("class-like-metadata");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "interface IncludedContract { public function label(); }\n",
            "trait IncludedTrait { public function traitLabel() { return 'trait'; } }\n",
        ),
    )
    .expect("write included class-like metadata fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included.php';\n",
            "class UsesIncludedDeclarations { use IncludedTrait; }\n",
        ),
    )
    .expect("write root class-like metadata fixture");

    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read root class-like metadata fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    assert_eq!(unit.include_metadata.included_files.len(), 1);
    assert_eq!(
        unit.include_metadata.included_files[0].interface_names,
        vec!["IncludedContract".to_string()]
    );
    assert_eq!(
        unit.include_metadata.included_files[0].trait_names,
        vec!["IncludedTrait".to_string()]
    );
    assert!(
        unit.program.statements.iter().any(
            |stmt| matches!(stmt, php_compiler::ast::Stmt::Interface(interface) if interface.name == "IncludedContract")
        ),
        "literal include should expand to reusable interface metadata statements"
    );
    assert!(
        unit.program
            .statements
            .iter()
            .any(|stmt| matches!(stmt, php_compiler::ast::Stmt::Trait(trait_decl) if trait_decl.name == "IncludedTrait")),
        "literal include should expand to reusable trait metadata statements"
    );
}

#[test]
fn native_executable_c_source_uses_included_class_metadata_boundary() {
    let dir = include_discovery_fixture_dir("c-source");
    let root = write_supported_include_fixture(&dir);
    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read c-source include fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_declare_user_class_parent_bytes")
            && source.contains("phpc_native_declare_user_class_method_bytes")
            && source.contains("phpc_native_static_property_storage_declare_properties_and_defaults_bytes")
            && source.contains("phpc_native_value_class_metadata_exists_with_autoload_policy_and_diagnostic")
            && source.contains("phpc_native_class_constant_declare_constant_bytes_and_free"),
        "generated C should declare included class/method/property/constant metadata through shared runtime boundaries:\n{source}"
    );
}

#[test]
fn native_executable_c_source_uses_included_trait_metadata_boundary_with_trait_execution() {
    let dir = include_discovery_fixture_dir("trait-c-source");
    let root = write_supported_trait_include_fixture(&dir);
    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read trait c-source include fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_value_class_metadata_exists_with_autoload_policy_and_diagnostic")
            && source.contains(
                "phpc_native_callable_table_register_visibility_staticness_magic_signature_frame_callback_and_free"
            )
            && source.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            ),
        "generated C should accept classes using included trait metadata and route trait methods through declared frames:\n{source}"
    );
}

#[test]
fn native_executable_c_source_uses_included_trait_constructor_metadata_boundary() {
    let dir = include_discovery_fixture_dir("trait-constructor-c-source");
    let root = write_supported_trait_constructor_include_fixture(&dir);
    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read trait constructor c-source include fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_KIND_CONSTRUCTOR")
            && source.contains(
                "phpc_native_callable_table_register_visibility_staticness_magic_signature_frame_callback_and_free"
            )
            && source.contains(
                "phpc_native_constructor_allocation_invoke_value_with_access_context_diagnostic_and_free_scope_receiver_arguments"
            )
            && !source.contains("object-instantiation lowering rejects"),
        "included trait constructors should compose into declared constructor metadata and use shared constructor invocation:\n{source}"
    );
}

#[test]
fn native_executable_c_source_accepts_included_interface_metadata_boundary() {
    let dir = include_discovery_fixture_dir("interface-c-boundary");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        "<?php\ninterface IncludedContract { public function label(); }\n",
    )
    .expect("write included interface fixture");
    fs::write(
        &root,
        "<?php\nrequire __DIR__ . '/included.php';\necho 'after';\n",
    )
    .expect("write root interface fixture");

    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read root interface fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "included interface declarations should compose with generated-C metadata without blocking unrelated root execution:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_included_class_metadata_consumers() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("exe");
    let root = write_supported_include_fixture(&dir);
    let output = dir.join("program");
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile included class metadata executable");
    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output)
        .output()
        .expect("run included class metadata executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "exists|child|static|42|child-const|base-const\n"
    );
}

#[test]
fn emit_exe_links_and_runs_included_trait_method_consumers() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("trait-exe");
    let root = write_supported_trait_include_fixture(&dir);
    let output = dir.join("program");
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile included trait metadata executable");
    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output)
        .output()
        .expect("run included trait metadata executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "trait-meta|trait\n");
}

#[test]
fn emit_exe_links_and_runs_included_trait_constructor_consumers() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("trait-constructor-exe");
    let root = write_supported_trait_constructor_include_fixture(&dir);
    let output = dir.join("program");
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile included trait constructor executable");
    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output)
        .output()
        .expect("run included trait constructor executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "trait-ctor-meta|included-ctor\n"
    );
}

#[test]
fn emit_exe_links_and_runs_side_effecting_literal_include_unit() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-side-effects-exe");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    let output = dir.join("program");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "$value = 'included';\n",
            "echo 'inc:', $value;\n",
        ),
    )
    .expect("write side-effecting include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included.php';\n",
            "echo '|', $value, \"\\n\";\n",
        ),
    )
    .expect("write root side-effecting include fixture");

    let output = compile_exe(&root, &output, "side-effecting include executable");
    let run = Command::new(&output)
        .output()
        .expect("run side-effecting include executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "inc:included|included\n"
    );
}

#[test]
fn emit_exe_links_and_runs_include_once_return_value_and_duplicate_true() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-once-return-exe");
    let root = dir.join("root.php");
    let included = dir.join("value.php");
    let output = dir.join("program");
    fs::write(&included, "<?php\nreturn 'loaded';\n").expect("write include return fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "$first = include_once __DIR__ . '/value.php';\n",
            "$second = include_once __DIR__ . '/value.php';\n",
            "echo $first, '|', $second, \"\\n\";\n",
        ),
    )
    .expect("write include_once return fixture");

    let output = compile_exe(&root, &output, "include_once return executable");
    let run = Command::new(&output)
        .output()
        .expect("run include_once return executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "loaded|1\n");
}

#[test]
fn emit_exe_blocks_dynamic_include_paths_before_native_link() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-dynamic-blocker");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        "<?php\n$name = 'value.php';\n$value = include $name;\necho $value;\n",
    )
    .expect("write dynamic include blocker fixture");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile dynamic include blocker executable");

    assert!(!compile.status.success());
    assert!(
        String::from_utf8_lossy(&compile.stderr).contains("literal same-repository path"),
        "stderr should report dynamic include path blocker:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn emit_exe_blocks_cyclic_once_registry_paths_before_native_link() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-once-cycle-blocker");
    let root = dir.join("root.php");
    let other = dir.join("other.php");
    let output = dir.join("program");
    fs::write(&root, "<?php\nrequire_once __DIR__ . '/other.php';\n")
        .expect("write root cyclic once fixture");
    fs::write(&other, "<?php\nrequire_once __DIR__ . '/root.php';\n")
        .expect("write nested cyclic once fixture");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile cyclic once blocker executable");

    assert!(!compile.status.success());
    assert!(
        String::from_utf8_lossy(&compile.stderr).contains("cyclic include graph"),
        "stderr should report cyclic once registry blocker:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn literal_include_graph_blocks_dynamic_missing_cyclic_and_side_effecting_files() {
    let dir = include_discovery_fixture_dir("blockers");
    fs::write(dir.join("class.php"), "<?php\nclass BlockedMeta {}\n").unwrap();

    let dynamic_root = dir.join("dynamic.php");
    fs::write(&dynamic_root, "<?php\nrequire $file;\n").unwrap();
    let dynamic = include_discovery_error(&dynamic_root);
    assert_eq!(dynamic.phase, Phase::Parse);
    assert!(dynamic.message.contains("literal same-repository path"));

    let missing_root = dir.join("missing.php");
    fs::write(&missing_root, "<?php\nrequire __DIR__ . '/absent.php';\n").unwrap();
    let missing = include_discovery_error(&missing_root);
    assert_eq!(missing.phase, Phase::Io);

    let cycle_a = dir.join("cycle-a.php");
    let cycle_b = dir.join("cycle-b.php");
    fs::write(&cycle_a, "<?php\nrequire __DIR__ . '/cycle-b.php';\n").unwrap();
    fs::write(&cycle_b, "<?php\nrequire __DIR__ . '/cycle-a.php';\n").unwrap();
    let cycle = include_discovery_error(&cycle_a);
    assert_eq!(cycle.phase, Phase::Parse);
    assert!(cycle.message.contains("cyclic include graph"));

    let side_effect_inc = dir.join("side-effect.inc");
    let side_effect_root = dir.join("side-effect.php");
    fs::write(
        &side_effect_inc,
        "<?php\necho 'side effect';\nclass SideEffect {}\n",
    )
    .unwrap();
    fs::write(
        &side_effect_root,
        "<?php\nrequire __DIR__ . '/side-effect.inc';\n",
    )
    .unwrap();
    let side_effect = include_discovery_error(&side_effect_root);
    assert_eq!(side_effect.phase, Phase::Parse);
    assert!(side_effect
        .message
        .contains("top-level executable side effects"));

    let autoload_inc = dir.join("autoload.inc");
    let autoload_root = dir.join("autoload.php");
    fs::write(
        &autoload_inc,
        "<?php\nspl_autoload_register(function ($name) { require __DIR__ . '/' . $name . '.php'; });\nclass AutoloadSideEffect {}\n",
    )
    .unwrap();
    fs::write(
        &autoload_root,
        "<?php\nrequire __DIR__ . '/autoload.inc';\n",
    )
    .unwrap();
    let autoload = include_discovery_error(&autoload_root);
    assert_eq!(autoload.phase, Phase::Parse);
    assert!(autoload.message.contains("autoload registration"));

    let late_root = dir.join("late.php");
    fs::write(
        &late_root,
        "<?php\necho 'before';\nrequire __DIR__ . '/class.php';\n",
    )
    .unwrap();
    let late = include_discovery_error(&late_root);
    assert_eq!(late.phase, Phase::Parse);
    assert!(late
        .message
        .contains("before executable top-level statements"));
}

#[test]
fn native_include_require_expression_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1108/native_include_expression_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1108/native_include_expression_boundary_emit_ir.cli"),
    )
    .expect("native include expression CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn include_discovery_error(path: &Path) -> php_compiler::error::Diagnostic {
    compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(path).expect("read include discovery error fixture"),
        path,
        workspace_root(),
    )
    .unwrap_err()
}

fn compile_exe(root: &Path, output: &Path, label: &str) -> PathBuf {
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("compile {label}: {error}"));
    assert!(
        compile.status.success(),
        "{label} compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    output.to_path_buf()
}

fn write_supported_include_fixture(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("create include discovery fixture dir");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "class IncludedBase { public const BASE_TOKEN = 'base-const'; public static $count = 1; }\n",
            "class IncludedBox extends IncludedBase {\n",
            "    public const TOKEN = 'child-const';\n",
            "    public function label() { return 'child'; }\n",
            "    public static function mark() { return 'static'; }\n",
            "}\n",
        ),
    )
    .expect("write included class fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included.php';\n",
            "echo class_exists('IncludedBox') ? 'exists' : 'missing';\n",
            "echo '|';\n",
            "$box = new IncludedBox();\n",
            "echo $box->label();\n",
            "echo '|';\n",
            "echo IncludedBox::mark();\n",
            "echo '|';\n",
            "IncludedBox::$count = 42;\n",
            "echo IncludedBox::$count;\n",
            "echo '|';\n",
            "echo IncludedBox::TOKEN;\n",
            "echo '|';\n",
            "echo IncludedBox::BASE_TOKEN;\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write root class fixture");
    root
}

fn write_supported_trait_include_fixture(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("create include trait discovery fixture dir");
    let root = dir.join("root.php");
    let included = dir.join("included-trait.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "trait IncludedMetaTrait {\n",
            "    public function traitLabel() { return 'trait'; }\n",
            "}\n",
        ),
    )
    .expect("write included trait fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included-trait.php';\n",
            "class UsesIncludedMetaTrait { use IncludedMetaTrait; }\n",
            "echo class_exists('UsesIncludedMetaTrait') ? 'trait-meta' : 'missing';\n",
            "echo '|';\n",
            "$box = new UsesIncludedMetaTrait();\n",
            "echo $box->traitLabel();\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write root trait fixture");
    root
}

fn write_supported_trait_constructor_include_fixture(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("create include trait constructor discovery fixture dir");
    let root = dir.join("root.php");
    let included = dir.join("included-trait-constructor.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "trait IncludedConstructorTrait {\n",
            "    public function __construct($value) { $this->label = 'included-' . $value; }\n",
            "    public function traitLabel() { return $this->label; }\n",
            "}\n",
        ),
    )
    .expect("write included trait constructor fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included-trait-constructor.php';\n",
            "class UsesIncludedConstructorTrait {\n",
            "    public $label;\n",
            "    use IncludedConstructorTrait;\n",
            "}\n",
            "echo class_exists('UsesIncludedConstructorTrait') ? 'trait-ctor-meta' : 'missing';\n",
            "echo '|';\n",
            "$box = new UsesIncludedConstructorTrait('ctor');\n",
            "echo $box->traitLabel();\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write root trait constructor fixture");
    root
}

fn include_discovery_fixture_dir(name: &str) -> PathBuf {
    let mut dir = workspace_root().join("target/include-class-discovery");
    dir.push(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create include discovery fixture dir");
    dir
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a workspace root")
        .to_path_buf()
}

fn has_cc() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
