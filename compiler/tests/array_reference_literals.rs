use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn array_reference_elements_evaluate_current_values_without_aliasing() {
    let execution = run_source(
        r#"<?php
$value = "Ada";
$items = array(&$value, "name" => &$value);
echo $items[0], "|", $items["name"];
$value = "Grace";
echo "|", $items[0], "|", $items["name"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada|Ada|Ada|Ada");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_keys_remain_explicitly_unsupported() {
    let error = parse_error(
        r#"<?php
$key = "name";
$value = "Ada";
$items = array(&$key => $value);
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 17);
    assert_eq!(
        error.message,
        "unsupported array reference key: reference keys are not implemented"
    );
}
