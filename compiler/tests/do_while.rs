use php_compiler::run_source;

#[test]
fn do_while_executes_body_before_first_condition_check() {
    let execution = run_source(
        r#"<?php
$i = 3;
do {
    echo $i, ":";
    $i = $i + 1;
} while ($i < 3);
echo "\n", $i, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3:\n4\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn do_while_continue_checks_condition_after_body() {
    let execution = run_source(
        r#"<?php
$i = 0;
do {
    $i = $i + 1;
    if ($i == 2) {
        continue;
    }
    echo $i, ":";
} while ($i < 4);
echo "after:", $i;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1:3:4:after:4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn do_while_break_exits_before_condition_and_uppercase_single_statement_works() {
    let execution = run_source(
        r#"<?php
$i = 0;
do {
    $i = $i + 1;
    if ($i == 3) {
        break;
    }
    echo $i, ":";
} while ($i < 5);
echo "after:", $i, "\n";

DO echo "single"; WHILE (false);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1:2:after:3\nsingle");
    assert_eq!(execution.exit_code, 0);
}
