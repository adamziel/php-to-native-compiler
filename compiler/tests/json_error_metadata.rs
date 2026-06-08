use php_compiler::run_source;

#[test]
fn json_last_error_metadata_reports_initial_no_error_state() {
    let execution = run_source(
        r#"<?php
var_dump(json_last_error(), json_last_error_msg());
foreach (["json_last_error", "json_last_error_msg"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getExtensionName(), ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(0)\nstring(8) \"No error\"\n11:json:0/0;11:json:0/0;"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_last_error_rejects_arguments_as_catchable_type_error() {
    let execution = run_source(
        r#"<?php
try {
    json_last_error(true);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    json_last_error_msg(true);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "json_last_error() expects exactly 0 arguments, 1 given\njson_last_error_msg() expects exactly 0 arguments, 1 given"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
