use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn logical_xor_uses_truthiness_and_boolean_results() {
    let execution = run_source(
        r#"<?php
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php", [], [1]] as $value) {
    var_dump($value xor true);
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\n"
    );
}

#[test]
fn logical_xor_evaluates_both_operands() {
    let execution = run_source(
        r#"<?php
function trace($name, $value) {
    echo $name, "\n";
    return $value;
}

var_dump(false xor trace("rhs-true", true));
var_dump(true xor trace("rhs-true-again", true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "rhs-true\nbool(true)\nrhs-true-again\nbool(false)\n"
    );
}

#[test]
fn logical_xor_precedence_matches_php_word_operator_boundaries() {
    let execution = run_source(
        r#"<?php
$word_xor_false = false xor true;
$word_xor_true = true xor false;

var_dump($word_xor_false);
var_dump($word_xor_true);
var_dump(true xor false and false);
var_dump(true or true xor true);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(true)\nbool(true)\nbool(true)\n"
    );
}

#[test]
fn logical_xor_accepts_assignment_expression_operands() {
    let execution = run_source(
        r#"<?php
$left = false;
$right = false;
var_dump(($left = true) xor ($right = false));
var_dump($left);
var_dump($right);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "bool(true)\nbool(true)\nbool(false)\n");
}

#[test]
fn emit_ir_rejects_logical_xor_that_needs_php_truthiness() {
    let error = emit_ir_source(
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0 : 5;\necho $value xor true;\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM logical lowering rejects unsupported logical operands until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior"
    );
}
