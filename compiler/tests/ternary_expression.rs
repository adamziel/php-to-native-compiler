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
fn short_ternary_remains_an_explicit_unsupported_boundary() {
    let error = run_source("<?php\n$value = '';\necho $value ?: 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 13);
    assert_eq!(
        error.message,
        "unsupported short ternary expression: short ternary value reuse is not implemented"
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
        "ternary conditional expressions are supported by phpc run but not LLVM IR emission yet"
    );
}
