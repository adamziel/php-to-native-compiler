use php_compiler::run_source;

#[test]
fn settype_invalid_type_and_resource_type_raise_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
try {
    settype($undefined, "unknown");
} catch (ValueError $exception) {
    echo get_class($exception), ":", $exception->getMessage(), "\n";
}

$value = "text";
try {
    settype($value, "resource");
} catch (ValueError $exception) {
    echo get_class($exception), ":", $exception->getMessage(), "\n";
}
var_dump($value);

foreach (["settype", "sleep", "php_uname"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
}
echo "\n";

$reflection = new ReflectionFunction("settype");
echo $reflection->getName(), "|", $reflection->getNumberOfParameters(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "ValueError:settype(): Argument #2 ($type) must be a valid type\n",
            "ValueError:Cannot convert to resource type\n",
            "string(4) \"text\"\n",
            "111111\n",
            "settype|2\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_uname_invalid_modes_raise_value_errors() {
    let execution = run_source(
        r#"<?php
foreach (["", "test", "z"] as $mode) {
    try {
        php_uname($mode);
    } catch (Throwable $exception) {
        echo $exception::class, ": ", $exception->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "ValueError: php_uname(): Argument #1 ($mode) must be a single character\n",
            "ValueError: php_uname(): Argument #1 ($mode) must be a single character\n",
            "ValueError: php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn negative_sleep_reports_php_value_error_with_internal_call_frame() {
    let execution = run_source(
        r#"<?php
sleep(-10);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Fatal error: Uncaught ValueError: sleep(): Argument #1 ($seconds) must be greater than or equal to 0 in Command line code:2\n",
            "Stack trace:\n",
            "#0 Command line code(2): sleep(-10)\n",
            "#1 {main}\n",
            "  thrown in Command line code on line 2",
        )
    );
    assert_eq!(execution.exit_code, 255);
}
