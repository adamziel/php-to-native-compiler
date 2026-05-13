use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn symbolic_logical_operators_use_truthiness_and_boolean_results() {
    let execution = run_source(
        r#"<?php
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php", [], [1]] as $value) {
    var_dump($value && true);
}
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php", [], [1]] as $value) {
    var_dump($value || false);
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\n"
    );
}

#[test]
fn logical_operators_short_circuit_right_operands() {
    let execution = run_source(
        r#"<?php
function trace($name, $value) {
    echo $name, "\n";
    return $value;
}

var_dump(false && trace("bad-and", true));
var_dump(true || trace("bad-or", false));
var_dump(true && trace("and-rhs", "php"));
var_dump(false || trace("or-rhs", [1]));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(true)\nand-rhs\nbool(true)\nor-rhs\nbool(true)\n"
    );
}

#[test]
fn logical_precedence_matches_symbolic_and_word_operator_boundaries() {
    let execution = run_source(
        r#"<?php
$symbol_or = false || true;
$word_or = false or true;
$symbol_and = true && false;
$word_and = true and false;

var_dump($symbol_or);
var_dump($word_or);
var_dump($symbol_and);
var_dump($word_and);
var_dump(true || false && false);
var_dump((true || false) && false);
var_dump(false or true and false);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\n"
    );
}

#[test]
fn word_logical_operators_short_circuit_assignment_operands() {
    let execution = run_source(
        r#"<?php
$left = "start";
false and $left = "bad-and";
echo $left, "\n";
true or $left = "bad-or";
echo $left, "\n";
true and $left = "ran-and";
echo $left, "\n";
false or $left = "ran-or";
echo $left, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "start\nstart\nran-and\nran-or\n");
}

#[test]
fn emit_ir_rejects_logical_operators_until_lowering_exists() {
    let error = emit_ir_source("<?php\necho true && false;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "logical operators are supported by phpc run but not LLVM IR emission yet"
    );
}
