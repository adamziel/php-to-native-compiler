use php_compiler::run_source;

#[test]
fn switch_matches_with_loose_scalar_comparison_and_fallthrough() {
    let execution = run_source(
        r#"<?php
$value = "2";
switch ($value) {
    case 1:
        echo "one";
        break;
    case 2:
        echo "two";
    default:
        echo "-default";
    case "tail":
        echo "-tail";
        break;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "two-default-tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn switch_uses_default_only_when_no_case_matches() {
    let execution = run_source(
        r#"<?php
$value = "none";
switch ($value) {
    default:
        echo "default";
    case "none":
        echo "matched";
        break;
}
echo "\n";

$missing = "missing";
switch ($missing) {
    default:
        echo "fallback";
    case "after":
        echo "-after";
        break;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "matched\nfallback-after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn switch_break_does_not_escape_enclosing_loop() {
    let execution = run_source(
        r#"<?php
$i = 0;
while ($i < 3) {
    switch ($i) {
        case 0:
            echo "zero";
            break;
        case 1:
            echo "one";
            break;
        default:
            echo "many";
    }
    echo ":";
    $i = $i + 1;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "zero:one:many:");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn loop_depth_break_can_exit_switch_and_outer_loop() {
    let execution = run_source(
        r#"<?php
$i = 0;
while ($i < 3) {
    switch ($i) {
        case 0:
            echo "zero";
            break 2;
        default:
            echo "other";
    }
    echo "after";
    $i = $i + 1;
}
echo ":done";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "zero:done");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uppercase_switch_and_case_keywords_execute() {
    let execution = run_source(
        r#"<?php
SWITCH (1) {
    CASE 1:
        echo "one";
        break;
    DEFAULT:
        echo "default";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "one");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn switch_accepts_semicolon_case_and_default_separators() {
    let execution = run_source(
        r#"<?php
$value = "2";
switch ($value) {
    case 1;
        echo "one";
        break;
    case 2;
        echo "two";
    default;
        echo "-default";
    case "tail";
        echo "-tail";
        break;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "two-default-tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn alternate_switch_reuses_statement_switch_execution() {
    let execution = run_source(
        r#"<?php
$value = "2";
switch ($value):
    case 1:
        echo "one";
        break;
    case 2;
        echo "two";
    default:
        echo "-default";
    case "tail";
        echo "-tail";
        break;
endswitch;
echo "\n";

$word = "none";
SWITCH ($word):
    DEFAULT;
        echo "fallback";
    CASE "none":
        echo "matched";
        break;
ENDSWITCH;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "two-default-tail\nmatched");
    assert_eq!(execution.exit_code, 0);
}
