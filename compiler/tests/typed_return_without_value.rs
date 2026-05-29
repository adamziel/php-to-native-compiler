use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn typed_function_return_without_value_is_startup_fatal() {
    let stderr = fatal_for(
        r#"<?php
function foo(): int {
    return;
}
"#,
        "tests/type_declarations/typed_return_without_value.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: A function with return type must return a value in tests/type_declarations/typed_return_without_value.php on line 3"
    );
}

#[test]
fn nullable_typed_return_without_value_suggests_return_null() {
    let stderr = fatal_for(
        r#"<?php
function foo(): ?int {
    return;
}
"#,
        "tests/type_declarations/nullable_typed_return_without_value.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: A function with return type must return a value (did you mean \"return null;\" instead of \"return;\"?) in tests/type_declarations/nullable_typed_return_without_value.php on line 3"
    );
}

#[test]
fn typed_method_return_without_value_is_startup_fatal() {
    let stderr = fatal_for(
        r#"<?php
class A {
    public function foo(): string {
        if (true) {
            return;
        }
    }
}
"#,
        "tests/type_declarations/typed_method_return_without_value.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: A function with return type must return a value in tests/type_declarations/typed_method_return_without_value.php on line 5"
    );
}

#[test]
fn void_return_without_value_remains_allowed() {
    let execution = php_compiler::run_source(
        r#"<?php
function foo(): void {
    return;
}
echo "ok";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ok");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
