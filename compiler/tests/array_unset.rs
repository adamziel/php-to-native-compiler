use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn unset_direct_array_offsets_removes_existing_keys_and_ignores_missing_keys() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items["city"] = "Paris";
$items["2"] = "two";
$items[] = "next-before";

unset($items["name"]);
unset($items["missing"]);
unset($items[2]);
$items[] = "next-after";

echo "count:", count($items), "\n";
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
if (isset($items["name"])) {
    echo "name:set\n";
} else {
    echo "name:unset\n";
}
if (array_key_exists(2, $items)) {
    echo "two:set\n";
} else {
    echo "two:unset\n";
}
if (array_key_exists(4, $items)) {
    echo "append:4";
} else {
    echo "append:other";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "count:3\ncity=Paris\n3=next-before\n4=next-after\nname:unset\ntwo:unset\nappend:4"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_array_offset_treats_null_or_undefined_targets_as_noop() {
    let source = r#"<?php
$nullable = null;
unset($nullable["missing"]);
unset($undefined["missing"]);

if (isset($nullable)) {
    echo "nullable:set\n";
} else {
    echo "nullable:unset\n";
}
if (isset($undefined)) {
    echo "undefined:set";
} else {
    echo "undefined:unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "nullable:unset\nundefined:unset");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_nested_array_offsets_removes_existing_keys_and_ignores_missing_paths() {
    let source = r#"<?php
$options = [];
$options["group"]["first"] = "one";
$options["group"]["second"] = "two";
$options["other"] = null;

unset($options["group"]["first"]);
unset($options["group"]["missing"]);
unset($options["missing"]["child"]);
unset($options["other"]["child"]);
unset($undefined["missing"]["child"]);

if (array_key_exists("first", $options["group"])) {
    echo "first:set\n";
} else {
    echo "first:unset\n";
}
echo "second:", $options["group"]["second"], "\n";
if (isset($undefined)) {
    echo "undefined:set";
} else {
    echo "undefined:unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "first:unset\nsecond:two\nundefined:unset");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_array_offset_rejects_non_array_targets() {
    let error = runtime_error("<?php\n$value = 1;\nunset($value[0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot unset offset on int"
    );
}

#[test]
fn unset_nested_array_offset_rejects_non_array_intermediate_values() {
    let error =
        runtime_error("<?php\n$items = ['outer' => 1];\nunset($items['outer']['inner']);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot unset offset on int"
    );
}
