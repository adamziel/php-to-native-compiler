use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_find_family_passes_value_and_key_and_short_circuits() {
    let source = r#"<?php
$items = ["a" => 1, "b" => 2, "c" => 3, "d" => 4];

var_dump(array_find($items, fn($value) => $value > 2));
var_dump(array_find_key($items, fn($value, $key) => $key === "c"));
var_dump(array_any($items, fn($value, $key) => $key === "b" && $value === 2));
var_dump(array_all($items, fn($value, $key) => is_string($key) && $value > 0));

try {
    var_dump(array_find($items, function ($value) {
        if ($value > 1) {
            throw new Exception("should-not-run");
        }
        return true;
    }));
} catch (Exception) {
    echo "unexpected\n";
}

try {
    var_dump(array_any($items, function ($value) {
        if ($value === 2) {
            throw new Exception("stop");
        }
        var_dump($value);
        return false;
    }));
} catch (Exception) {
    echo "caught\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "int(3)\nstring(1) \"c\"\nbool(true)\nbool(true)\nint(1)\nint(1)\ncaught\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_find_family_accepts_string_static_array_and_dynamic_callables() {
    let source = r#"<?php
function even_value($value) {
    return $value % 2 === 0;
}

class CheckArrayFind {
    public static function belowTen($value) {
        return $value < 10;
    }
}

$items = ["a" => 1, "b" => 2, "c" => 12];
$find = "array_find";
$any = "array_any";

var_dump($find($items, "even_value"));
var_dump(array_find_key($items, ["CheckArrayFind", "belowTen"]));
var_dump(array_all($items, "CheckArrayFind::belowTen"));
var_dump($any($items, "CheckArrayFind::belowTen"));
echo function_exists("array_find") ? "exists\n" : "missing\n";
echo is_callable("array_find_key") ? "callable\n" : "not-callable\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "int(2)\nstring(1) \"a\"\nbool(false)\nbool(true)\nexists\ncallable\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_find_family_reference_params_warn_and_receive_values() {
    let source = r#"<?php
function find_ref(&$value, &$key) {
    echo "fn:$value:$key\n";
    $value = 99;
    $key = "changed";
    return true;
}

class RefChecker {
    public static function staticFind(&$value, &$key) {
        echo "static-string:$value:$key\n";
        $value = 66;
        $key = "static-string";
        return true;
    }

    public static function staticAny(&$value, &$key) {
        echo "static:$value:$key\n";
        $value = 77;
        $key = "static";
        return true;
    }

    public function instanceAll(&$value, &$key) {
        echo "object:$value:$key\n";
        $value = 55;
        $key = "object";
        return true;
    }
}

$items = ["a" => 1, "b" => 2];
$closure = function (&$value, &$key) {
    echo "closure:$value:$key\n";
    $value = 88;
    $key = "closure";
    return true;
};

var_dump(array_find($items, "find_ref"));
var_dump(array_find_key($items, $closure));
var_dump(array_find($items, "RefChecker::staticFind"));
var_dump(array_any($items, ["RefChecker", "staticAny"]));
$checker = new RefChecker();
var_dump(array_all(["z" => 3], [$checker, "instanceAll"]));
var_dump(array_any([[2, 1]], "sort"));
echo implode(",", array_keys($items)), "|", implode(",", $items), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution
            .stdout
            .matches("must be passed by reference, value given")
            .count(),
        11,
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("fn:1:a\nint(1)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("closure:1:a\nstring(1) \"a\"\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("static:1:a\nbool(true)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("static-string:1:a\nint(1)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("object:3:z\nbool(true)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("sort(): Argument #1 ($array) must be passed by reference, value given"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("a,b|1,2\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_find_family_reports_arity_and_native_lowering_boundary() {
    let execution = run_source("<?php\narray_any([1]);\n").unwrap();
    assert_eq!(execution.exit_code, 255);
    assert!(
        execution.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function array_any(), 1 passed"
        ),
        "{}",
        execution.stdout
    );

    let error = emit_ir_source("<?php\narray_find([1], fn($v) => true);\n").unwrap_err();
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
