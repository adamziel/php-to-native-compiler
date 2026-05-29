use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

fn run_for(source: &str, file: &str) -> (String, String) {
    let execution = run_source_with_source_file(source, file).expect("source should run");
    assert_eq!(execution.exit_code, 0);
    (execution.stdout, execution.stderr)
}

#[test]
fn qualified_scalar_parameter_type_is_reserved() {
    let stderr = fatal_for(
        r#"<?php
function foo(bar\int $value): int {
    return $value;
}
foo(10);
"#,
        "tests/type_declarations/scalar_relative_typehint.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"bar\\int\" as a type name as it is reserved in tests/type_declarations/scalar_relative_typehint.php on line 2"
    );
}

#[test]
fn qualified_scalar_return_type_is_reserved() {
    let stderr = fatal_for(
        r#"<?php
function foo(): Result\string {
    return "ok";
}
"#,
        "tests/type_declarations/scalar_relative_return_type.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"Result\\string\" as a type name as it is reserved in tests/type_declarations/scalar_relative_return_type.php on line 2"
    );
}

#[test]
fn qualified_scalar_property_type_is_reserved() {
    let stderr = fatal_for(
        r#"<?php
class Box {
    public Data\bool $value;
}
"#,
        "tests/type_declarations/scalar_relative_property_type.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"Data\\bool\" as a type name as it is reserved in tests/type_declarations/scalar_relative_property_type.php on line 3"
    );
}

#[test]
fn self_return_type_requires_class_scope_for_named_function() {
    let stderr = fatal_for(
        r#"<?php
function foo(): self {}
"#,
        "tests/type_declarations/relative_self_global_function.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"self\" when no class scope is active in tests/type_declarations/relative_self_global_function.php on line 2"
    );
}

#[test]
fn static_return_type_requires_class_scope_for_named_function() {
    let stderr = fatal_for(
        r#"<?php
function foo(): static {}
"#,
        "tests/type_declarations/relative_static_global_function.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"static\" when no class scope is active in tests/type_declarations/relative_static_global_function.php on line 2"
    );
}

#[test]
fn parent_return_type_requires_class_scope_for_named_function() {
    let stderr = fatal_for(
        r#"<?php
function foo(): parent {}
"#,
        "tests/type_declarations/relative_parent_global_function.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"parent\" when no class scope is active in tests/type_declarations/relative_parent_global_function.php on line 2"
    );
}

#[test]
fn parent_return_type_requires_parent_class_scope_for_interface_method() {
    let stderr = fatal_for(
        r#"<?php
interface T {
    public function foo(): parent;
}
"#,
        "tests/type_declarations/relative_parent_interface.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"parent\" when current class scope has no parent in tests/type_declarations/relative_parent_interface.php on line 3"
    );
}

#[test]
fn parent_parameter_type_requires_parent_class_scope_for_class_method() {
    let stderr = fatal_for(
        r#"<?php
class A {
    public function method(parent $value) {}
}
"#,
        "tests/type_declarations/relative_parent_class_method.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"parent\" when current class scope has no parent in tests/type_declarations/relative_parent_class_method.php on line 3"
    );
}

#[test]
fn method_parameter_cannot_use_static_type() {
    let stderr = fatal_for(
        r#"<?php
class Test {
    public function test(static $param) {}
}
"#,
        "tests/type_declarations/static_type_param.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use the static modifier on a parameter in tests/type_declarations/static_type_param.php on line 3"
    );
}

#[test]
fn trait_method_parameter_cannot_use_nullable_static_type() {
    let stderr = fatal_for(
        r#"<?php
trait T {
    public function test(?static $param) {}
}
"#,
        "tests/type_declarations/static_type_trait_param.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use the static modifier on a parameter in tests/type_declarations/static_type_trait_param.php on line 3"
    );
}

#[test]
fn union_parameter_cannot_include_static_type() {
    let stderr = fatal_for(
        r#"<?php
class Test {
    public function test(static|array $param) {}
}
"#,
        "tests/type_declarations/static_type_union_param.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use the static modifier on a parameter in tests/type_declarations/static_type_union_param.php on line 3"
    );
}

#[test]
fn promoted_constructor_parameter_cannot_use_static_modifier() {
    let stderr = fatal_for(
        r#"<?php
class Test {
    public function __construct(public static $value) {}
}
"#,
        "tests/ctor_promotion/ctor_promotion_additional_modifiers.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use the static modifier on a parameter in tests/ctor_promotion/ctor_promotion_additional_modifiers.php on line 3"
    );
}

#[test]
fn unresolved_parent_intersection_type_is_rejected_in_trait() {
    let stderr = fatal_for(
        r#"<?php
trait T {
    public function foo(): PARENT&Iterator {}
}
?>
DONE
"#,
        "tests/type_declarations/intersection_types/relative_parent_trait.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type PARENT cannot be part of an intersection type in tests/type_declarations/intersection_types/relative_parent_trait.php on line 3"
    );
}

#[test]
fn unresolved_self_intersection_type_is_rejected_in_trait() {
    let stderr = fatal_for(
        r#"<?php
trait T {
    public function foo(): SELF&Iterator {}
}
?>
DONE
"#,
        "tests/type_declarations/intersection_types/relative_self_trait.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type SELF cannot be part of an intersection type in tests/type_declarations/intersection_types/relative_self_trait.php on line 3"
    );
}

#[test]
fn dnf_trait_parent_type_is_preserved_as_resolvable_at_use_site() {
    let (stdout, stderr) = run_for(
        r#"<?php
trait TraitExample {
    public function bar(): (X&Y)|parent { return parent::class; }
}

class A {
    use TraitExample;
}
?>
DONE
"#,
        "tests/type_declarations/relative_types/traits/trait_parent_type_in_class_no_parent4.php",
    );

    assert_eq!(stdout, "DONE\n");
    assert_eq!(stderr, "");
}
