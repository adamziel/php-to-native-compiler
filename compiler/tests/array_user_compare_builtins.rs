use php_compiler::run_source;

#[test]
fn array_ukey_builtins_compare_keys_with_user_callback() {
    let execution = run_source(
        r#"<?php
function key_cmp($left, $right) {
    if ($left == $right) {
        return 0;
    }
    return ($left > $right) ? 1 : -1;
}

$left = ["a" => 1, "b" => 2, "c" => 3];
$right = ["b" => 20, "c" => 30, "d" => 40];

$intersect = array_intersect_ukey($left, $right, "key_cmp");
echo implode(",", array_keys($intersect)), "|", implode(",", $intersect), "\n";

$diff = array_diff_ukey($left, $right, "key_cmp");
echo implode(",", array_keys($diff)), "|", implode(",", $diff), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "b,c|2,3\na|1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_ukey_argument_type_errors_are_catchable() {
    let execution = run_source(
        r#"<?php
function cmp($left, $right) {
    if ($left == $right) {
        return 0;
    }
    return ($left > $right) ? 1 : -1;
}

try {
    array_intersect_ukey(1, ["a" => 1], "cmp");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_diff_ukey(["a" => 1], false, "cmp");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array_intersect_ukey(): Argument #1 ($array) must be of type array, int given\narray_diff_ukey(): Argument #2 must be of type array, false given\n"
    );
    assert_eq!(execution.exit_code, 0);
}
