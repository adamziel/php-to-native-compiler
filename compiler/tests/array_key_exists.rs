use php_compiler::run_source;

#[test]
fn array_key_exists_checks_keys_without_null_filtering() {
    let source = r#"<?php
$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["2"] = "two";
$items["02"] = "zero two";
$key = "present";

if (array_key_exists($key, $items)) {
    echo "present:exists\n";
}
if (array_key_exists("null", $items)) {
    echo "null:exists\n";
}
if (isset($items["null"])) {
    echo "null:isset\n";
} else {
    echo "null:not-set\n";
}
if (array_key_exists("missing", $items)) {
    echo "missing:exists\n";
} else {
    echo "missing:absent\n";
}
if (array_key_exists(2, $items)) {
    echo "int-normalized:exists\n";
}
if (array_key_exists("02", $items)) {
    echo "leading-zero-string:exists\n";
}
$exists = "array_key_exists";
if ($exists("present", $items)) {
    echo "dynamic:exists";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "present:exists\nnull:exists\nnull:not-set\nmissing:absent\nint-normalized:exists\nleading-zero-string:exists\ndynamic:exists"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_throws_type_error_for_unsupported_key_types() {
    let execution = run_source(
        "<?php\n$items = [];\ntry { array_key_exists([], $items); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Cannot access offset of type array on array"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_accepts_null_and_bool_key_coercions() {
    let source = r#"<?php
$items = [];
$items[""] = "empty";
$items[0] = "zero";
$items[1] = "one";
$items["01"] = "string one";

if (array_key_exists(null, $items)) {
    echo "null:exists\n";
}
if (array_key_exists(false, $items)) {
    echo "false:exists\n";
}
if (array_key_exists(true, $items)) {
    echo "true:exists\n";
}
if (array_key_exists("01", $items)) {
    echo "string-one:exists\n";
}

$call = "array_key_exists";
if ($call(false, $items)) {
    echo "dynamic:false";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Deprecated: Using null as the key parameter for array_key_exists() is deprecated, use an empty string instead in Command line code on line 8\nnull:exists\nfalse:exists\ntrue:exists\nstring-one:exists\ndynamic:false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_accepts_integral_float_key_coercions() {
    let source = r#"<?php
$items = [];
$items[-1] = "minus";
$items[0] = "zero";
$items[1] = "one";
$items[2] = "two";

if (array_key_exists(1.0, $items)) {
    echo "one:exists\n";
}
if (array_key_exists(2.0, $items)) {
    echo "two:exists\n";
}
if (array_key_exists(-1.0, $items)) {
    echo "minus:exists\n";
}

$call = "array_key_exists";
if ($call(0.0, $items)) {
    echo "dynamic:zero";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "one:exists\ntwo:exists\nminus:exists\ndynamic:zero"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_accepts_lossy_float_key_coercions_with_deprecation() {
    let execution = run_source(
        "<?php\n$items = [0 => \"zero\", 1 => \"one\"];\necho array_key_exists(1.5, $items), \"\\n\";\necho array_key_exists(1.00000000000001, $items), \"\\n\";\necho array_key_exists(1.99999999999999, $items), \"\\n\";\necho array_key_exists(1.2345678900E-10, $items);\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Deprecated: Implicit conversion from float 1.5 to int loses precision in Command line code on line 3\n1\n\nDeprecated: Implicit conversion from float 1.00000000000001 to int loses precision in Command line code on line 4\n1\n\nDeprecated: Implicit conversion from float 1.99999999999999 to int loses precision in Command line code on line 5\n1\n\nDeprecated: Implicit conversion from float 1.23456789E-10 to int loses precision in Command line code on line 6\n1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_casts_stream_resources_after_default_context_id() {
    let execution = run_source(
        "<?php\n$items = [4 => \"default-context\"];\n$stream = fopen(\"php://memory\", \"r\");\necho array_key_exists($stream, $items) ? \"hit\" : \"miss\";\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: Resource ID#5 used as offset, casting to integer (5) in Command line code on line 4\nmiss"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_exists_requires_array_second_argument() {
    let execution = run_source(
        "<?php\ntry { array_key_exists(\"name\", 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array_key_exists(): Argument #2 ($array) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}
