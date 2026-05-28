use php_compiler::run_source_with_source_file;

fn run(source: &str) -> php_compiler::interpreter::Execution {
    run_source_with_source_file(source, "typed-property-disallowed.php").unwrap()
}

#[test]
fn callable_property_type_reports_php_fatal() {
    let execution = run(r#"<?php
class A {
    public callable $a;
}
$obj = new A;
var_dump($obj);
"#);

    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stderr,
        "Fatal error: Property A::$a cannot have type callable in typed-property-disallowed.php on line 3"
    );
}

#[test]
fn nullable_callable_property_type_reports_declared_type() {
    let execution = run(r#"<?php
class A {
    public ?callable $a;
}
$obj = new A;
var_dump($obj);
"#);

    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stderr,
        "Fatal error: Property A::$a cannot have type ?callable in typed-property-disallowed.php on line 3"
    );
}

#[test]
fn disallowed_property_type_detection_is_not_class_specific() {
    let execution = run(r#"<?php
trait T {
    protected callable $handler;
}
class UsesT {
    use T;
}
"#);

    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stderr,
        "Fatal error: Property T::$handler cannot have type callable in typed-property-disallowed.php on line 3"
    );
}
