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
