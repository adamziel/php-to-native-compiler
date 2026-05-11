use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn lex_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Lex);
    error
}

#[test]
fn materialized_symbol_table_preserves_static_variable_behavior() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
$label = $name . "-static";
$items = [];
$items["label"] = $label;
echo isset($name), "\n";
echo $items["label"], "\n";
function shadow($name = "local") {
    $name = $name . "-scope";
    return $name;
}
echo shadow(), "\n";
echo $name, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\nAda-static\nlocal-scope\nAda\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn variable_variables_are_rejected_with_stable_lex_error() {
    let error = lex_error(
        r#"<?php
$name = "value";
$$name = "dynamic";
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported variable variable: variable variables are not implemented"
    );
}

#[test]
fn include_require_constructs_are_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
include 'config.php';
"#,
            2,
            1,
            "unsupported include: include/require resolution and execution are not implemented",
        ),
        (
            r#"<?php
include_once 'config.php';
"#,
            2,
            1,
            "unsupported include_once: include/require resolution and execution are not implemented",
        ),
        (
            r#"<?php
require 'bootstrap.php';
"#,
            2,
            1,
            "unsupported require: include/require resolution and execution are not implemented",
        ),
        (
            r#"<?php
$ok = require_once 'bootstrap.php';
"#,
            2,
            7,
            "unsupported require_once: include/require resolution and execution are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}
