use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn fatal_stdout(source: &str) -> String {
    let execution = run_source(source).unwrap();
    assert_eq!(execution.exit_code, 255);
    execution.stdout
}

#[test]
fn array_map_invokes_string_named_callbacks_and_preserves_one_array_keys() {
    let source = r#"<?php
function label_value($value) {
    return "mapped:" . $value;
}

$items = [];
$items["first"] = "Ada";
$items[5] = "Bob";
$items["empty"] = "";
$items[] = "Linus";

$mapped = array_map("label_value", $items);
print_r(array_keys($mapped));
echo $mapped["first"], "|", $mapped[5], "|", $mapped["empty"], "|", $mapped[6], "\n";
$mapped[] = "after";
echo $mapped[7], "\n";
print_r($items);

$call = "array_map";
$lengths = $call("strlen", ["empty" => "", "zero" => "0", "space" => " "]);
echo count($lengths), "|", $lengths["empty"], "|", $lengths["zero"], "|", $lengths["space"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => first\n    [1] => 5\n    [2] => empty\n    [3] => 6\n)\nmapped:Ada|mapped:Bob|mapped:|mapped:Linus\nafter\nArray\n(\n    [first] => Ada\n    [5] => Bob\n    [empty] => \n    [6] => Linus\n)\n3|0|1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_null_callback_identity_preserves_one_array_keys_and_values() {
    let source = r#"<?php
$items = [];
$items["first"] = "Ada";
$items[5] = "Bob";
$items["empty"] = "";
$items[] = "Linus";

$identity = array_map(null, $items);
print_r(array_keys($identity));
echo $identity["first"], "|", $identity[5], "|", $identity["empty"], "|", $identity[6], "\n";
$identity[] = "after";
echo $identity[7], "\n";
$identity["first"] = "Changed";
echo $items["first"], "|", $identity["first"], "\n";

$call = "array_map";
$dynamic = $call(null, ["x" => "A", 4 => "B"]);
print_r($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => first\n    [1] => 5\n    [2] => empty\n    [3] => 6\n)\nAda|Bob||Linus\nafter\nAda|Changed\nArray\n(\n    [x] => A\n    [4] => B\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_invokes_string_named_callback_with_two_arrays_and_null_padding() {
    let source = r#"<?php
function pair_label($left, $right) {
    if ($left === null) {
        $left = "NULL";
    }
    if ($right === null) {
        $right = "NULL";
    }
    return $left . ":" . $right;
}

$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$right = [];
$right["a"] = "R1";
$right["b"] = "R2";
$right["c"] = "R3";

$mapped = array_map("pair_label", $left, $right);
print_r(array_keys($mapped));
echo $mapped[0], "|", $mapped[1], "|", $mapped[2], "\n";
print_r($left);
print_r($right);

$call = "array_map";
$dynamic = $call("pair_label", ["x" => "A", "y" => "B", "z" => "C"], ["one" => "1"]);
print_r($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n)\nL1:R1|L2:R2|NULL:R3\nArray\n(\n    [first] => L1\n    [5] => L2\n)\nArray\n(\n    [a] => R1\n    [b] => R2\n    [c] => R3\n)\nArray\n(\n    [0] => A:1\n    [1] => B:NULL\n    [2] => C:NULL\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_requires_array_argument() {
    let stdout = fatal_stdout("<?php\necho array_map(\"strlen\", 42);\n");

    assert!(
        stdout.contains(
            "Fatal error: Uncaught TypeError: array_map(): Argument #2 ($array) must be of type array, int given"
        ),
        "{stdout}"
    );
}

#[test]
fn array_map_reports_php_type_errors_for_non_array_operands() {
    let source = r#"<?php
foreach ([42, false, null, new stdClass] as $value) {
    try {
        array_map(null, $value);
    } catch (Throwable $e) {
        echo $e->getMessage(), "\n";
    }
}

$call = "array_map";
foreach ([42, new stdClass] as $value) {
    try {
        $call("strlen", ["ok"], $value);
    } catch (Throwable $e) {
        echo $e->getMessage(), "\n";
    }
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_map(): Argument #2 ($array) must be of type array, int given\narray_map(): Argument #2 ($array) must be of type array, bool given\narray_map(): Argument #2 ($array) must be of type array, null given\narray_map(): Argument #2 ($array) must be of type array, stdClass given\narray_map(): Argument #3 must be of type array, int given\narray_map(): Argument #3 must be of type array, stdClass given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_callback_requires_string_callable() {
    let stdout = fatal_stdout("<?php\n$items = [\"Ada\"];\necho array_map(42, $items);\n");

    assert!(
        stdout.contains(
            "Fatal error: Uncaught TypeError: array_map(): Argument #1 ($callback) must be a valid callback or null, no array or string given"
        ),
        "{stdout}"
    );

    let execution = run_source(
        "<?php\n$items = [\"Ada\"];\n$callback = fn($value) => $value . \"!\";\n$mapped = array_map($callback, $items);\necho $mapped[0];\n",
    )
    .unwrap();
    assert_eq!(execution.stdout, "Ada!");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_callback_reports_unknown_function() {
    let stdout =
        fatal_stdout("<?php\n$items = [\"Ada\"];\necho array_map(\"missing_map\", $items);\n");

    assert!(
        stdout.contains(
            "Fatal error: Uncaught TypeError: array_map(): Argument #1 ($callback) must be a valid callback or null, function \"missing_map\" not found or invalid function name"
        ),
        "{stdout}"
    );
}

#[test]
fn array_map_reports_php_callback_errors_and_allows_extra_user_args() {
    let source = r#"<?php
class HiddenMap {
    private static function nope($value) {
        return $value;
    }
}

$extra = array_map(function($left) { return $left; }, [1], [2]);
echo $extra[0], "\n";

try {
    array_map("echo", [1]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_map("", [1]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_map(["HiddenMap", "nope"], [1]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_map(42, [1]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_map("pow", [2]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

if (False === false && TRUE === true && NuLl === null) {
    echo "case-insensitive constants\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "1\narray_map(): Argument #1 ($callback) must be a valid callback or null, function \"echo\" not found or invalid function name\narray_map(): Argument #1 ($callback) must be a valid callback or null, function \"\" not found or invalid function name\narray_map(): Argument #1 ($callback) must be a valid callback or null, cannot access private method HiddenMap::nope()\narray_map(): Argument #1 ($callback) must be a valid callback or null, no array or string given\npow() expects exactly 2 arguments, 1 given\ncase-insensitive constants\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_null_callback_zips_two_arrays_with_null_padding() {
    let source = r#"<?php
$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$right = [];
$right["a"] = "R1";
$right["b"] = null;
$right["c"] = "R3";

$mapped = array_map(null, $left, $right);
print_r(array_keys($mapped));
echo count($mapped), "|", count($mapped[0]), "|", count($mapped[1]), "|", count($mapped[2]), "\n";
echo $mapped[0][0], "|", $mapped[0][1], "\n";
if ($mapped[1][1] === null) {
    echo "right-null\n";
}
if ($mapped[2][0] === null) {
    echo "left-null\n";
}
echo $mapped[2][1], "\n";
$mapped[] = ["after"];
echo count($mapped), "|", count($mapped[3]), "|", $mapped[3][0], "\n";
print_r($left);
print_r($right);

$call = "array_map";
$dynamic = $call(null, ["x" => "A", "y" => "B", "z" => "C"], ["one" => "1"]);
echo count($dynamic), "|", count($dynamic[0]), "|", count($dynamic[1]), "|", count($dynamic[2]), "\n";
echo $dynamic[0][0], "|", $dynamic[0][1], "\n";
if ($dynamic[1][1] === null) {
    echo "dynamic-right-null\n";
}
if ($dynamic[2][1] === null) {
    echo $dynamic[2][0], "|dynamic-right-null";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n)\n3|2|2|2\nL1|R1\nright-null\nleft-null\nR3\n4|1|after\nArray\n(\n    [first] => L1\n    [5] => L2\n)\nArray\n(\n    [a] => R1\n    [b] => \n    [c] => R3\n)\n3|2|2|2\nA|1\ndynamic-right-null\nC|dynamic-right-null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_null_callback_zips_variadic_arrays_with_null_padding() {
    let source = r#"<?php
$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$middle = [];
$middle[] = "M1";
$middle[] = "M2";
$middle[] = "M3";

$right = [];
$right["r"] = "R1";

$mapped = array_map(null, $left, $middle, $right);
print_r(array_keys($mapped));
echo count($mapped), "|", count($mapped[0]), "|", count($mapped[1]), "|", count($mapped[2]), "\n";
echo $mapped[0][0], "|", $mapped[0][1], "|", $mapped[0][2], "\n";
if ($mapped[1][2] === null) {
    echo "right-null\n";
}
if ($mapped[2][0] === null) {
    echo "left-null\n";
}
echo $mapped[2][1], "\n";
$mapped[] = ["after"];
echo count($mapped), "|", count($mapped[3]), "|", $mapped[3][0], "\n";
print_r($left);
print_r($middle);
print_r($right);

$call = "array_map";
$dynamic = $call(null, ["x" => "A", "y" => "B"], ["one" => "1"], ["p" => "P", "q" => "Q", "r" => "R"], ["last" => "Z"]);
echo count($dynamic), "|", count($dynamic[0]), "|", count($dynamic[1]), "|", count($dynamic[2]), "\n";
echo $dynamic[0][0], "|", $dynamic[0][1], "|", $dynamic[0][2], "|", $dynamic[0][3], "\n";
if ($dynamic[1][1] === null) {
    echo "dynamic-second-null\n";
}
echo $dynamic[1][0], "|", $dynamic[1][2], "\n";
if ($dynamic[2][0] === null) {
    echo "dynamic-left-null\n";
}
if ($dynamic[2][1] === null) {
    echo "dynamic-second-null-tail\n";
}
if ($dynamic[2][3] === null) {
    echo $dynamic[2][2], "|dynamic-fourth-null";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n)\n3|3|3|3\nL1|M1|R1\nright-null\nleft-null\nM3\n4|1|after\nArray\n(\n    [first] => L1\n    [5] => L2\n)\nArray\n(\n    [0] => M1\n    [1] => M2\n    [2] => M3\n)\nArray\n(\n    [r] => R1\n)\n3|4|4|4\nA|1|P|Z\ndynamic-second-null\nB|Q\ndynamic-left-null\ndynamic-second-null-tail\nR|dynamic-fourth-null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_null_callback_accepts_unpacked_array_argument_list() {
    let source = r#"<?php
$arrays = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
];

$mapped = array_map(null, ...$arrays);
echo count($mapped), "|", count($mapped[0]), "\n";
echo $mapped[0][0], ":", $mapped[0][1], ":", $mapped[0][2], "\n";
echo $mapped[1][0], ":", $mapped[1][1], ":", $mapped[1][2], "\n";
echo $mapped[2][0], ":", $mapped[2][1], ":", $mapped[2][2], "\n";

$call = "array_map";
$dynamic = $call(null, ...$arrays);
echo $dynamic[0][0], ":", $dynamic[1][1], ":", $dynamic[2][2];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "3|3\n1:4:7\n2:5:8\n3:6:9\n1:5:9");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_invokes_string_named_callback_with_variadic_arrays_and_null_padding() {
    let source = r#"<?php
function combine_three($left, $middle, $right) {
    if ($left === null) {
        $left = "NULL";
    }
    if ($middle === null) {
        $middle = "NULL";
    }
    if ($right === null) {
        $right = "NULL";
    }
    return $left . ":" . $middle . ":" . $right;
}

$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$middle = [];
$middle[] = "M1";
$middle[] = "M2";
$middle[] = "M3";

$right = [];
$right["r"] = "R1";

$mapped = array_map("combine_three", $left, $middle, $right);
print_r(array_keys($mapped));
echo count($mapped), "|", $mapped[0], "|", $mapped[1], "|", $mapped[2], "\n";
$mapped[] = "after";
echo count($mapped), "|", $mapped[3], "\n";
print_r($left);
print_r($middle);
print_r($right);

$call = "array_map";
$dynamic = $call("combine_three", ["x" => "A", "y" => "B"], ["one" => "1"], ["p" => "P", "q" => "Q", "r" => "R"]);
echo count($dynamic), "|", $dynamic[0], "|", $dynamic[1], "|", $dynamic[2], "\n";

$builtin = array_map("var_dump", [1], [2], [3]);
echo count($builtin), "|";
if ($builtin[0] === null) {
    echo "builtin-return-null";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n)\n3|L1:M1:R1|L2:M2:NULL|NULL:M3:NULL\n4|after\nArray\n(\n    [first] => L1\n    [5] => L2\n)\nArray\n(\n    [0] => M1\n    [1] => M2\n    [2] => M3\n)\nArray\n(\n    [r] => R1\n)\n3|A:1:P|B:NULL:Q|NULL:NULL:R\nint(1)\nint(2)\nint(3)\n1|builtin-return-null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_requires_third_array_argument() {
    let stdout =
        fatal_stdout("<?php\n$items = [\"Ada\"];\necho array_map(\"strlen\", $items, 42);\n");

    assert!(
        stdout.contains(
            "Fatal error: Uncaught TypeError: array_map(): Argument #3 must be of type array, int given"
        ),
        "{stdout}"
    );
}

#[test]
fn array_map_requires_variadic_array_arguments() {
    let stdout = fatal_stdout(
        "<?php\n$left = [\"Ada\"];\n$middle = [\"Grace\"];\n$right = [\"Linus\"];\necho array_map(null, $left, $middle, $right, 42);\n",
    );

    assert!(
        stdout.contains(
            "Fatal error: Uncaught TypeError: array_map(): Argument #5 must be of type array, int given"
        ),
        "{stdout}"
    );
}

#[test]
fn array_map_variadic_callback_allows_extra_callback_arguments() {
    let execution = run_source(
        "<?php\nfunction combine_two($a, $b) { return $a . ':' . $b; }\n$left = [\"Ada\"];\n$middle = [\"Grace\"];\n$right = [\"Linus\"];\nprint_r(array_map(\"combine_two\", $left, $middle, $right));\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "Array\n(\n    [0] => Ada:Grace\n)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_user_callbacks_with_reference_params_warn_and_receive_values() {
    let execution = run_source(
        r#"<?php
$messages = [];
set_error_handler(function($errno, $errstr) use (&$messages) {
    $messages[] = $errstr;
    return true;
});

function map_ref(&$value) {
    $value = "local-" . $value;
    return $value;
}

$closure = function(&$value) {
    return "closure-" . $value;
};

class MapHelper {
    public static function stat(&$value) {
        return "static-" . $value;
    }

    public function inst(&$value) {
        return "instance-" . $value;
    }
}

$items = ["x" => "one", "y" => "two"];
$mapped = array_map("map_ref", $items);
print_r($mapped);
print_r($items);

$closure_result = array_map($closure, ["c" => "three"]);
$static_result = array_map(["MapHelper", "stat"], ["four"]);
$instance_result = array_map([new MapHelper(), "inst"], ["five"]);
echo $closure_result["c"], "|", $static_result[0], "|", $instance_result[0], "\n";

foreach ($messages as $message) {
    echo $message, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array\n(\n    [x] => local-one\n    [y] => local-two\n)\n\
Array\n(\n    [x] => one\n    [y] => two\n)\n\
closure-three|static-four|instance-five\n\
map_ref(): Argument #1 ($value) must be passed by reference, value given\n\
map_ref(): Argument #1 ($value) must be passed by reference, value given\n\
{closure}(): Argument #1 ($value) must be passed by reference, value given\n\
MapHelper::stat(): Argument #1 ($value) must be passed by reference, value given\n\
MapHelper::inst(): Argument #1 ($value) must be passed by reference, value given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_map_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_map(\"strlen\", [\"name\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );

    let two_array_error =
        emit_ir_source("<?php\necho array_map(\"pair\", [\"left\"], [\"right\"]);\n").unwrap_err();

    assert_eq!(two_array_error.phase, Phase::Codegen);
    assert!(
        two_array_error.message.contains("function calls"),
        "{}",
        two_array_error.message
    );

    let null_callback_error =
        emit_ir_source("<?php\necho array_map(null, [\"name\"]);\n").unwrap_err();

    assert_eq!(null_callback_error.phase, Phase::Codegen);
    assert!(
        null_callback_error.message.contains("function calls"),
        "{}",
        null_callback_error.message
    );

    let null_zip_error =
        emit_ir_source("<?php\necho array_map(null, [\"left\"], [\"right\"]);\n").unwrap_err();

    assert_eq!(null_zip_error.phase, Phase::Codegen);
    assert!(
        null_zip_error.message.contains("function calls"),
        "{}",
        null_zip_error.message
    );

    let null_variadic_zip_error =
        emit_ir_source("<?php\necho array_map(null, [\"left\"], [\"middle\"], [\"right\"]);\n")
            .unwrap_err();

    assert_eq!(null_variadic_zip_error.phase, Phase::Codegen);
    assert!(
        null_variadic_zip_error.message.contains("function calls"),
        "{}",
        null_variadic_zip_error.message
    );

    let callback_variadic_error =
        emit_ir_source("<?php\necho array_map(\"var_dump\", [1], [2], [3]);\n").unwrap_err();

    assert_eq!(callback_variadic_error.phase, Phase::Codegen);
    assert!(
        callback_variadic_error.message.contains("function calls"),
        "{}",
        callback_variadic_error.message
    );
}
