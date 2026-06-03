use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn duplicate_builtin_type_in_union_is_redundant() {
    let stderr = fatal_for(
        r#"<?php
function test(): int|INT {}
"#,
        "tests/type_declarations/union_duplicate_int.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Duplicate type int is redundant in tests/type_declarations/union_duplicate_int.php on line 2"
    );
}

#[test]
fn bool_with_true_reports_true_redundant() {
    let stderr = fatal_for(
        r#"<?php
function test(): bool|true {}
"#,
        "tests/type_declarations/union_bool_true.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Duplicate type true is redundant in tests/type_declarations/union_bool_true.php on line 2"
    );
}

#[test]
fn true_with_false_requires_bool() {
    let stderr = fatal_for(
        r#"<?php
function test(): false|true {}
"#,
        "tests/type_declarations/union_true_false.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Type contains both true and false, bool must be used instead in tests/type_declarations/union_true_false.php on line 2"
    );
}

#[test]
fn iterable_redundant_with_array_or_traversable() {
    let stderr = fatal_for(
        r#"<?php
function test(): iterable|Traversable|ArrayAccess {}
"#,
        "tests/type_declarations/union_iterable_traversable.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Duplicate type Traversable is redundant in tests/type_declarations/union_iterable_traversable.php on line 2"
    );
}

#[test]
fn duplicate_iterable_reports_expanded_array_alias() {
    let stderr = fatal_for(
        r#"<?php
function test(): iterable|iterable|null {}
"#,
        "Zend/tests/type_declarations/iterable/iterable_alias_redundancy_iterable.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Duplicate type array is redundant in Zend/tests/type_declarations/iterable/iterable_alias_redundancy_iterable.php on line 2"
    );
}

#[test]
fn object_with_class_type_is_redundant() {
    let stderr = fatal_for(
        r#"<?php
function test(): object|Test {}
"#,
        "tests/type_declarations/union_object_class.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Type Test|object contains both object and a class type, which is redundant in tests/type_declarations/union_object_class.php on line 2"
    );
}

#[test]
fn object_with_iterable_and_class_displays_expanded_alias_members() {
    let stderr = fatal_for(
        r#"<?php
function test(): object|iterable|T|null {}
"#,
        "Zend/tests/type_declarations/iterable/iterable_alias_redundancy_object.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Type Traversable|T|object|array|null contains both object and a class type, which is redundant in Zend/tests/type_declarations/iterable/iterable_alias_redundancy_object.php on line 2"
    );
}

#[test]
fn relative_self_duplicate_resolves_to_class_name() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public function method(): self|Foo {}
}
"#,
        "tests/type_declarations/union_relative_self.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Duplicate type Foo is redundant in tests/type_declarations/union_relative_self.php on line 3"
    );
}

#[test]
fn dnf_duplicate_intersection_arm_reports_redundant_with_type() {
    let stderr = fatal_for(
        r#"<?php
interface X {}
use A as B;
function foo(): (X&A)|(X&B) {}
"#,
        "Zend/tests/type_declarations/dnf_types/redundant_types/duplicate_class_alias_type.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Type X&A is redundant with type X&A in Zend/tests/type_declarations/dnf_types/redundant_types/duplicate_class_alias_type.php on line 4"
    );
}

#[test]
fn dnf_rejects_more_restrictive_intersection_arm_against_subset() {
    let first_order = fatal_for(
        r#"<?php
interface A {}
interface B {}
function test(): (A&B)|A {}
"#,
        "Zend/tests/type_declarations/dnf_types/redundant_types/less_restrictive_first.php",
    );
    assert_eq!(
        first_order,
        "Fatal error: Type A&B is redundant as it is more restrictive than type A in Zend/tests/type_declarations/dnf_types/redundant_types/less_restrictive_first.php on line 4"
    );

    let second_order = fatal_for(
        r#"<?php
interface A {}
interface B {}
interface C {}
function test(): (A&B&C)|(A&B) {}
"#,
        "Zend/tests/type_declarations/dnf_types/redundant_types/less_restrictive_second.php",
    );
    assert_eq!(
        second_order,
        "Fatal error: Type A&B&C is redundant as it is more restrictive than type A&B in Zend/tests/type_declarations/dnf_types/redundant_types/less_restrictive_second.php on line 5"
    );
}

#[test]
fn duplicate_class_like_type_in_intersection_is_redundant() {
    let stderr = fatal_for(
        r#"<?php
function test(): Foo&A&FOO {}
"#,
        "Zend/tests/type_declarations/intersection_types/redundant_types/duplicate_class_type.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: Duplicate type FOO is redundant in Zend/tests/type_declarations/intersection_types/redundant_types/duplicate_class_type.php on line 2"
    );
}

#[test]
fn nullable_null_is_rejected() {
    let stderr = fatal_for(
        r#"<?php
function test(): ?null {}
"#,
        "tests/type_declarations/nullable_null.php",
    );
    assert_eq!(
        stderr,
        "Fatal error: null cannot be marked as nullable in tests/type_declarations/nullable_null.php on line 2"
    );
}
