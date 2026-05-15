use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn direct_array_offset_isset_matches_supported_php_subset() {
    let source = r#"<?php
$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["empty"] = "";
$items["zero"] = 0;
$items["false"] = false;
$items["2"] = "two";
$key = "present";

if (isset($items[$key])) {
    echo "present:set\n";
}
if (isset($items["null"])) {
    echo "null:set\n";
} else {
    echo "null:unset\n";
}
if (isset($items["missing"])) {
    echo "missing:set\n";
} else {
    echo "missing:unset\n";
}
if (isset($missing[0])) {
    echo "undefined:set\n";
} else {
    echo "undefined:unset\n";
}
$number = 42;
if (isset($number[0])) {
    echo "scalar:set\n";
} else {
    echo "scalar:unset\n";
}
$nullable = null;
if (isset($nullable[0])) {
    echo "nullable:set\n";
} else {
    echo "nullable:unset\n";
}
if (isset($items["present"], $items["empty"], $items["zero"], $items["false"])) {
    echo "multi:set\n";
}
if (isset($items["present"], $items["null"])) {
    echo "multi-null:set\n";
} else {
    echo "multi-null:unset\n";
}
if (isset($items[2])) {
    echo "int-normalized:set";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "present:set\nnull:unset\nmissing:unset\nundefined:unset\nscalar:unset\nnullable:unset\nmulti:set\nmulti-null:unset\nint-normalized:set"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn nested_array_offset_isset_matches_supported_php_subset() {
    let source = r#"<?php
$items = [];
$items["present"] = [];
$items["present"]["leaf"] = "value";
$items["present"]["null"] = null;
$items["present"]["false"] = false;
$items["empty"] = [];
$items["empty"]["leaf"] = "value";
$items["scalar"] = 42;
$items["nullable"] = null;
$outer = "present";
$inner = "leaf";

if (isset($items[$outer][$inner])) {
    echo "nested:set\n";
}
if (isset($items[$outer]["null"])) {
    echo "nested-null:set\n";
} else {
    echo "nested-null:unset\n";
}
if (isset($items[$outer]["false"])) {
    echo "nested-false:set\n";
}
if (isset($items["missing"]["leaf"])) {
    echo "missing-parent:set\n";
} else {
    echo "missing-parent:unset\n";
}
if (isset($items["scalar"]["leaf"])) {
    echo "scalar-parent:set\n";
} else {
    echo "scalar-parent:unset\n";
}
if (isset($items["nullable"]["leaf"])) {
    echo "nullable-parent:set\n";
} else {
    echo "nullable-parent:unset\n";
}
if (isset($missing[$outer][$inner])) {
    echo "missing-root:set\n";
} else {
    echo "missing-root:unset\n";
}
if (isset($items[$outer][$inner], $items["empty"]["leaf"])) {
    echo "multi:set";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "nested:set\nnested-null:unset\nnested-false:set\nmissing-parent:unset\nscalar-parent:unset\nnullable-parent:unset\nmissing-root:unset\nmulti:set"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn nested_array_offset_isset_evaluates_indices_left_to_right() {
    let source = r#"<?php
function key_name($name) {
    echo $name;
    return $name;
}
if (isset($missing[key_name("outer")][key_name("inner")])) {
    echo ":set";
} else {
    echo ":unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "outerinner:unset");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_variable_array_offset_isset_operands_remain_explicitly_unsupported() {
    let error = runtime_error("<?php\nfunction make() { return [1]; }\necho isset(make()[0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call isset(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported"
    );
}
