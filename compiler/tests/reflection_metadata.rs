use php_compiler::run_source;

#[test]
fn reflection_function_variadic_constructor_and_has_method_rows() {
    let execution = run_source(
        r#"<?php
function test1($args) {}
function test2(...$args) {}
function test3($arg, ...$args) {}

var_dump((new ReflectionFunction('test1'))->isVariadic());
var_dump((new ReflectionFunction('test2'))->isVariadic());
var_dump((new ReflectionFunction('test3'))->isVariadic());

class NewCtor {
    function __construct() {}
}

class ExtendsNewCtor extends NewCtor {
}

$classes = array('NewCtor', 'ExtendsNewCtor');
foreach ($classes as $class) {
    $rc = new ReflectionClass($class);
    $rm = $rc->getConstructor();
    if ($rm != null) {
        echo "Constructor of $class: " . $rm->getName() . "\n";
    } else {
        echo "No constructor for $class\n";
    }
}

class C {
    function f() {}
}

$rc = new ReflectionClass("C");
echo "Check invalid params:\n";
var_dump($rc->hasMethod(1));
var_dump($rc->hasMethod(1.5));
var_dump($rc->hasMethod(true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "Constructor of NewCtor: __construct\n",
            "Constructor of ExtendsNewCtor: __construct\n",
            "Check invalid params:\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn internal_datetime_parameter_defaults_reflect_php_metadata() {
    let execution = run_source(
        r#"<?php
function show_default_values($method) {
    foreach ($method->getParameters() as $parameter) {
        try {
            var_dump($parameter->getDefaultValue());
        } catch (ReflectionException $exception) {
            echo $exception->getMessage(), "\n";
        }
    }
}

function show_default_constant_names($method) {
    foreach ($method->getParameters() as $parameter) {
        try {
            var_dump($parameter->getDefaultValueConstantName());
        } catch (ReflectionException $exception) {
            echo $exception->getMessage(), "\n";
        }
    }
}

function show_default_constant_flags($method) {
    foreach ($method->getParameters() as $parameter) {
        try {
            var_dump($parameter->isDefaultValueConstant());
        } catch (ReflectionException $exception) {
            echo $exception->getMessage(), "\n";
        }
    }
}

$setTime = (new ReflectionClass("DateTime"))->getMethod("setTime");
$transitions = (new ReflectionClass("DateTimeZone"))->getMethod("getTransitions");
$identifiers = (new ReflectionClass("DateTimeZone"))->getMethod("listIdentifiers");

show_default_values($setTime);
echo "----------\n";
show_default_constant_names($transitions);
echo "----------\n";
show_default_constant_names($identifiers);
echo "----------\n";
show_default_constant_flags($setTime);
echo "----------\n";
show_default_constant_flags($identifiers);
echo "----------\n";
foreach ($setTime->getParameters() as $parameter) {
    echo $parameter, "\n";
}
echo "----------\n";
foreach ($identifiers->getParameters() as $parameter) {
    echo $parameter, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Internal error: Failed to retrieve the default value\n",
            "Internal error: Failed to retrieve the default value\n",
            "int(0)\n",
            "int(0)\n",
            "----------\n",
            "string(11) \"PHP_INT_MIN\"\n",
            "NULL\n",
            "----------\n",
            "string(17) \"DateTimeZone::ALL\"\n",
            "NULL\n",
            "----------\n",
            "Internal error: Failed to retrieve the default value\n",
            "Internal error: Failed to retrieve the default value\n",
            "bool(false)\n",
            "bool(false)\n",
            "----------\n",
            "bool(true)\n",
            "bool(false)\n",
            "----------\n",
            "Parameter #0 [ <required> int $hour ]\n",
            "Parameter #1 [ <required> int $minute ]\n",
            "Parameter #2 [ <optional> int $second = 0 ]\n",
            "Parameter #3 [ <optional> int $microsecond = 0 ]\n",
            "----------\n",
            "Parameter #0 [ <optional> int $timezoneGroup = DateTimeZone::ALL ]\n",
            "Parameter #1 [ <optional> ?string $countryCode = null ]\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_modifier_names_and_class_cloneability_metadata() {
    let execution = run_source(
        r#"<?php
echo implode(",", Reflection::getModifierNames(ReflectionMethod::IS_FINAL | ReflectionMethod::IS_PROTECTED)), "\n";
echo implode(",", Reflection::getModifierNames(ReflectionProperty::IS_PUBLIC | ReflectionProperty::IS_STATIC | ReflectionProperty::IS_READONLY)), "\n";
echo implode(",", Reflection::getModifierNames(ReflectionProperty::IS_VIRTUAL)), "\n";
echo implode(",", Reflection::getModifierNames(ReflectionProperty::IS_PROTECTED_SET)), "\n";
echo implode(",", Reflection::getModifierNames(ReflectionProperty::IS_PRIVATE_SET)), "\n";
echo implode(",", Reflection::getModifierNames(ReflectionClass::IS_FINAL | ReflectionClass::IS_READONLY)), "\n";

class PlainCloneable {}
class PrivateClone {
    private function __clone() {}
}
class ProtectedClone {
    protected function __clone() {}
}
abstract class AbstractClone {}
trait CloneTrait {}

foreach (array('PlainCloneable', 'PrivateClone', 'ProtectedClone', 'AbstractClone', 'CloneTrait') as $class) {
    echo (new ReflectionClass($class))->isCloneable() ? "1\n" : "0\n";
}

try {
    clone new ReflectionClass('stdClass');
} catch (Error $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "final,protected\n",
            "public,static,readonly\n",
            "virtual\n",
            "protected(set)\n",
            "private(set)\n",
            "final,readonly\n",
            "1\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "Error: Trying to clone an uncloneable object of class ReflectionClass\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
