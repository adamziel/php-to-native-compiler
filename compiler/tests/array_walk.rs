use php_compiler::run_source;

#[test]
fn array_walk_too_many_arguments_are_catchable_argument_count_errors() {
    let source = r#"<?php
$items = [1];

function needs_three($value, $key, $userdata) {}

try {
    array_walk($items, "needs_three");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk_recursive($items, "needs_three");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk($items, "strval", "userdata", "extra");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk_recursive($items, "strval", "userdata", "extra");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Too few arguments to function needs_three(), 2 passed and exactly 3 expected\nToo few arguments to function needs_three(), 2 passed and exactly 3 expected\narray_walk() expects at most 3 arguments, 4 given\narray_walk_recursive() expects at most 3 arguments, 4 given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_walk_accepts_object_input_with_mangled_property_keys_and_writeback() {
    let source = r#"<?php
class Sample {
    private $secret = 1;
    protected $guard = 2;
    public $open = 3;
}

function label($key) {
    if ($key === "\0Sample\0secret") {
        return "private";
    }
    if ($key === "\0*\0guard") {
        return "protected";
    }
    return $key;
}

function show_and_bump(&$value, $key) {
    echo label($key), "=", $value, "\n";
    $value += 10;
}

$object = new Sample();
array_walk($object, "show_and_bump");
$vars = get_mangled_object_vars($object);
echo $vars["\0Sample\0secret"], "|", $vars["\0*\0guard"], "|", $object->open, "\n";

array_walk_recursive($object, function ($value, $key) {
    echo label($key), ":", $value, "\n";
});
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "private=1\nprotected=2\nopen=3\n11|12|13\nprivate:11\nprotected:12\nopen:13\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_walk_invalid_string_callbacks_are_catchable_type_errors() {
    let source = r#"<?php
$items = [1];

try {
    array_walk($items, "echo");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk_recursive($items, "echo");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_walk(): Argument #2 ($callback) must be a valid callback, function \"echo\" not found or invalid function name\narray_walk_recursive(): Argument #2 ($callback) must be a valid callback, function \"echo\" not found or invalid function name\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_walk_direct_root_tracks_visible_mutation_and_lazy_references() {
    let source = r#"<?php
$items = array_fill(0, 10, 1);
array_walk($items, function ($value, $key) use (&$items) {
    echo $key, "\n";
    unset($items[$key]);
    unset($items[$key + 1]);
    unset($items[$key + 2]);
});
echo "left=", count($items), "\n";

$array = [1, 2, 3];
array_walk($array, function ($value, $key) {
    echo $value, "\n";
    if ($value == 2) {
        $GLOBALS["array"] = [4, 5];
    }
});
echo $array[0], ",", $array[1], "\n";

$array = [1, 2, 3];
$array2 = [4, 5];
array_walk($array, function (&$value, $key) use ($array2) {
    echo $value, "\n";
    if ($value == 2) {
        $GLOBALS["array"] = $array2;
    }
    $value *= 10;
});
echo $array[0], ",", $array[1], "|", $array2[0], ",", $array2[1], "\n";

$data = ["key1" => "val1", ["key2" => "val2"]];
function apply_dumb($item, $key) {}
array_walk_recursive($data, "apply_dumb");
$data2 = $data;
$data2[0] = "altered";
echo $data[0]["key2"], "|", $data2[0], "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "0\n3\n6\n9\nleft=0\n1\n2\n4\n5\n4,5\n1\n2\n4\n5\n40,50|4,5\nval2|altered\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_walk_recursive_direct_root_stops_after_unsets_and_settype_scalar_replacement() {
    let source = r#"<?php
$arr = [
    [1, 2, 3],
    [1, 2, 3],
    [1, 2, 3],
];
array_walk_recursive($arr, function (&$value, $key) use (&$arr) {
    echo $key, "\n";
    unset($arr[$key]);
});
echo "left=", count($arr), "\n";

class CastRoot {
    function __toString() {
        global $arr;
        $arr = 1;
        return "hi";
    }
}

$arr = ["string" => new CastRoot];
try {
    array_walk_recursive($arr, "settype");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "0\n1\n2\nleft=0\nIterated value is no longer an array or object\n"
    );
    assert_eq!(execution.exit_code, 0);
}
