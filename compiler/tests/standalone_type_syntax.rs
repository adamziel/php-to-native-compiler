use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn nullable_void_return_is_rejected_as_standalone_only() {
    let stderr = fatal_for(
        r#"<?php
function test(): ?void {}
"#,
        "tests/type_declarations/nullable_void.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Void can only be used as a standalone type in tests/type_declarations/nullable_void.php on line 2"
    );
}

#[test]
fn nullable_mixed_parameter_is_rejected_because_mixed_includes_null() {
    let stderr = fatal_for(
        r#"<?php
function foo(?mixed $value) {}
"#,
        "tests/type_declarations/nullable_mixed_parameter.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type mixed cannot be marked as nullable since mixed already includes null in tests/type_declarations/nullable_mixed_parameter.php on line 2"
    );
}

#[test]
fn nullable_mixed_property_is_rejected_because_mixed_includes_null() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public ?mixed $value;
}
"#,
        "tests/type_declarations/nullable_mixed_property.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type mixed cannot be marked as nullable since mixed already includes null in tests/type_declarations/nullable_mixed_property.php on line 3"
    );
}

#[test]
fn mixed_union_return_is_rejected_as_standalone_only() {
    let stderr = fatal_for(
        r#"<?php
function foo(): mixed|string|null {}
"#,
        "tests/type_declarations/mixed_union_return.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type mixed can only be used as a standalone type in tests/type_declarations/mixed_union_return.php on line 2"
    );
}

#[test]
fn void_union_return_is_rejected_as_standalone_only() {
    let stderr = fatal_for(
        r#"<?php
function foo(): T|void {}
"#,
        "tests/type_declarations/void_union_return.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Void can only be used as a standalone type in tests/type_declarations/void_union_return.php on line 2"
    );
}

#[test]
fn never_union_return_is_rejected_as_standalone_only() {
    let stderr = fatal_for(
        r#"<?php
function foo(): T|never {}
"#,
        "tests/type_declarations/never_union_return.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: never can only be used as a standalone type in tests/type_declarations/never_union_return.php on line 2"
    );
}
