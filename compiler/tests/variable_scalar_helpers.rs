use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn boolval_covers_current_value_truthiness_subset() {
    let execution = run_source(
        r#"<?php
var_dump(boolval(false));
var_dump(boolval(null));
var_dump(boolval(""));
var_dump(boolval(0));
var_dump(boolval([]));
var_dump(boolval(true));
var_dump(boolval("abc"));
var_dump(boolval(0.5));
var_dump(boolval(100));
var_dump(boolval(new stdClass()));
var_dump(boolval(STDIN));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn floatval_and_doubleval_cover_basic_scalar_and_resource_subset() {
    let execution = run_source(
        r#"<?php
$values = [
    0.0,
    1.2e3,
    1e-5,
    "10.2 Some Dollars",
    "bob-1.3e3",
    STDERR,
    true,
    null,
];
foreach ($values as $value) {
    var_dump(floatval($value));
}
foreach ($values as $value) {
    var_dump(doubleval($value));
}
"#,
    )
    .unwrap();

    let expected = concat!(
        "float(0)\n",
        "float(1200)\n",
        "float(1.0E-5)\n",
        "float(10.2)\n",
        "float(0)\n",
        "float(3)\n",
        "float(1)\n",
        "float(0)\n",
    );
    assert_eq!(execution.stdout, format!("{expected}{expected}"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn scalar_helper_metadata_and_dynamic_calls_are_available() {
    let execution = run_source(
        r#"<?php
foreach (["boolval", "floatval", "doubleval"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
$call = "boolval";
echo "|", $call("0") ? "true" : "false";
$call = "floatval";
echo "|", $call("-.5e+7 tail");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/1;11:1/1;11:1/1;|false|-5000000");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn settype_reports_current_catchable_value_errors_only() {
    let execution = run_source(
        r#"<?php
$value = 1;
try {
    settype($value, "unknown");
} catch (ValueError $exception) {
    echo $exception->getMessage(), "\n";
}
try {
    settype($value, "resource");
} catch (ValueError $exception) {
    echo $exception->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "settype(): Argument #2 ($type) must be a valid type\nCannot convert to resource type\n"
    );
    assert_eq!(execution.exit_code, 0);

    let error = run_source("<?php\n$value = 1;\nsettype($value, \"int\");\n").unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(
        error.message,
        "unsupported call settype(): value-changing conversions are not implemented in the current subset"
    );
}
