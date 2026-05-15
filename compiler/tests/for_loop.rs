use php_compiler::run_source;

#[test]
fn for_loop_executes_initializer_condition_increment_and_body() {
    let execution = run_source(
        r#"<?php
$sum = 0;
for ($i = 0; $i < 4; $i = $i + 1) {
    $sum = $sum + $i;
    echo $i, ":";
}
echo "\n", $sum, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0:1:2:3:\n6\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn for_loop_supports_optional_header_slots_and_single_statement_body() {
    let execution = run_source(
        r#"<?php
$i = 0;
FOR (; $i < 3; $i = $i + 1) echo $i;
echo "\n";

$j = 0;
for (; ; $j = $j + 1) {
    if ($j >= 2) {
        break;
    }
    echo "j", $j, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "012\nj0\nj1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn for_loop_continue_runs_increment_before_next_condition() {
    let execution = run_source(
        r#"<?php
for ($i = 0; $i < 5; $i = $i + 1) {
    if ($i == 1) {
        continue;
    }
    if ($i == 4) {
        break;
    }
    echo $i, ":";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0:2:3:");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn loop_depth_break_exits_nested_control_structures() {
    let execution = run_source(
        r#"<?php
$i = 0;
while ($i < 3) {
    $j = 0;
    while ($j < 3) {
        echo $i, ":", $j, "\n";
        break 2;
    }
    echo "inner-after\n";
    $i = $i + 1;
}
echo "done\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0:0\ndone\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn loop_depth_continue_targets_outer_loop_after_switch() {
    let execution = run_source(
        r#"<?php
for ($i = 0; $i < 4; $i = $i + 1) {
    switch ($i) {
        case 1:
            echo "skip:";
            continue 2;
        default:
            echo $i, ":";
    }
    echo "after:";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0:after:skip:2:after:3:after:");
    assert_eq!(execution.exit_code, 0);
}
