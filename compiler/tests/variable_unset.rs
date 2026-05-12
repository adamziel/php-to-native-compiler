use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn unset_static_variable_removes_symbol_and_undefined_names_are_noops() {
    let source = r#"<?php
$value = "Ada";
$nullable = null;

unset($value);
unset($missing);
unset($nullable);

if (isset($value)) {
    echo "value:set\n";
} else {
    echo "value:unset\n";
}
if (empty($missing)) {
    echo "missing:empty\n";
} else {
    echo "missing:not-empty\n";
}
if (isset($nullable)) {
    echo "nullable:set\n";
} else {
    echo "nullable:unset\n";
}

$value = "Bea";
echo "value=", $value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "value:unset\nmissing:empty\nnullable:unset\nvalue=Bea"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_inside_user_function_removes_only_local_symbol() {
    let source = r#"<?php
$name = "global";

function demo($name) {
    unset($name);
    if (isset($name)) {
        echo "local:set\n";
    } else {
        echo "local:unset\n";
    }
}

demo("local");
echo "global=", $name;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "local:unset\nglobal=global");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reading_variable_after_unset_uses_existing_undefined_variable_diagnostic() {
    let error = runtime_error(
        r#"<?php
$value = 1;
unset($value);
echo $value;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$value'");
}
