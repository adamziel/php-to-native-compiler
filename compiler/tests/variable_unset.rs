use php_compiler::run_source;

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
    let execution = run_source(
        r#"<?php
$value = 1;
unset($value);
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: Undefined variable $value in Command line code on line 4\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
