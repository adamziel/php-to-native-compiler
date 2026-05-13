use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn ternary_uses_current_truthiness_rules() {
    let execution = run_source(
        r#"<?php
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php", [], [1]] as $value) {
    echo $value ? "T" : "F";
}
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "FFTFTFTFFTFT\n");
}

#[test]
fn ternary_evaluates_only_selected_branch() {
    let execution = run_source(
        r#"<?php
function trace($name, $value) {
    echo $name, "\n";
    return $value;
}

echo (trace("condition-true", true) ? trace("true-branch", "T") : trace("false-branch", "F")), "\n";
echo (trace("condition-false", false) ? trace("missing", $undefined) : trace("false-branch", "F")), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "condition-true\ntrue-branch\nT\ncondition-false\nfalse-branch\nF\n"
    );
}

#[test]
fn ternary_supports_parenthesized_nesting_and_value_contexts() {
    let execution = run_source(
        r#"<?php
echo (true ? (false ? "bad" : "inner") : "outer"), "\n";
echo ((false ? "bad" : true) ? "outer-true" : "outer-false"), "\n";

$items = ["a" => 10, "b" => 20];
$key = false ? "a" : "b";
echo $items[$key], "\n";

$assigned = true ? ($slot = "assigned") : ($slot = "wrong");
echo $assigned, ":", $slot, "\n";
echo strlen(false ? "no" : "four"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inner\nouter-true\n20\nassigned:assigned\n4\n"
    );
}

#[test]
fn short_ternary_reuses_truthy_condition_values_and_fallbacks_for_falsey_values() {
    let execution = run_source(
        r#"<?php
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php"] as $value) {
    var_dump($value ?: "fallback");
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(8) \"fallback\"\nstring(8) \"fallback\"\nbool(true)\nstring(8) \"fallback\"\nint(1)\nstring(8) \"fallback\"\nfloat(0.5)\nstring(8) \"fallback\"\nstring(8) \"fallback\"\nstring(3) \"php\"\n"
    );
}

#[test]
fn short_ternary_evaluates_condition_once_and_fallback_lazily() {
    let execution = run_source(
        r#"<?php
function trace($name, $value) {
    echo $name, "\n";
    return $value;
}

echo trace("truthy-condition", "kept") ?: trace("truthy-fallback", "fallback"), "\n";
echo trace("falsey-condition", "") ?: trace("falsey-fallback", "fallback"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "truthy-condition\nkept\nfalsey-condition\nfalsey-fallback\nfallback\n"
    );
}

#[test]
fn ternary_mixes_with_null_coalescing_and_assignment_branches() {
    let execution = run_source(
        r#"<?php
function trace($name, $value) {
    echo "call:", $name, "\n";
    return $value;
}

echo (($missing ?? false) ? trace("bad-condition-true", "bad") : trace("condition-false", "F")), "\n";
echo (false ? trace("bad-true", "bad") : $missing ?? trace("false-branch-coalesce", "C")), "\n";
echo (true ? $missing ?? trace("true-branch-coalesce", "T") : trace("bad-false", "bad")), "\n";

$left = "left-start";
$right = "right-start";
$picked = true ? ($left = trace("assign-left", "L")) : ($right = trace("assign-right", "R"));
echo $picked, ":", $left, ":", $right, "\n";

$count = 1;
$fallback = null;
$picked = false ? ($count += trace("bad-compound", 10)) : ($fallback ??= trace("coalesce-assign", "ready"));
echo $picked, ":", $count, ":", $fallback, "\n";

echo (($missing ?? "kept") ?: trace("bad-short-fallback", "bad")), "\n";
echo (($empty ?? "") ?: trace("short-fallback", "short"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "call:condition-false\nF\ncall:false-branch-coalesce\nC\ncall:true-branch-coalesce\nT\ncall:assign-left\nL:L:right-start\ncall:coalesce-assign\nready:1:ready\nkept\ncall:short-fallback\nshort"
    );
}

#[test]
fn unparenthesized_nested_ternary_remains_an_explicit_unsupported_boundary() {
    let error =
        run_source("<?php\n$flag = true;\necho $flag ? false ? 'bad' : 'inner' : 'outer';\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 20);
    assert_eq!(
        error.message,
        "unsupported nested ternary expression: parenthesize nested ternary expressions in the current subset"
    );
}

#[test]
fn emit_ir_rejects_ternary_expression_until_lowering_exists() {
    let error = emit_ir_source("<?php\necho true ? 1 : 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM conditional lowering rejects ternary and null coalescing expressions until native PHP truthiness, null-aware lookup, and branch side-effect ordering exist; phpc run handles current conditional expression behavior"
    );
}
