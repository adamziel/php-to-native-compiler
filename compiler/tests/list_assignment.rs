use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_DESTRUCTURING_REJECTION: &str = "LLVM array destructuring lowering rejects list(...) and [...] assignment targets until native array storage layout, ordered key lookup, missing-key diagnostics, nested destructuring, references/copy-on-write, and exact native assignment ordering exist; phpc run handles current simple destructuring assignment behavior";

#[test]
fn simple_positional_list_assignment_reads_numeric_keys() {
    let execution = run_source(
        r#"<?php
list($a, $b) = ["zero", "one", "ignored"];
echo $a, "|", $b, "\n";

$items = ["name" => "Ada", 1 => "one", "0" => "zero"];
list($first, $second) = $items;
echo $first, "|", $second;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "zero|one\nzero|one");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn list_assignment_evaluates_rhs_once_before_left_to_right_writes() {
    let execution = run_source(
        r#"<?php
function pair() {
    echo "rhs\n";
    return [1, 2];
}

list($value, $value) = pair();
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "rhs\n2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn list_assignment_uses_local_function_scope_and_missing_offsets_become_null() {
    let execution = run_source(
        r#"<?php
$a = "global";

function pick() {
    list($a, $b) = ["local"];
    echo $a, "|", $b === null ? "null" : "value", "\n";
}

pick();
echo $a;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "local|null\nglobal");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn list_assignment_rejects_non_array_rhs_before_target_writes() {
    let error = run_source(
        r#"<?php
$a = "old";
$b = "old";
list($a, $b) = 42;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call list(): right-hand side must be array, got int"
    );
}

#[test]
fn emit_ir_rejects_list_assignment_until_native_array_destructuring_exists() {
    let error = emit_ir_source("<?php\nlist($a, $b) = missing_call();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_DESTRUCTURING_REJECTION);
}
