use php_compiler::run_source;

#[test]
fn libxml_empty_error_state_matches_no_parse_metadata_slice() {
    let execution = run_source(
        r#"<?php
var_dump(libxml_use_internal_errors(false));
var_dump(libxml_use_internal_errors(true));
var_dump(libxml_use_internal_errors());
var_dump(libxml_use_internal_errors(null));
var_dump(libxml_get_errors());
var_dump(libxml_get_last_error());
var_dump(libxml_clear_errors());

foreach ([
    "libxml_use_internal_errors",
    "libxml_get_errors",
    "libxml_get_last_error",
    "libxml_clear_errors",
] as $name) {
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
        "bool(false)\nbool(false)\nbool(true)\nbool(true)\narray(0) {\n}\nbool(false)\nNULL\n11:libxml:0/1;11:libxml:0/0;11:libxml:0/0;11:libxml:0/0;"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn libxml_use_internal_errors_tracks_request_local_bool_state() {
    let execution = run_source(
        r#"<?php
$fn = "libxml_use_internal_errors";
var_dump($fn(0));
var_dump($fn("1"));
var_dump($fn("0"));
var_dump($fn());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
