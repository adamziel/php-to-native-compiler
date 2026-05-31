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
fn unset_typed_property_magic_get_read_validates_return_without_touching_fresh_or_private() {
    let execution = run_source(
        r#"<?php
class GoodBox {
    public int $bar;
    public function __get($name) {
        echo "good:", $name, "\n";
        return 0;
    }
}

$good = new GoodBox();
unset($good->bar);
var_dump($good->bar);

class WeakNumericBox {
    public int $bar;
    public function __get($name) {
        echo "weak:", $name, "\n";
        return "42";
    }
}

$weak = new WeakNumericBox();
unset($weak->bar);
var_dump($weak->bar);

class BadBox {
    public int $bar;
    public function __get($name) {
        echo "bad:", $name, "\n";
        return "bad";
    }
}

$bad = new BadBox();
unset($bad->bar);
try {
    var_dump($bad->bar);
} catch (Throwable $error) {
    echo "type:", get_class($error), ":", $error->getMessage(), "\n";
}

class SideEffectBadBox {
    public int $bar;
    public function __get($name) {
        echo "side-bad:", $name, "\n";
        $this->bar = 77;
        return "bad";
    }
}

$sideBad = new SideEffectBadBox();
unset($sideBad->bar);
try {
    var_dump($sideBad->bar);
} catch (Throwable $error) {
    echo "side-type:", get_class($error), ":", $error->getMessage(), "\n";
}
var_dump($sideBad->bar);

class FreshBox {
    public int $bar;
    public function __get($name) {
        echo "fresh-get\n";
        return 0;
    }
}

try {
    var_dump((new FreshBox())->bar);
} catch (Throwable $error) {
    echo "fresh:", get_class($error), ":", $error->getMessage(), "\n";
}

class PrivateBox {
    private int $bar;
    public function __construct() {
        unset($this->bar);
    }
    public function __get($name) {
        echo "private:", $name, "\n";
        return "bad";
    }
}

var_dump((new PrivateBox())->bar);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "good:bar\nint(0)\nweak:bar\nint(42)\nbad:bar\ntype:TypeError:Cannot assign string to property BadBox::$bar of type int\nside-bad:bar\nside-type:TypeError:Cannot assign string to property SideEffectBadBox::$bar of type int\nint(77)\nfresh:Error:Typed property FreshBox::$bar must not be accessed before initialization\nprivate:bar\nstring(3) \"bad\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_typed_property_magic_get_strictness_uses_magic_method_provenance() {
    let strict_caller_weak_decl = run_source(
        r#"<?php
declare(strict_types=1);
eval('declare(strict_types=0); class WeakMagicBox { public int $bar; public function __get($name) { return "42"; } }');
$box = new WeakMagicBox();
unset($box->bar);
var_dump($box->bar);
"#,
    )
    .unwrap();
    assert_eq!(strict_caller_weak_decl.stdout, "int(42)\n");
    assert_eq!(strict_caller_weak_decl.stderr, "");
    assert_eq!(strict_caller_weak_decl.exit_code, 0);

    let weak_caller_strict_decl = run_source(
        r#"<?php
declare(strict_types=0);
eval('declare(strict_types=1); class StrictMagicBox { public int $bar; public function __get($name) { return "42"; } }');
$box = new StrictMagicBox();
unset($box->bar);
try {
    var_dump($box->bar);
} catch (Throwable $error) {
    echo get_class($error), ":", $error->getMessage(), "\n";
}
"#,
    )
    .unwrap();
    assert_eq!(
        weak_caller_strict_decl.stdout,
        "TypeError:Cannot assign string to property StrictMagicBox::$bar of type int\n"
    );
    assert_eq!(weak_caller_strict_decl.stderr, "");
    assert_eq!(weak_caller_strict_decl.exit_code, 0);

    let strict_eval_then_later_weak = run_source(
        r#"<?php
eval('declare(strict_types=1); class EvalStrictMagicBox { public int $bar; public function __get($name) { return "42"; } }');
class LaterWeakMagicBox { public int $bar; public function __get($name) { return "42"; } }

$strict = new EvalStrictMagicBox();
unset($strict->bar);
try {
    var_dump($strict->bar);
} catch (Throwable $error) {
    echo get_class($error), ":", $error->getMessage(), "\n";
}

$weak = new LaterWeakMagicBox();
unset($weak->bar);
var_dump($weak->bar);
"#,
    )
    .unwrap();
    assert_eq!(
        strict_eval_then_later_weak.stdout,
        "TypeError:Cannot assign string to property EvalStrictMagicBox::$bar of type int\nint(42)\n"
    );
    assert_eq!(strict_eval_then_later_weak.stderr, "");
    assert_eq!(strict_eval_then_later_weak.exit_code, 0);
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
