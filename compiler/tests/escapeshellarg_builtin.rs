use php_compiler::run_source;

#[test]
fn escapeshellarg_wraps_unix_single_quoted_arguments() {
    let execution = run_source(
        r#"<?php
var_dump(escapeshellarg("Mr O'Neil"));
var_dump(escapeshellarg("Mr O\\'Neil"));
var_dump(escapeshellarg("%FILENAME"));
var_dump(escapeshellarg(""));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(14) \"'Mr O'\\''Neil'\"\n",
            "string(15) \"'Mr O\\'\\''Neil'\"\n",
            "string(11) \"'%FILENAME'\"\n",
            "string(2) \"''\"\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn escapeshellarg_is_available_as_internal_builtin_metadata() {
    let execution = run_source(
        r#"<?php
$call = "escapeshellarg";
echo function_exists($call) ? "1" : "0";
echo is_callable($call) ? "1" : "0";
$function = new ReflectionFunction($call);
echo ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
echo "|", $call("can't");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/1|'can'\\''t'");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
