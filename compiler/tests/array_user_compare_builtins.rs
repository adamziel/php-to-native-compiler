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

#[test]
fn array_udiff_callback_return_and_arity_edges_match_php() {
    let execution = run_source(
        r#"<?php
echo "*** Testing array_udiff() : usage variation ***\n";

$arr1 = array(1);
$arr2 = array(1);

echo "\n-- comparison function with an incorrect return value --\n";
function incorrect_return_value($val1, $val2) {
    return array(1);
}
var_dump(array_udiff($arr1, $arr2, "incorrect_return_value"));

echo "\n-- comparison function taking too many parameters --\n";
function too_many_parameters($val1, $val2, $val3) {
    return 0;
}
try {
    var_dump(array_udiff($arr1, $arr2, "too_many_parameters"));
} catch (Throwable $e) {
    echo "Exception: " . $e->getMessage() . "\n";
}

echo "\n-- comparison function taking too few parameters --\n";
function too_few_parameters($val1) {
    return 0;
}
var_dump(array_udiff($arr1, $arr2, "too_few_parameters"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "*** Testing array_udiff() : usage variation ***\n\n-- comparison function with an incorrect return value --\narray(1) {\n  [0]=>\n  int(1)\n}\n\n-- comparison function taking too many parameters --\nException: Too few arguments to function too_many_parameters(), 2 passed and exactly 3 expected\n\n-- comparison function taking too few parameters --\narray(0) {\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}
