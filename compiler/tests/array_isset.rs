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
fn complex_array_offset_isset_operands_remain_explicitly_unsupported() {
    let error = runtime_error("<?php\n$items = [[1]];\necho isset($items[0][0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call isset(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported"
    );
}
