use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn inherited_property_type_mismatch_reports_child_class_line() {
    let stderr = fatal_for(
        r#"<?php
class Foo { public int $qux; }

class Bar extends Foo {
    public string $qux;
}
"#,
        "Zend/tests/type_declarations/typed_properties_006.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type of Bar::$qux must be int (as in class Foo) in Zend/tests/type_declarations/typed_properties_006.php on line 4"
    );
}

#[test]
fn inherited_property_class_type_mismatch_uses_parent_type_name() {
    let stderr = fatal_for(
        r#"<?php
class Whatever {}
class Thing extends Whatever {}
class Foo { public Whatever $qux; }

class Bar extends Foo {
    public Thing $qux;
}
"#,
        "Zend/tests/type_declarations/typed_properties_007.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type of Bar::$qux must be Whatever (as in class Foo) in Zend/tests/type_declarations/typed_properties_007.php on line 6"
    );
}

#[test]
fn inherited_typed_property_cannot_become_untyped() {
    let stderr = fatal_for(
        r#"<?php
class Foo { public int $qux; }

class Bar extends Foo {
    public $qux;
}
"#,
        "Zend/tests/type_declarations/typed_properties_008.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type of Bar::$qux must be int (as in class Foo) in Zend/tests/type_declarations/typed_properties_008.php on line 4"
    );
}

#[test]
fn inherited_untyped_property_cannot_gain_type() {
    let stderr = fatal_for(
        r#"<?php
class Foo { public $bar = 42; }

class Baz extends Foo {
    public int $bar = 33;
}
"#,
        "Zend/tests/type_declarations/typed_properties_035.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type of Baz::$bar must be omitted to match the parent definition in class Foo in Zend/tests/type_declarations/typed_properties_035.php on line 4"
    );
}

#[test]
fn string_default_must_match_property_type() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public int $bar = "string";
}
"#,
        "Zend/tests/type_declarations/typed_properties_013.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use string as default value for property Foo::$bar of type int in Zend/tests/type_declarations/typed_properties_013.php on line 3"
    );
}

#[test]
fn int_default_must_match_array_property_type() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public array $bar = 32;
}
"#,
        "Zend/tests/type_declarations/typed_properties_014.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use int as default value for property Foo::$bar of type array in Zend/tests/type_declarations/typed_properties_014.php on line 3"
    );
}

#[test]
fn null_default_requires_nullable_class_type() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public stdClass $bar = null;
}
"#,
        "Zend/tests/type_declarations/typed_properties_015.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Default value for property of type stdClass may not be null. Use the nullable type ?stdClass to allow null default value in Zend/tests/type_declarations/typed_properties_015.php on line 3"
    );
}

#[test]
fn null_default_requires_nullable_scalar_type() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public int $foo = null;
}
"#,
        "Zend/tests/type_declarations/typed_properties_049.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Default value for property of type int may not be null. Use the nullable type ?int to allow null default value in Zend/tests/type_declarations/typed_properties_049.php on line 3"
    );
}

#[test]
fn default_must_match_one_union_property_type() {
    let stderr = fatal_for(
        r#"<?php
class Test {
    public int|float $prop = "1";
}
"#,
        "Zend/tests/type_declarations/union_types/illegal_default_value_property.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use string as default value for property Test::$prop of type int|float in Zend/tests/type_declarations/union_types/illegal_default_value_property.php on line 3"
    );
}

#[test]
fn bool_default_must_match_literal_property_type() {
    let stderr = fatal_for(
        r#"<?php
class Test {
    public true $flag = false;
}
"#,
        "typed-property-bool-literal.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use bool as default value for property Test::$flag of type true in typed-property-bool-literal.php on line 3"
    );
}

#[test]
fn null_default_is_not_legal_for_intersection_property_type() {
    let stderr = fatal_for(
        r#"<?php
interface X {}
interface Y {}
class Test { public X&Y $y = null; }
"#,
        "Zend/tests/type_declarations/intersection_types/bug81268.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use null as default value for property Test::$y of type X&Y in Zend/tests/type_declarations/intersection_types/bug81268.php on line 4"
    );
}

#[test]
fn conflicting_trait_property_definitions_are_fatal() {
    let stderr = fatal_for(
        r#"<?php
trait T1 { public int $prop; }
trait T2 { public string $prop; }
class C { use T1, T2; }
"#,
        "Zend/tests/type_declarations/typed_properties_085.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: T1 and T2 define the same property ($prop) in the composition of C. However, the definition differs and is considered incompatible. Class was composed in Zend/tests/type_declarations/typed_properties_085.php on line 4"
    );
}

#[test]
fn property_cannot_have_never_type() {
    let stderr = fatal_for(
        r#"<?php
class Foo {
    public never $int;
}
"#,
        "Zend/tests/type_declarations/typed_properties_109.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Property Foo::$int cannot have type never in Zend/tests/type_declarations/typed_properties_109.php on line 3"
    );
}
