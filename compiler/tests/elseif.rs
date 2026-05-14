use php_compiler::run_source;

#[test]
fn elseif_chain_takes_first_truthy_branch_and_skips_later_conditions() {
    let execution = run_source(
        r#"<?php
$value = 2;
if ($value == 1) {
    echo "one";
} elseif ($value == 2) {
    echo "two";
} elseif ($missing) {
    echo "missing";
} else {
    echo "else";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "two");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn elseif_chain_falls_through_to_else_when_no_condition_matches() {
    let execution = run_source(
        r#"<?php
$value = 4;
if ($value == 1) {
    echo "one";
} elseif ($value == 2) {
    echo "two";
} elseif ($value == 3) {
    echo "three";
} else {
    echo "else";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "else");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn elseif_chain_supports_single_statement_bodies_and_uppercase_tail_keywords() {
    let execution = run_source(
        r#"<?php
$value = "b";
if ($value == "a") echo "a";
ELSEIF ($value == "b") echo "b";
ELSE echo "else";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn alternate_if_chain_executes_current_conditional_subset() {
    let execution = run_source(
        r#"<?php
$value = 2;
if ($value == 1):
    echo "one";
elseif ($value == 2):
    echo "two";
elseif ($missing):
    echo "missing";
else:
    echo "else";
endif;
echo "\n";

$flag = false;
if ($flag):
    echo "flag";
else:
    echo "fallback";
endif;
echo "\n";

function probe($value) {
    if ($value):
        return "yes";
    else:
        return "no";
    endif;
}
echo probe(true), "\n";
echo probe(false);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "two\nfallback\nyes\nno");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn alternate_if_supports_nested_forms_and_uppercase_tail_keywords() {
    let execution = run_source(
        r#"<?php
$value = 3;
if ($value > 0):
    if ($value == 3):
        echo "nested";
    ELSE:
        echo "other";
    ENDIF;
else:
    echo "outer";
endif;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "nested");
    assert_eq!(execution.exit_code, 0);
}
