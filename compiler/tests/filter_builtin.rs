use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn filter_metadata_builtins_match_current_ext_filter_table() {
    let execution = run_source(
        r#"<?php
$filters = filter_list();
echo count($filters), "|", $filters[0], "|", $filters[10], "|", $filters[20], "\n";
echo filter_id("stripped"), "|", filter_id("string"), "|", filter_id("url"), "|", filter_id("int"), "\n";
var_dump(filter_id("none"));
var_dump(filter_id(-1));
$call = "filter_id";
echo $call("boolean"), "\n";
echo function_exists("filter_list") ? "fn" : "missing";
echo "|", is_callable("filter_id") ? "callable" : "missing";
$reflection = new ReflectionFunction("filter_id");
echo "|", $reflection->getExtensionName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "21|int|stripped|callback\n513|513|518|257\nbool(false)\nbool(false)\n258\nfn|callable|filter:1/1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_metadata_builtins_reject_forms_outside_current_subset() {
    let arity = run_source("<?php\nfilter_list('extra');\n").unwrap_err();
    assert_eq!(arity.phase, Phase::Runtime);
    assert_eq!(arity.line, 2);
    assert_eq!(
        arity.message,
        "arity mismatch for filter_list(): expected 0 argument(s), got 1"
    );

    let non_scalar = run_source("<?php\nfilter_id([]);\n").unwrap();
    assert_eq!(non_scalar.exit_code, 255);
    assert!(non_scalar.stdout.contains(
        "Fatal error: Uncaught TypeError: filter_id(): Argument #1 ($name) must be of type string, array given"
    ));
}
