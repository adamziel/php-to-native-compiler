use php_compiler::{run_source, run_source_with_source_file};

#[test]
fn uninitialized_typed_property_read_is_catchable_error() {
    let execution = run_source_with_source_file(
        r#"<?php
class Box { public int $id; }
$box = new Box();
try {
    var_dump($box->id);
} catch (Error $error) {
    echo $error->getMessage(), "\n";
}
echo isset($box->id) ? "set" : "unset";
"#,
        "Zend/tests/type_declarations/typed_properties_magic_get_message.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Typed property Box::$id must not be accessed before initialization\nunset"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn assignment_to_unset_declared_property_invokes_magic_set_when_available() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id;
    public function __set($name, $value) {
        echo "set:", $name, "=", $value, "\n";
    }
}

$box = new Box();
$box->id = 1;
unset($box->id);
$box->id = 2;
try {
    var_dump($box->id);
} catch (Error $error) {
    echo $error->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "set:id=2\nTyped property Box::$id must not be accessed before initialization"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn assignment_to_unset_declared_property_without_magic_set_reinitializes_property() {
    let execution = run_source(
        r#"<?php
class Box { public int $id; }
$box = new Box();
$box->id = 1;
unset($box->id);
$box->id = 2;
var_dump($box->id);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(2)\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn typed_properties_magic_set_phpt_behavior() {
    let execution = run_source(
        r#"<?php
class Test {
    public int $foo;
    public function __get($name) {
        echo "__get ", $name, "\n";
        return null;
    }
    public function __set($name, $value) {
        echo "__set ", $name, " = ", $value, "\n";
    }
    public function __isset($name) {
        echo "__isset ", $name, "\n";
        return true;
    }
    public function __unset($name) {
        echo "__unset ", $name, "\n";
    }
}

$test = new Test;
try {
    var_dump($test->foo);
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
var_dump(isset($test->foo));
$test->foo = 42;
var_dump($test->foo);

unset($test->foo);
$test->foo = 42;

$test = new Test;
unset($test->foo);
$test->foo = 42;

class Test2 extends Test {
}

$test = new Test;
$test->foo = 42;
var_dump($test->foo);
unset($test->foo);
$test->foo = 42;

$test = clone $test;
$test->foo = 42;
$test = clone new Test;
$test->foo = 42;
var_dump($test->foo);
unset($test->foo);
$test->foo = 42;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Typed property Test::$foo must not be accessed before initialization\nbool(false)\nint(42)\n__set foo = 42\n__set foo = 42\nint(42)\n__set foo = 42\n__set foo = 42\nint(42)\n__set foo = 42\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
