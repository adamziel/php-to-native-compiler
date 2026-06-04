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
fn reflection_class_relationship_methods_report_php_metadata_errors() {
    let execution = run_source(
        r#"<?php
interface I1 {}
class A implements I1 {}
class B extends A {}
interface I2 extends I1 {}
class C implements I2 {}

$refs = array(
    "A" => new ReflectionClass("A"),
    "B" => new ReflectionClass("B"),
    "C" => new ReflectionClass("C"),
    "I1" => new ReflectionClass("I1"),
    "I2" => new ReflectionClass("I2"),
);

function yn($value) {
    return $value ? "1" : "0";
}

echo "impl|", yn($refs["A"]->implementsInterface($refs["I1"]));
echo yn($refs["B"]->implementsInterface("I1"));
echo yn($refs["C"]->implementsInterface("I1"));
echo yn($refs["C"]->implementsInterface("I2"));
echo yn($refs["I2"]->implementsInterface($refs["I1"]));
echo yn($refs["I1"]->implementsInterface("I1")), "\n";

foreach (array("A", "Missing", 2) as $target) {
    try {
        $refs["A"]->implementsInterface($target);
    } catch (ReflectionException $e) {
        echo "impl-error|", $e->getMessage(), "\n";
    }
}

try {
    $refs["A"]->implementsInterface();
} catch (ArgumentCountError $e) {
    echo "impl-arity|", $e->getMessage(), "\n";
}

echo "sub|", yn($refs["B"]->isSubclassOf($refs["A"]));
echo yn($refs["A"]->isSubclassOf($refs["B"])), "\n";

foreach (array("Missing", 2) as $target) {
    try {
        $refs["A"]->isSubclassOf($target);
    } catch (ReflectionException $e) {
        echo "sub-error|", $e->getMessage(), "\n";
    }
}

try {
    $refs["A"]->isSubclassOf("A", "B");
} catch (ArgumentCountError $e) {
    echo "sub-arity|", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "impl|111111\n",
            "impl-error|A is not an interface\n",
            "impl-error|Interface \"Missing\" does not exist\n",
            "impl-error|Interface \"2\" does not exist\n",
            "impl-arity|ReflectionClass::implementsInterface() expects exactly 1 argument, 0 given\n",
            "sub|10\n",
            "sub-error|Class \"Missing\" does not exist\n",
            "sub-error|Class \"2\" does not exist\n",
            "sub-arity|ReflectionClass::isSubclassOf() expects exactly 1 argument, 2 given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_parameter_deprecated_class_array_and_contextual_default_metadata() {
    let execution = run_source(
        r#"<?php
namespace ReflectionParamDefaults {
    const VALUE = "v";
}

namespace {
    function yn($value) {
        return $value ? "1" : "0";
    }

    function class_name_for($parameter) {
        $class = @$parameter->getClass();
        return $class ? $class->getName() : "NULL";
    }

    class BaseParamDefault {
        const FALLBACK = 20;
    }

    class ChildParamDefault extends BaseParamDefault {
        const DEFAULT_VALUE = 12;

        public function method(
            array $array,
            ?array $nullableArray,
            stdClass $object,
            self $selfType,
            parent $parentType,
            $selfDefault = self::DEFAULT_VALUE,
            $parentDefault = array(parent::FALLBACK)
        ) {}
    }

    function global_constant_default($value = ReflectionParamDefaults\VALUE) {}

    $method = new ReflectionMethod(ChildParamDefault::class, "method");
    $params = $method->getParameters();

    echo "array|", yn(@$params[0]->isArray()), yn(@$params[1]->isArray()), yn(@$params[2]->isArray()), "\n";
    echo "class|", class_name_for($params[2]), "|", class_name_for($params[3]), "|", class_name_for($params[4]), "\n";
    $arrayDefault = $params[6]->getDefaultValue();
    echo "default|", $params[5]->getDefaultValue(), "|", $arrayDefault[0], "\n";
    echo "constant|", (new ReflectionFunction("global_constant_default"))->getParameters()[0]->getDefaultValueConstantName(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array|110\n",
            "class|stdClass|ChildParamDefault|BaseParamDefault\n",
            "default|12|20\n",
            "constant|ReflectionParamDefaults\\VALUE\n",
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
fn internal_datetime_signatures_feed_declaration_compatibility() {
    for (source, expected) in [
        (
            r#"<?php
class MyDateTimeZone extends DateTimeZone
{
    public static function listIdentifiers(): array {}
}
"#,
            "Declaration of MyDateTimeZone::listIdentifiers(): array must be compatible with DateTimeZone::listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null): array",
        ),
        (
            r#"<?php
class MyDateTimeZone extends DateTimeZone
{
    public function getTransitions(): array|false {}
}
"#,
            "Declaration of MyDateTimeZone::getTransitions(): array|false must be compatible with DateTimeZone::getTransitions(int $timestampBegin = PHP_INT_MIN, int $timestampEnd = 2147483647): array|false",
        ),
        (
            r#"<?php
class MyDateTime extends DateTime
{
    public function setTime(int $hour, int $minute, int $second = 0, bool $microsecond = false): DateTime {}
}
"#,
            "Declaration of MyDateTime::setTime(int $hour, int $minute, int $second = 0, bool $microsecond = false): DateTime must be compatible with DateTime::setTime(int $hour, int $minute, int $second = 0, int $microsecond = 0): DateTime",
        ),
        (
            r#"<?php
class MyDateTime extends DateTime
{
    public static function createFromFormat(): DateTime|false {}
}
"#,
            "Declaration of MyDateTime::createFromFormat(): DateTime|false must be compatible with DateTime::createFromFormat(string $format, string $datetime, ?DateTimeZone $timezone = null): DateTime|false",
        ),
        (
            r#"<?php
interface MyDateTimeInterface extends DateTimeInterface
{
    public function diff(): DateInterval;
}
"#,
            "Declaration of MyDateTimeInterface::diff(): DateInterval must be compatible with DateTimeInterface::diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval",
        ),
    ] {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.exit_code, 255, "{source}");
        let diagnostics = format!("{}{}", execution.stdout, execution.stderr);
        assert!(
            diagnostics.contains(expected),
            "expected fatal message not found\nexpected: {expected}\nstdout: {}\nstderr: {}",
            execution.stdout,
            execution.stderr
        );
    }
}

#[test]
fn internal_datetime_tentative_return_rows_match_startup_diagnostics() {
    let incompatible = run_source(
        r#"<?php
class MyDateTimeZone extends DateTimeZone
{
    public static function listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null): string
    {
        return "";
    }
}

var_dump(MyDateTimeZone::listIdentifiers());
"#,
    )
    .unwrap();
    assert_eq!(incompatible.exit_code, 0);
    assert_eq!(incompatible.stderr, "");
    assert!(
        incompatible.stdout.contains("Deprecated: Return type of MyDateTimeZone::listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null): string should either be compatible with DateTimeZone::listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null): array, or the #[\\ReturnTypeWillChange] attribute should be used to temporarily suppress the notice"),
        "stdout: {}",
        incompatible.stdout
    );
    assert!(incompatible.stdout.ends_with("string(0) \"\"\n"));

    let missing = run_source(
        r#"<?php
class MyDateTimeZone extends DateTimeZone
{
    public static function listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null)
    {
    }
}
"#,
    )
    .unwrap();
    assert_eq!(missing.exit_code, 0);
    assert_eq!(missing.stderr, "");
    assert!(
        missing.stdout.contains("Deprecated: Return type of MyDateTimeZone::listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null) should either be compatible with DateTimeZone::listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null): array, or the #[\\ReturnTypeWillChange] attribute should be used to temporarily suppress the notice"),
        "stdout: {}",
        missing.stdout
    );

    for (source, expected) in [
        (
            r#"<?php
class Test extends DateTime {
    public static function createFromFormat($format, $datetime, ?Wrong $timezone = null): DateTime|false {}
}
"#,
            "Could not check compatibility between Test::createFromFormat($format, $datetime, ?Wrong $timezone = null): DateTime|false and DateTime::createFromFormat(string $format, string $datetime, ?DateTimeZone $timezone = null): DateTime|false, because class Wrong is not available",
        ),
        (
            r#"<?php
class Test extends DateTime {
    public static function createFromFormat($format, $datetime, $timezone = null): Wrong {}
}
"#,
            "Could not check compatibility between Test::createFromFormat($format, $datetime, $timezone = null): Wrong and DateTime::createFromFormat(string $format, string $datetime, ?DateTimeZone $timezone = null): DateTime|false, because class Wrong is not available",
        ),
    ] {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.exit_code, 255, "{source}");
        let diagnostics = format!("{}{}", execution.stdout, execution.stderr);
        assert!(
            diagnostics.contains(expected),
            "expected fatal message not found\nexpected: {expected}\nstdout: {}\nstderr: {}",
            execution.stdout,
            execution.stderr
        );
    }
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

#[test]
fn reflection_method_properties_prototypes_and_function_extensions() {
    let execution = run_source(
        r#"<?php
function yn($value) {
    return $value ? "1" : "0";
}

class BaseProto {
    public function label() {}
    private function hidden() {}
}

interface ProtoContract {
    public function run();
}

class ChildProto extends BaseProto implements ProtoContract {
    public function label() {}
    public function run() {}
    private function hidden() {}
}

$class = new ReflectionClass(ChildProto::class);

function show_method($label, $method) {
    echo $label, "|", $method->class, "::", $method->name, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "\n";
}

$label = $class->getMethod("label");
show_method("label", $label);
echo "interpolated|$label->class::$label->name()\n";
$labelPrototype = $label->getPrototype();
show_method("label-prototype", $labelPrototype);

$run = $class->getMethod("run");
show_method("run", $run);
echo "run-has-prototype|", yn($run->hasPrototype()), "\n";
$runPrototype = $run->getPrototype();
show_method("run-prototype", $runPrototype);

$hidden = $class->getMethod("hidden");
echo "hidden-has-prototype|", yn($hidden->hasPrototype()), "\n";
try {
    $hidden->getPrototype();
} catch (ReflectionException $exception) {
    echo "hidden-prototype|", $exception->getMessage(), "\n";
}
try {
    $class->getMethod("missing");
} catch (ReflectionException $exception) {
    echo "missing|", $exception->getMessage(), "\n";
}

$sort = new ReflectionFunction("sort");
echo "extension|", get_class($sort->getExtension()), "|", $sort->getExtension()->getName(), "\n";
function local_proto_fn() {}
var_dump((new ReflectionFunction("local_proto_fn"))->getExtension());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "label|ChildProto::label|label|ChildProto\n",
            "interpolated|ChildProto::label()\n",
            "label-prototype|BaseProto::label|label|BaseProto\n",
            "run|ChildProto::run|run|ChildProto\n",
            "run-has-prototype|1\n",
            "run-prototype|ProtoContract::run|run|ProtoContract\n",
            "hidden-has-prototype|0\n",
            "hidden-prototype|Method ChildProto::hidden does not have a prototype\n",
            "missing|Method ChildProto::missing() does not exist\n",
            "extension|ReflectionExtension|standard\n",
            "NULL\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_invocation_argument_diagnostics_are_php_shaped() {
    let execution = run_source(
        r#"<?php
class TestClass {
    public $prop = "Hello";

    public function foo() {
        echo "foo|$this->prop\n";
        return "Return Val";
    }

    public function methodWithArgs($a, $b) {
        echo "args|$a|$b\n";
    }

    public static function staticMethod() {
        echo "static\n";
        try {
            var_dump($this);
        } catch (Throwable $e) {
            echo "this|", $e->getMessage(), "\n";
        }
    }

    private static function privateMethod() {
        echo "private\n";
    }
}

abstract class AbstractClass {
    abstract function foo();
}

function show($label, $callable) {
    echo $label, "|\n";
    try {
        var_dump($callable());
    } catch (Throwable $e) {
        echo get_class($e), ": ", $e->getMessage(), "\n";
    }
}

show("construct-none", function () {
    return new ReflectionMethod();
});
show("construct-many", function () {
    return new ReflectionMethod("a", "b", "c");
});

$fromName = new ReflectionMethod("TestClass::foo");
echo "from-name|", $fromName->getDeclaringClass()->getName(), "::", $fromName->getName(), "\n";

$test = new TestClass();
$foo = new ReflectionMethod("TestClass", "foo");
$methodWithArgs = new ReflectionMethod("TestClass", "methodWithArgs");
$staticMethod = new ReflectionMethod("TestClass", "staticMethod");
$privateMethod = ReflectionMethod::createFromMethodName("TestClass::privateMethod");
$abstractMethod = ReflectionMethod::createFromMethodName("AbstractClass::foo");

show("invoke-extra", function () use ($foo, $test) {
    return $foo->invoke($test, true);
});
show("invokeArgs-extra", function () use ($methodWithArgs, $test) {
    return $methodWithArgs->invokeArgs($test, array(1, "two", 3));
});
show("invoke-missing-target", function () use ($staticMethod) {
    return $staticMethod->invoke();
});
show("invoke-non-object", function () use ($foo) {
    return $foo->invoke(true);
});
show("invoke-non-instance", function () use ($foo) {
    return $foo->invoke(new stdClass());
});
show("private-static", function () use ($privateMethod, $test) {
    return $privateMethod->invoke($test);
});
show("static-object", function () use ($staticMethod) {
    return $staticMethod->invoke(new stdClass());
});
show("invokeArgs-non-array", function () use ($foo, $test) {
    return $foo->invokeArgs($test, true);
});
show("abstract-invoke", function () use ($abstractMethod, $test) {
    return $abstractMethod->invoke($test);
});
show("abstract-invokeArgs", function () use ($abstractMethod) {
    return $abstractMethod->invokeArgs(true);
});
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "construct-none|\n",
            "ArgumentCountError: ReflectionMethod::__construct() expects at least 1 argument, 0 given\n",
            "construct-many|\n",
            "ArgumentCountError: ReflectionMethod::__construct() expects at most 2 arguments, 3 given\n",
            "from-name|TestClass::foo\n",
            "invoke-extra|\n",
            "foo|Hello\n",
            "string(10) \"Return Val\"\n",
            "invokeArgs-extra|\n",
            "args|1|two\n",
            "NULL\n",
            "invoke-missing-target|\n",
            "TypeError: ReflectionMethod::invoke() expects at least 1 argument, 0 given\n",
            "invoke-non-object|\n",
            "TypeError: ReflectionMethod::invoke(): Argument #1 ($object) must be of type ?object, true given\n",
            "invoke-non-instance|\n",
            "ReflectionException: Given object is not an instance of the class this method was declared in\n",
            "private-static|\n",
            "private\n",
            "NULL\n",
            "static-object|\n",
            "static\n",
            "this|Using $this when not in object context\n",
            "NULL\n",
            "invokeArgs-non-array|\n",
            "TypeError: ReflectionMethod::invokeArgs(): Argument #2 ($args) must be of type array, true given\n",
            "abstract-invoke|\n",
            "ReflectionException: Trying to invoke abstract method AbstractClass::foo()\n",
            "abstract-invokeArgs|\n",
            "ReflectionException: Trying to invoke abstract method AbstractClass::foo()\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_property_dynamic_instance_metadata_is_bounded() {
    let execution = run_source(
        r#"<?php
#[AllowDynamicProperties]
class Bag {
    private $hidden;
    public $declared;
}

$bag = new Bag();
$bag->dynamic = "value";

$dynamic = new ReflectionProperty($bag, "dynamic");
echo $dynamic->getName(), "|", $dynamic->getMangledName(), "|";
var_dump($dynamic->isDefault(), $dynamic->isDynamic(), $dynamic->hasDefaultValue());
var_dump($dynamic->isReadable(null, $bag));
unset($bag->dynamic);
var_dump($dynamic->isReadable(null, $bag));
var_dump($dynamic->isWritable(null, $bag));
var_dump($dynamic->isWritable(null, null));

$declared = new ReflectionProperty($bag, "declared");
var_dump($declared->isDefault(), $declared->isDynamic());

$hidden = new ReflectionProperty($bag, "hidden");
var_dump($hidden->isWritable(null, $bag));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "dynamic|dynamic|bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
