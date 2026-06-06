use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_ARRAY_DESTRUCTURING_REJECTION: &str = "LLVM array destructuring lowering rejects list(...) and [...] assignment targets until native array storage layout, ordered key lookup, missing-key diagnostics, nested destructuring, references/copy-on-write, and exact native assignment ordering exist; phpc run handles current bounded destructuring assignment behavior";
const LLVM_ARRAY_ACCESS_REJECTION: &str = "LLVM ArrayAccess lowering rejects object offset reads/writes/isset/empty/unset/compound paths until native ArrayAccess dispatch for offsetGet(), offsetSet(), offsetExists(), and offsetUnset(), object handles, references/copy-on-write, and exact PHP diagnostics exist; phpc run handles current bounded ArrayAccess behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_NATIVE_ARRAY_NON_LOCAL_ASSIGNMENT_REJECTION: &str = "LLVM native array non-local assignment lowering rejects object, dynamic-object, non-direct object, and static property assignment targets until non-local owner cells, magic property writes, typed/static property state, assignment-expression results, references/copy-on-write, and exact diagnostics share one assignment owner contract; local variables and native array offset assignments use their shared native lvalue assignment contracts";
const LLVM_NATIVE_ARRAY_NON_LOCAL_UNSET_REJECTION: &str = "LLVM native array non-local unset lowering rejects object, dynamic-object, non-direct object, and static property unsets until non-local owner cells, magic __unset dispatch, typed/static property state, references/copy-on-write, and exact diagnostics share one unset owner contract; local variables and native array offset unsets use their shared native lvalue unset contracts";

#[test]
fn phpc_run_still_handles_current_array_subset() {
    let execution = run_source(
        r#"<?php
$items = ["name" => "Ada", 2 => "two", "02" => "zero two"];
$items[] = "next";
$items["name"] = "Grace";
echo $items["name"], "|", $items[2], "|", $items[3], "\n";
unset($items["02"]);
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
print_r(array_values($items));
print_r(array_keys($items));
echo count(array_filter($items)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Grace|two|next\nname=Grace\n2=two\n3=next\nArray\n(\n    [0] => Grace\n    [1] => two\n    [2] => next\n)\nArray\n(\n    [0] => name\n    [1] => 2\n    [2] => 3\n)\n3\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_literals_offsets_iteration_unset_and_builtins() {
    for source in [
        "<?php\necho [1];\n",
        "<?php\n$items = [1];\necho $items[0];\n",
        "<?php\n$items[0] = 1;\n",
        "<?php\nforeach ([1] as $value) { echo $value; }\n",
        "<?php\n$items = [1];\nunset($items[0]);\n",
        "<?php\necho array_values([1]);\n",
        "<?php\necho array_filter([1], \"strlen\");\n",
        "<?php\necho count([1]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_ARRAY_REJECTION);
    }
}

#[test]
fn emit_ir_routes_array_destructuring_rhs_calls_through_call_boundary() {
    let error = emit_ir_source("<?php\nlist($first) = missing_call();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_routes_non_local_assignments_to_shared_assignment_owner_boundary() {
    for source in [
        "<?php\n$box->name = 1;\n",
        "<?php\n$name = \"slot\";\n$box->$name = 1;\n",
        "<?php\n$box->child->name = 1;\n",
        "<?php\nRoot::$name = 1;\n",
        "<?php\nself::$name = 1;\n",
        "<?php\nparent::$name = 1;\n",
        "<?php\nstatic::$name = 1;\n",
        "<?php\necho ($box->name = 1);\n",
        "<?php\necho (Root::$name = 1);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(
            error.message,
            LLVM_NATIVE_ARRAY_NON_LOCAL_ASSIGNMENT_REJECTION
        );
    }

    let error = emit_ir_source("<?php\n$box->items[0] = 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_ACCESS_REJECTION);
}

#[test]
fn emit_ir_routes_non_local_unsets_to_shared_unset_owner_boundary() {
    for source in [
        "<?php\nunset($box->name);\n",
        "<?php\n$name = \"slot\";\nunset($box->$name);\n",
        "<?php\nunset($box->child->name);\n",
        "<?php\nunset(Root::$name);\n",
        "<?php\nunset(self::$name);\n",
        "<?php\nunset(parent::$name);\n",
        "<?php\nunset(static::$name);\n",
        "<?php\nunset($local, $box->name);\n",
        "<?php\nunset($local, Root::$name);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_NATIVE_ARRAY_NON_LOCAL_UNSET_REJECTION);
    }

    let error = emit_ir_source("<?php\nunset($local, $box->items[0]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_ACCESS_REJECTION);
}

#[test]
fn emit_asm_rejects_arrays_before_backend_execution() {
    let error = emit_asm_source("<?php\necho [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}

#[test]
fn emit_asm_rejects_array_destructuring_before_backend_execution() {
    let error = emit_asm_source("<?php\nlist($first) = [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_DESTRUCTURING_REJECTION);
}

#[test]
fn native_array_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone174/native_array_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone174/native_array_boundary_emit_ir.cli"),
    )
    .expect("native array CLI snapshot is readable");
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
