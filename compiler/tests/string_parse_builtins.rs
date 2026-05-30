use php_compiler::run_source;

#[test]
fn parse_str_writes_direct_result_array() {
    let execution = run_source(
        r#"<?php
parse_str("first=val1&arr[1]=sid&arr[]=bill&name=O%27Reilly", $out);
var_dump($out);
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("[\"first\"]=>\n  string(4) \"val1\""));
    assert!(execution.stdout.contains("[\"arr\"]=>\n  array(2)"));
    assert!(execution
        .stdout
        .contains("[\"name\"]=>\n  string(8) \"O'Reilly\""));
}

#[test]
fn parse_str_reflection_result_parameter_matches_php_shape() {
    let execution = run_source(
        r#"<?php
$function = new ReflectionFunction("parse_str");
foreach ($function->getParameters() as $parameter) {
    echo $parameter->getName(), " byref=", $parameter->isPassedByReference() ? "1" : "0";
    echo " variadic=", $parameter->isVariadic() ? "1" : "0";
    echo " hasType=", $parameter->hasType() ? "1" : "0";
    echo " optional=", $parameter->isOptional() ? "1" : "0";
    echo "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string byref=0 variadic=0 hasType=1 optional=0\n",
            "result byref=1 variadic=0 hasType=0 optional=0\n",
        )
    );
}

#[test]
fn str_getcsv_reuses_bounded_csv_record_parser() {
    let execution = run_source(
        r#"<?php
var_dump(str_getcsv('foo||bar', '|', '"', ''));
var_dump(str_getcsv('', ',', '"', ''));
try {
    str_getcsv('csv_string', 'separator', '"', '');
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(3) {\n",
            "  [0]=>\n",
            "  string(3) \"foo\"\n",
            "  [1]=>\n",
            "  string(0) \"\"\n",
            "  [2]=>\n",
            "  string(3) \"bar\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  NULL\n",
            "}\n",
            "str_getcsv(): Argument #2 ($separator) must be a single character\n",
        )
    );
}

#[test]
fn str_getcsv_accepts_direct_named_escape_and_fills_defaults() {
    let execution = run_source(
        r#"<?php
var_dump(str_getcsv('"f", "o", ""', escape: ''));
var_dump(str_getcsv('foo||bar', '|', escape: ''));
var_dump(str_getcsv('.foo..bar.', '.', '.', '.'));
var_dump(str_getcsv('', escape: ''));
echo str_pad("x", length: 3, pad_type: STR_PAD_LEFT), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(3) {\n",
            "  [0]=>\n",
            "  string(1) \"f\"\n",
            "  [1]=>\n",
            "  string(1) \"o\"\n",
            "  [2]=>\n",
            "  string(0) \"\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  string(3) \"foo\"\n",
            "  [1]=>\n",
            "  string(0) \"\"\n",
            "  [2]=>\n",
            "  string(3) \"bar\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(7) \"foo.bar\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  NULL\n",
            "}\n",
            "  x\n",
        )
    );
}

#[test]
fn strpbrk_returns_suffix_from_first_matching_byte() {
    let execution = run_source(
        r#"<?php
var_dump(strpbrk("This is a Simple text.", "mi"));
var_dump(strpbrk("This is a Simple text.", "Z"));
try {
    strpbrk("This is a Simple text.", "");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(20) \"is is a Simple text.\"\n",
            "bool(false)\n",
            "strpbrk(): Argument #2 ($characters) must be a non-empty string\n",
        )
    );
}
