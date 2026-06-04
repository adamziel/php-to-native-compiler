use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_DESTRUCTURING_REJECTION: &str = "LLVM array destructuring lowering rejects list(...) and [...] assignment targets until native array storage layout, ordered key lookup, missing-key diagnostics, nested destructuring, references/copy-on-write, and exact native assignment ordering exist; phpc run handles current bounded destructuring assignment behavior";

#[test]
fn simple_positional_list_assignment_reads_numeric_keys() {
    let execution = run_source(
        r#"<?php
list($a, $b) = ["zero", "one", "ignored"];
echo $a, "|", $b, "\n";

$items = ["name" => "Ada", 1 => "one", "0" => "zero"];
list($first, $second) = $items;
echo $first, "|", $second, "\n";

list(, $textdomain, $language) = ["full-match", "default", "en_US"];
echo $textdomain, "|", $language, "\n";

list($left, , $right,) = ["left", "skip", "right"];
echo $left, "|", $right;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "zero|one\nzero|one\ndefault|en_US\nleft|right"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn short_positional_list_assignment_aliases_current_list_subset() {
    let execution = run_source(
        r#"<?php
[$a, $b] = ["zero", "one", "ignored"];
echo $a, "|", $b, "\n";

$items = ["name" => "Ada", 1 => "one", "0" => "zero"];
[$first, $second] = $items;
echo $first, "|", $second, "\n";

[, $textdomain, $language] = ["full-match", "default", "en_US"];
echo $textdomain, "|", $language, "\n";

[$left, , $right,] = ["left", "skip", "right"];
echo $left, "|", $right;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "zero|one\nzero|one\ndefault|en_US\nleft|right"
    );
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

    assert_eq!(
        execution.stdout,
        "Warning: Undefined array key 1 in Command line code on line 5\nlocal|null\nglobal"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn list_assignment_non_array_rhs_emits_warnings_and_assigns_null() {
    let execution = run_source(
        r#"<?php
$a = "old";
$b = "old";
list($a, $b) = 42;
echo $a === null ? "null" : "value", "|", $b === null ? "null" : "value";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: Cannot use int as array in Command line code on line 4\n\nWarning: Cannot use int as array in Command line code on line 4\nnull|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn list_assignment_supports_nested_keyed_and_missing_slots() {
    let execution = run_source(
        r#"<?php
$data = [
    "names" => ["first" => "Ada", "last" => "Lovelace"],
    "values" => [1, 2],
];

list(
    "names" => list("first" => $first, "last" => $last),
    "values" => [$one, $two],
    "missing" => $missing
) = $data;

echo $first, "|", $last, "|", $one, "|", $two, "|", $missing === null ? "null" : "value";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: Undefined array key \"missing\" in Command line code on line 7\nAda|Lovelace|1|2|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn list_assignment_collects_nested_writes_before_mutating_targets() {
    let execution = run_source(
        r#"<?php
$a = [[1, 2], 3];
list(list($a, $b), $c) = $a;
echo $a, "|", $b, "|", $c, "\n";

$b = [1, [2, 3]];
list($a, list($b, $c)) = $b;
echo $a, "|", $b, "|", $c;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|2|3\n1|2|3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_list_assignment_until_native_array_destructuring_exists() {
    let error = emit_ir_source("<?php\nlist($a, $b) = [1, 2];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_DESTRUCTURING_REJECTION);

    let short_error = emit_ir_source("<?php\n[$a, $b] = [1, 2];\n").unwrap_err();

    assert_eq!(short_error.phase, Phase::Codegen);
    assert_eq!(short_error.line, 2);
    assert_eq!(short_error.column, 1);
    assert_eq!(short_error.message, LLVM_ARRAY_DESTRUCTURING_REJECTION);
}
