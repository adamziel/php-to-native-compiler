use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";

#[test]
fn phpc_run_still_handles_current_object_class_subset() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
    private $secret;
    public function open() {
        return "unused";
    }
}
$box = new Box();
$box->name = "Ada";
echo get_class($box), "\n";
echo is_object($box), "\n";
echo get_debug_type($box), "\n";
echo class_exists("box"), "\n";
echo property_exists($box, "name"), "\n";
echo method_exists("Box", "OPEN"), "\n";
print_r(get_class_methods($box));
print_r(get_class_vars("Box"));
print_r(get_object_vars($box));
echo "done";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Box\n1\nBox\n1\n1\n1\nArray\n(\n    [0] => open\n)\nArray\n(\n    [name] => \n)\nArray\n(\n    [name] => Ada\n)\ndone"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_class_declarations_with_specific_boundary() {
    let error =
        emit_ir_source("<?php\nclass Box { public $name; }\necho \"after\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_object_instantiation_before_constructor_or_layout_lowering() {
    let error = emit_ir_source("<?php\n$box = new Box();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_constructor_argument_instantiation_before_native_constructor_lowering() {
    let error = emit_ir_source("<?php\n$box = new Box(\"Ada\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_public_property_reads_and_writes_with_specific_boundary() {
    for source in [
        "<?php\necho $box->name;\n",
        "<?php\n$box->name = \"Ada\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_instance_method_calls_with_specific_boundary() {
    let error = emit_ir_source("<?php\n$box->label();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_object_metadata_builtins_before_lowering_arguments() {
    for source in [
        "<?php\necho get_class([]);\n",
        "<?php\necho get_class_methods([]);\n",
        "<?php\necho get_class_vars([]);\n",
        "<?php\necho get_object_vars([]);\n",
        "<?php\necho get_mangled_object_vars([]);\n",
        "<?php\necho get_parent_class([]);\n",
        "<?php\necho get_declared_classes();\n",
        "<?php\necho get_declared_interfaces();\n",
        "<?php\necho get_declared_traits();\n",
        "<?php\necho get_called_class();\n",
        "<?php\necho spl_object_id([]);\n",
        "<?php\necho spl_object_hash([]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_object_class_features_before_backend_execution() {
    let error = emit_asm_source("<?php\nclass Box { public $name; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_asm_rejects_constructor_argument_instantiation_before_backend_execution() {
    let error = emit_asm_source("<?php\n$box = new Box(\"Ada\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_asm_rejects_instance_method_calls_before_backend_execution() {
    let error = emit_asm_source("<?php\n$box->label();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn native_object_class_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone173/native_object_class_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone173/native_object_class_boundary_emit_ir.cli"),
    )
    .expect("native object/class CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
