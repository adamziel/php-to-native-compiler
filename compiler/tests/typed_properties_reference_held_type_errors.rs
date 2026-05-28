use php_compiler::run_source_with_source_file;

#[test]
fn reference_held_by_typed_property_reports_type_error() {
    let execution = run_source_with_source_file(
        r#"<?php
class A {
    public $foo = 1;
    public int $bar = 2;
}
class B {
    public A $a;
}
$f = function (&$n) {
    $n = "ops";
};
$o = new B;
$o->a = new A;
try {
    $f($o->a->bar);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
var_dump($o->a->bar);
"#,
        "Zend/tests/type_declarations/typed_properties_reference_held_type_error.php",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert_eq!(
        execution.stdout,
        "Cannot assign string to reference held by property A::$bar of type int\nint(2)\n"
    );
}

#[test]
fn uncaught_reference_held_by_typed_property_keeps_closure_stack_frame() {
    let execution = run_source_with_source_file(
        r#"<?php
class A {
    public $foo = 1;
    public int $bar = 2;
}
class B {
    public A $a;
}
$f = function (&$n) {
    var_dump($n);
    $n = "ops";
};
$o = new B;
$o->a = new A;
$f($o->a->foo);
$f($o->a->bar);
"#,
        "Zend/tests/type_declarations/typed_properties_055.php",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
    assert!(execution.stdout.contains("int(1)\nint(2)\n\nFatal error: Uncaught TypeError: Cannot assign string to reference held by property A::$bar of type int"));
    assert!(execution
        .stdout
        .contains("Stack trace:\n#0 Zend/tests/type_declarations/typed_properties_055.php("));
    assert!(execution
        .stdout
        .contains("): {closure:Zend/tests/type_declarations/typed_properties_055.php:"));
    assert!(execution.stdout.contains("}(2)\n#1 {main}\n  thrown in Zend/tests/type_declarations/typed_properties_055.php on line "));
}
