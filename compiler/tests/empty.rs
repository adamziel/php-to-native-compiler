use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn empty_direct_variables_and_array_offsets_match_supported_php_subset() {
    let source = r#"<?php
$null = null;
$false = false;
$true = true;
$zero = 0;
$one = 1;
$empty_string = "";
$zero_string = "0";
$text = "text";
$empty_array = [];
$filled_array = [0];

$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["false"] = false;
$items["zero"] = 0;
$items["empty"] = "";
$items["zero_string"] = "0";
$items["empty_array"] = [];
$items["filled_array"] = [0];
$items["2"] = "two";
$key = "present";

if (empty($missing)) {
    echo "missing:empty\n";
}
if (empty($null)) {
    echo "null:empty\n";
}
if (empty($false)) {
    echo "false:empty\n";
}
if (empty($zero)) {
    echo "zero:empty\n";
}
if (empty($empty_string)) {
    echo "empty-string:empty\n";
}
if (empty($zero_string)) {
    echo "zero-string:empty\n";
}
if (empty($true)) {
    echo "true:empty\n";
} else {
    echo "true:not-empty\n";
}
if (empty($one)) {
    echo "one:empty\n";
} else {
    echo "one:not-empty\n";
}
if (empty($text)) {
    echo "text:empty\n";
} else {
    echo "text:not-empty\n";
}
if (empty($empty_array)) {
    echo "empty-array:empty\n";
}
if (empty($filled_array)) {
    echo "filled-array:empty\n";
} else {
    echo "filled-array:not-empty\n";
}
if (empty($items[$key])) {
    echo "offset-present:empty\n";
} else {
    echo "offset-present:not-empty\n";
}
if (empty($items["null"])) {
    echo "offset-null:empty\n";
}
if (empty($items["false"])) {
    echo "offset-false:empty\n";
}
if (empty($items["zero"])) {
    echo "offset-zero:empty\n";
}
if (empty($items["empty"])) {
    echo "offset-empty-string:empty\n";
}
if (empty($items["zero_string"])) {
    echo "offset-zero-string:empty\n";
}
if (empty($items["missing"])) {
    echo "offset-missing:empty\n";
}
if (empty($missing_array[0])) {
    echo "offset-undefined-array:empty\n";
}
$number = 42;
if (empty($number[0])) {
    echo "offset-scalar-target:empty\n";
}
if (empty($items[2])) {
    echo "offset-int-normalized:empty";
} else {
    echo "offset-int-normalized:not-empty";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "missing:empty\nnull:empty\nfalse:empty\nzero:empty\nempty-string:empty\nzero-string:empty\ntrue:not-empty\none:not-empty\ntext:not-empty\nempty-array:empty\nfilled-array:not-empty\noffset-present:not-empty\noffset-null:empty\noffset-false:empty\noffset-zero:empty\noffset-empty-string:empty\noffset-zero-string:empty\noffset-missing:empty\noffset-undefined-array:empty\noffset-scalar-target:empty\noffset-int-normalized:not-empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn complex_empty_operands_remain_explicitly_unsupported() {
    let error = runtime_error("<?php\n$items = [[1]];\necho empty($items[0][0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call empty(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported"
    );
}
