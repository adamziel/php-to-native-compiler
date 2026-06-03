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
