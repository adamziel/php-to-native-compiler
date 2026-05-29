use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn scalar_return_type_cannot_be_part_of_intersection_type() {
    let stderr = fatal_for(
        r#"<?php
function foo(): int&Iterator {}
"#,
        "tests/type_declarations/intersection_invalid_return.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type int cannot be part of an intersection type in tests/type_declarations/intersection_invalid_return.php on line 2"
    );
}

#[test]
fn iterable_parameter_reports_php_canonical_intersection_member_name() {
    let stderr = fatal_for(
        r#"<?php
function foo(iterable&Iterator $value) {}
"#,
        "tests/type_declarations/intersection_invalid_parameter.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type Traversable|array cannot be part of an intersection type in tests/type_declarations/intersection_invalid_parameter.php on line 2"
    );
}

#[test]
fn static_method_return_cannot_be_part_of_intersection_type() {
    let stderr = fatal_for(
        r#"<?php
class A {
    public function foo(): static&Iterator {}
}
"#,
        "tests/type_declarations/intersection_invalid_method.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type static cannot be part of an intersection type in tests/type_declarations/intersection_invalid_method.php on line 3"
    );
}

#[test]
fn scalar_property_type_cannot_be_part_of_intersection_type() {
    let stderr = fatal_for(
        r#"<?php
class A {
    public array&Iterator $items;
}
"#,
        "tests/type_declarations/intersection_invalid_property.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type array cannot be part of an intersection type in tests/type_declarations/intersection_invalid_property.php on line 3"
    );
}
