use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const LLVM_OBJECT_PROPERTY_REJECTION: &str = "LLVM object-property lowering rejects instance property reads/writes and dynamic property-name access until native object layout, property tables/slots, visibility checks, magic property hooks, dynamic property policy, references/copy-on-write, and exact native object-property errors exist; phpc run handles current bounded object-property behavior";
const LLVM_OBJECT_METADATA_REJECTION: &str = "LLVM object-metadata lowering rejects object/class metadata builtins until native class metadata tables, object handles, inheritance/interface/trait/enum registries, property/method tables, autoload interaction, references/copy-on-write, and exact native object-metadata errors exist; phpc run handles current bounded object metadata behavior";
const LLVM_INSTANCEOF_REJECTION: &str = "LLVM instanceof lowering rejects class/interface relationship checks until native class metadata tables, object handles, inheritance/interface registries, class-name resolution, autoload interaction, references/copy-on-write, and exact native instanceof diagnostics exist; phpc run handles current bounded instanceof behavior";
const LLVM_CLASS_NAME_CONSTANT_REJECTION: &str = "LLVM class-name constant lowering rejects ClassName::class, self::class, parent::class, and static::class until native class-name resolution, active class/parent and late-static-binding context, namespace/import canonicalization, autoload-free class lookup interaction, references/copy-on-write, and exact native class-name constant diagnostics exist; phpc run handles current bounded class-name constant behavior";
const LLVM_STATIC_MEMBER_REJECTION: &str = "LLVM static-member lowering rejects class constants, static property reads/writes, and dynamic static-property receivers until native class constant tables, static property storage, class context and late-static-binding resolution, visibility checks, autoload/class lookup, references/copy-on-write, and exact native static-member errors exist; phpc run handles current bounded static-member behavior";
const LLVM_METHOD_CALL_REJECTION: &str = "LLVM method-call lowering rejects instance, named static, object static-receiver, self::, parent::, and static:: method calls until native method lookup, receiver/static receiver resolution, $this and late-static-binding context, argument/arity diagnostics, visibility checks, references/copy-on-write, and exact native method-call errors exist; phpc run handles current bounded method-call behavior";
const LLVM_CLONE_REJECTION: &str = "LLVM clone lowering rejects clone expressions, including direct-variable clone assignments that mirror public and context-aware non-public property reference slots, until native object handles, property slot cloning, __clone dispatch, reference-slot metadata, references/copy-on-write, and exact native error behavior exist; phpc run handles current bounded clone behavior";
const LLVM_ARRAY_ACCESS_REJECTION: &str = "LLVM ArrayAccess lowering rejects object offset reads/writes/isset/empty/unset/compound paths until native ArrayAccess dispatch for offsetGet(), offsetSet(), offsetExists(), and offsetUnset(), object handles, references/copy-on-write, and exact PHP diagnostics exist; phpc run handles current bounded ArrayAccess behavior";

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
fn phpc_run_still_handles_current_static_member_subset() {
    let execution = run_source(
        r#"<?php
class Box {
    const NAME = "Box";
    public static $count;
}
Box::$count = 2;
echo Box::class, "\n";
echo Box::NAME, "\n";
echo Box::$count;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Box\nBox\n2");
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
fn emit_ir_rejects_class_declarations_with_instance_defaults_after_parsing() {
    let error = emit_ir_source("<?php\nclass Box { public $name = \"Ada\"; }\necho \"after\";\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_inherited_class_declarations_with_specific_boundary() {
    let error = emit_ir_source("<?php\nclass Base {}\nclass Child extends Base {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_static_members_with_specific_boundary() {
    for source in [
        "<?php\necho Box::NAME;\n",
        "<?php\necho Box::$name;\n",
        "<?php\nBox::$name = \"Ada\";\n",
        "<?php\n$receiver::$name = \"Ada\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_STATIC_MEMBER_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_class_name_constants_with_specific_boundary() {
    for source in [
        "<?php\necho Box::class;\n",
        "<?php\nself::class;\n",
        "<?php\nparent::class;\n",
        "<?php\nstatic::class;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CLASS_NAME_CONSTANT_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_object_instantiation_before_constructor_or_layout_lowering() {
    let error = emit_ir_source("<?php\n$box = new Box();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}

#[test]
fn emit_ir_rejects_constructor_argument_instantiation_before_native_constructor_lowering() {
    let error = emit_ir_source("<?php\n$box = new Box(\"Ada\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}

#[test]
fn emit_ir_rejects_clone_expressions_with_specific_boundary() {
    for source in [
        "<?php\n$copy = clone $object;\n",
        "<?php\necho clone $object;\n",
        "<?php\n$copy = clone missing_object();\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CLONE_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_public_property_reads_and_writes_with_specific_boundary() {
    for source in [
        "<?php\necho $box->name;\n",
        "<?php\n$box->name = \"Ada\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_PROPERTY_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_dynamic_property_reads_and_writes_with_specific_boundary() {
    for source in [
        "<?php\n$name = \"name\";\necho $box->$name;\n",
        "<?php\n$name = \"name\";\n$box->$name = \"Ada\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_PROPERTY_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_object_array_access_offsets_with_specific_boundary() {
    for source in [
        "<?php\necho $holder->bag[\"name\"];\n",
        "<?php\n$holder->bag[\"name\"] = \"Ada\";\n",
        "<?php\n$holder->bag[] = \"Ada\";\n",
        "<?php\necho isset($holder->bag[\"name\"]) ? 1 : 0;\n",
        "<?php\necho empty($holder->bag[\"name\"]) ? 1 : 0;\n",
        "<?php\nunset($holder->bag[\"name\"]);\n",
        "<?php\n$holder->bag[\"name\"] += 1;\n",
        "<?php\necho ($holder->bag[\"name\"] += 1);\n",
        "<?php\n++$holder->bag[\"name\"];\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_ARRAY_ACCESS_REJECTION);
    }
}

#[test]
fn emit_ir_keeps_direct_array_offsets_on_array_boundary() {
    let error = emit_ir_source("<?php\necho $items[\"name\"];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior"
    );
}

#[test]
fn emit_ir_rejects_instance_method_calls_with_specific_boundary() {
    let error = emit_ir_source("<?php\n$box->label();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_METHOD_CALL_REJECTION);
}

#[test]
fn emit_ir_rejects_method_call_receiver_forms_with_specific_boundary() {
    for source in [
        "<?php\n$box->label(\"Ada\");\n",
        "<?php\nBox::label(\"Ada\");\n",
        "<?php\nself::label(\"Ada\");\n",
        "<?php\nparent::label(\"Ada\");\n",
        "<?php\nstatic::label(\"Ada\");\n",
        "<?php\n$receiver::label(\"Ada\");\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_METHOD_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_method_calls_before_lowering_receivers_or_arguments() {
    for source in [
        "<?php\nmissing_receiver()->label([]);\n",
        "<?php\n$box->label([]);\n",
        "<?php\n$receiver::label([]);\n",
        "<?php\nBox::label([]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_METHOD_CALL_REJECTION);
    }
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
        assert_eq!(error.message, LLVM_OBJECT_METADATA_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_builtin_class_metadata_lookups_with_specific_boundary() {
    for source in [
        "<?php\necho class_exists(\"Exception\") ? 1 : 0;\n",
        "<?php\necho property_exists(\"Exception\", \"message\") ? 1 : 0;\n",
        "<?php\necho is_a(\"Exception\", \"Exception\", true) ? 1 : 0;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_METADATA_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_instanceof_with_specific_boundary() {
    let error = emit_ir_source("<?php\n$is = $value instanceof Countable;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_INSTANCEOF_REJECTION);
}

#[test]
fn emit_ir_rejects_instanceof_before_lowering_operands() {
    let error = emit_ir_source("<?php\n$is = missing_value() instanceof Countable;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_INSTANCEOF_REJECTION);
}

#[test]
fn emit_asm_rejects_object_class_features_before_backend_execution() {
    let error = emit_asm_source("<?php\nclass Box { public $name; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_asm_rejects_instanceof_before_backend_execution() {
    let error = emit_asm_source("<?php\n$is = $value instanceof Countable;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_INSTANCEOF_REJECTION);
}

#[test]
fn emit_asm_rejects_class_declarations_with_instance_defaults_after_parsing() {
    let error = emit_asm_source("<?php\nclass Box { public $name = \"Ada\"; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_asm_rejects_inherited_class_declarations_before_backend_execution() {
    let error = emit_asm_source("<?php\nclass Base {}\nclass Child extends Base {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_asm_rejects_constructor_argument_instantiation_before_backend_execution() {
    let error = emit_asm_source("<?php\n$box = new Box(\"Ada\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}

#[test]
fn native_static_member_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1158/native_static_member_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1158/native_static_member_boundary_emit_ir.cli"),
    )
    .expect("native static-member IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_member_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1158/native_static_member_boundary.phpc-source");
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
        workspace_root
            .join("tests/fixtures/milestone1158/native_static_member_boundary_emit_asm.cli"),
    )
    .expect("native static-member assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_object_instantiation_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1153/native_object_instantiation_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1153/native_object_instantiation_boundary_emit_ir.cli"),
    )
    .expect("native object-instantiation IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_object_instantiation_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1153/native_object_instantiation_boundary.phpc-source");
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
        workspace_root
            .join("tests/fixtures/milestone1153/native_object_instantiation_boundary_emit_asm.cli"),
    )
    .expect("native object-instantiation assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn emit_asm_rejects_static_members_before_backend_execution() {
    for source in [
        "<?php\necho Box::NAME;\n",
        "<?php\necho Box::$name;\n",
        "<?php\nBox::$name = \"Ada\";\n",
        "<?php\n$receiver::$name = \"Ada\";\n",
    ] {
        let error = emit_asm_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_STATIC_MEMBER_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_class_name_constants_before_backend_execution() {
    for source in [
        "<?php\necho Box::class;\n",
        "<?php\nself::class;\n",
        "<?php\nparent::class;\n",
        "<?php\nstatic::class;\n",
    ] {
        let error = emit_asm_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CLASS_NAME_CONSTANT_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_clone_expressions_before_backend_execution() {
    let error = emit_asm_source("<?php\n$copy = clone $object;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CLONE_REJECTION);
    let assembly_error = emit_asm_source("<?php\necho clone $object;\n").unwrap_err();

    assert_eq!(assembly_error.phase, Phase::Codegen);
    assert_eq!(assembly_error.message, LLVM_CLONE_REJECTION);
}

#[test]
fn emit_asm_rejects_instance_method_calls_before_backend_execution() {
    let error = emit_asm_source("<?php\n$box->label();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_METHOD_CALL_REJECTION);
}

#[test]
fn emit_asm_rejects_method_call_receiver_forms_before_backend_execution() {
    for source in [
        "<?php\n$box->label(\"Ada\");\n",
        "<?php\nBox::label(\"Ada\");\n",
        "<?php\nself::label(\"Ada\");\n",
        "<?php\nparent::label(\"Ada\");\n",
        "<?php\nstatic::label(\"Ada\");\n",
        "<?php\n$receiver::label(\"Ada\");\n",
    ] {
        let error = emit_asm_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_METHOD_CALL_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_object_array_access_offsets_before_backend_execution() {
    let error = emit_asm_source("<?php\necho $holder->bag[\"name\"];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_ACCESS_REJECTION);
}

#[test]
fn emit_asm_rejects_object_metadata_before_backend_execution() {
    let error = emit_asm_source("<?php\nget_declared_classes();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_METADATA_REJECTION);
}

#[test]
fn emit_asm_rejects_property_reads_and_writes_before_backend_execution() {
    for source in [
        "<?php\necho $box->name;\n",
        "<?php\n$box->name = \"Ada\";\n",
        "<?php\n$name = \"name\";\necho $box->$name;\n",
        "<?php\n$name = \"name\";\n$box->$name = \"Ada\";\n",
    ] {
        let error = emit_asm_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_PROPERTY_REJECTION);
    }
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

#[test]
fn native_object_property_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1148/native_object_property_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1148/native_object_property_boundary_emit_ir.cli"),
    )
    .expect("native object-property IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_object_metadata_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1163/native_object_metadata_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1163/native_object_metadata_boundary_emit_ir.cli"),
    )
    .expect("native object-metadata IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_object_metadata_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1163/native_object_metadata_boundary.phpc-source");
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
        workspace_root
            .join("tests/fixtures/milestone1163/native_object_metadata_boundary_emit_asm.cli"),
    )
    .expect("native object-metadata assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_object_property_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1148/native_object_property_boundary.phpc-source");
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
        workspace_root
            .join("tests/fixtures/milestone1148/native_object_property_boundary_emit_asm.cli"),
    )
    .expect("native object-property assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_array_access_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1104/native_array_access_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1104/native_array_access_boundary_emit_ir.cli"),
    )
    .expect("native ArrayAccess CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_clone_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1100/native_clone_boundary.phpc-source");
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
        workspace_root.join("tests/fixtures/milestone1100/native_clone_boundary_emit_ir.cli"),
    )
    .expect("native clone CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_method_call_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1143/native_method_call_boundary.phpc-source");
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
        workspace_root.join("tests/fixtures/milestone1143/native_method_call_boundary_emit_ir.cli"),
    )
    .expect("native method-call IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_method_call_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1143/native_method_call_boundary.phpc-source");
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
        workspace_root
            .join("tests/fixtures/milestone1143/native_method_call_boundary_emit_asm.cli"),
    )
    .expect("native method-call assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_instanceof_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1178/native_instanceof_boundary.phpc-source",
        "tests/fixtures/milestone1178/native_instanceof_boundary_emit_ir.cli",
        "native instanceof IR CLI snapshot is readable",
    );
}

#[test]
fn native_instanceof_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1178/native_instanceof_boundary.phpc-source",
        "tests/fixtures/milestone1178/native_instanceof_boundary_emit_asm.cli",
        "native instanceof assembly CLI snapshot is readable",
    );
}

#[test]
fn native_class_name_constant_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1183/native_class_name_constant_boundary.phpc-source",
        "tests/fixtures/milestone1183/native_class_name_constant_boundary_emit_ir.cli",
        "native class-name constant IR CLI snapshot is readable",
    );
}

#[test]
fn native_class_name_constant_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1183/native_class_name_constant_boundary.phpc-source",
        "tests/fixtures/milestone1183/native_class_name_constant_boundary_emit_asm.cli",
        "native class-name constant assembly CLI snapshot is readable",
    );
}

fn assert_cli_snapshot_matches(
    mode: &str,
    fixture_path: &str,
    snapshot_path: &str,
    snapshot_context: &str,
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(fixture_path);
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, mode])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(snapshot_path)).expect(snapshot_context);
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
