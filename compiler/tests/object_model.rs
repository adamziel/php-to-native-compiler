use php_compiler::error::{Diagnostic, Phase};
use php_compiler::{class_metadata_source, run_source};
use php_runtime::Visibility;

fn parse_error(source: &str) -> Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn runtime_error(source: &str) -> Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn class_declarations_register_metadata_without_object_execution() {
    let source = r#"<?php
class Box {
    public $value;
    private static $cache;
    protected function compute($input = "x") {
        return $input;
    }
    public static function make() {
        return "ok";
    }
}
echo "ready\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "ready\n");

    let classes = class_metadata_source(source).unwrap();
    assert_eq!(classes.classes().len(), 1);

    let class = classes.lookup_class("box").unwrap();
    assert_eq!(class.name(), "Box");
    assert_eq!(class.properties().len(), 2);
    assert_eq!(class.methods().len(), 2);

    let value = class.property("value").unwrap();
    assert_eq!(value.visibility(), Visibility::Public);
    assert!(!value.is_static());

    let cache = class.property("cache").unwrap();
    assert_eq!(cache.visibility(), Visibility::Private);
    assert!(cache.is_static());

    let compute = class.method("COMPUTE").unwrap();
    assert_eq!(compute.visibility(), Visibility::Protected);
    assert!(!compute.is_static());

    let make = class.method("make").unwrap();
    assert_eq!(make.visibility(), Visibility::Public);
    assert!(make.is_static());
}

#[test]
fn new_class_name_instantiates_minimal_object_values() {
    let source = r#"<?php
class Box {
    public $value;
    private static $cache;
}

$box = new box();
if ($box) {
    echo "object is truthy\n";
}
if (isset($box)) {
    echo "object is set\n";
}
print_r($box);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object is truthy\nobject is set\nBox Object\n(\n    [value] => \n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn public_instance_property_reads_and_writes_mutate_object_slots() {
    let source = r#"<?php
class Profile {
    public $name;
    public $visits;
    protected $secret;
    private $token;
    private static $cache;
}

$profile = new profile();
echo "initial:", $profile->name, "\n";
$profile->name = "Ada";
$profile->visits = 3;
echo $profile->name, "\n";
echo $profile->visits + 2, "\n";
print_r($profile);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "initial:\nAda\n5\nProfile Object\n(\n    [name] => Ada\n    [visits] => 3\n    [secret:protected] => \n    [token:Profile:private] => \n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn isset_public_instance_properties_checks_current_slot_values() {
    let source = r#"<?php
class Profile {
    public $name;
    public $visits;
    private $token;
}

$profile = new profile();
if (isset($profile->name)) {
    echo "name:set before\n";
} else {
    echo "name:unset before\n";
}

$profile->name = "Ada";
$profile->visits = 0;
if (isset($profile->name, $profile->visits)) {
    echo "name+visits:set\n";
}
if (isset($profile->missing)) {
    echo "missing:set\n";
} else {
    echo "missing:unset\n";
}

$value = 1;
if (isset($value->name)) {
    echo "scalar:set\n";
} else {
    echo "scalar:unset\n";
}
if (isset($missing->name)) {
    echo "missing-target:set\n";
} else {
    echo "missing-target:unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "name:unset before\nname+visits:set\nmissing:unset\nscalar:unset\nmissing-target:unset"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_class_instantiation_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
$box = new Missing();
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 8);
    assert_eq!(error.message, "undefined class Missing");
}

#[test]
fn constructors_remain_explicitly_unsupported_for_instantiation() {
    let error = runtime_error(
        r#"<?php
class Box {
    public function __construct() {}
}

$box = new Box();
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 8);
    assert_eq!(
        error.message,
        "unsupported object instantiation for Box: constructors are not implemented"
    );

    let argument_error = runtime_error(
        r#"<?php
class Box {}

$box = new Box("name");
"#,
    );

    assert_eq!(argument_error.line, 4);
    assert_eq!(argument_error.column, 8);
    assert_eq!(
        argument_error.message,
        "unsupported object instantiation for Box: constructor arguments are not implemented"
    );
}

#[test]
fn duplicate_class_metadata_has_stable_runtime_errors() {
    let error = run_source(
        r#"<?php
class Box {}
class box {}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "class box is already defined");
}

#[test]
fn duplicate_class_members_have_stable_runtime_errors() {
    let property_error = run_source(
        r#"<?php
class Packet {
    public $value;
    private $value;
}
"#,
    )
    .unwrap_err();

    assert_eq!(property_error.phase, Phase::Runtime);
    assert_eq!(property_error.line, 4);
    assert_eq!(property_error.column, 13);
    assert_eq!(
        property_error.message,
        "class Packet already defines property value"
    );

    let method_error = run_source(
        r#"<?php
class Packet {
    public function Send() {}
    private function send() {}
}
"#,
    )
    .unwrap_err();

    assert_eq!(method_error.phase, Phase::Runtime);
    assert_eq!(method_error.line, 4);
    assert_eq!(method_error.column, 13);
    assert_eq!(
        method_error.message,
        "class Packet already defines method send"
    );
}

#[test]
fn unsupported_object_execution_syntax_is_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
class Box {
    public function name() {}
}
$box = new Box();
$box->name();
"#,
            6,
            5,
            "unsupported method call: method dispatch is not implemented",
        ),
        (
            r#"<?php
class Box {
    public $name;
}
$box = new Box();
$property = "name";
$box->$property;
"#,
            7,
            7,
            "unsupported dynamic property access: dynamic property names are not implemented",
        ),
        (
            r#"<?php
$box = new class {};
"#,
            2,
            12,
            "unsupported anonymous class: anonymous classes are not implemented",
        ),
        (
            r#"<?php
if (true) {
    class Nested {}
}
"#,
            3,
            5,
            "unsupported nested class declaration: only top-level class declarations are implemented",
        ),
        (
            r#"<?php
trait Logs {
    public function write($message) {}
}
"#,
            2,
            1,
            "unsupported trait declaration: trait parsing and trait use execution are not implemented",
        ),
        (
            r#"<?php
interface Logger {
    public function write($message);
}
"#,
            2,
            1,
            "unsupported interface declaration: interface parsing and implementation execution are not implemented",
        ),
        (
            r#"<?php
enum Status {
    case Draft;
}
"#,
            2,
            1,
            "unsupported enum declaration: enum parsing and case/value execution are not implemented",
        ),
        (
            r#"<?php
abstract class Base {}
"#,
            2,
            1,
            "unsupported class modifier: abstract, final, and readonly class modifiers are not implemented",
        ),
        (
            r#"<?php
final class Leaf {}
"#,
            2,
            1,
            "unsupported class modifier: abstract, final, and readonly class modifiers are not implemented",
        ),
        (
            r#"<?php
readonly class Value {}
"#,
            2,
            1,
            "unsupported class modifier: abstract, final, and readonly class modifiers are not implemented",
        ),
        (
            r#"<?php
if (true) {
    trait NestedTrait {}
}
"#,
            3,
            5,
            "unsupported trait declaration: trait parsing and trait use execution are not implemented",
        ),
        (
            r#"<?php
if (true) {
    interface NestedLogger {}
}
"#,
            3,
            5,
            "unsupported interface declaration: interface parsing and implementation execution are not implemented",
        ),
        (
            r#"<?php
if (true) {
    enum NestedStatus {}
}
"#,
            3,
            5,
            "unsupported enum declaration: enum parsing and case/value execution are not implemented",
        ),
        (
            r#"<?php
if (true) {
    abstract class NestedBase {}
}
"#,
            3,
            5,
            "unsupported class modifier: abstract, final, and readonly class modifiers are not implemented",
        ),
        (
            r#"<?php
class Child extends Base {}
"#,
            2,
            13,
            "unsupported class inheritance: extends is not implemented",
        ),
        (
            r#"<?php
class Service implements Logger {}
"#,
            2,
            15,
            "unsupported interface implementation: implements clauses are not implemented",
        ),
        (
            r#"<?php
class Box {
    public string $name;
}
"#,
            3,
            12,
            "unsupported property type declaration: typed property storage and enforcement are not implemented",
        ),
        (
            r#"<?php
class Box {
    public ?string $name;
}
"#,
            3,
            12,
            "unsupported property type declaration: typed property storage and enforcement are not implemented",
        ),
        (
            r#"<?php
class Box {
    public int|string $id;
}
"#,
            3,
            12,
            "unsupported property type declaration: typed property storage and enforcement are not implemented",
        ),
        (
            r#"<?php
class Base {
    abstract public function compute();
}
"#,
            3,
            5,
            "unsupported class member modifier: abstract, final, and readonly member modifiers are not implemented",
        ),
        (
            r#"<?php
class Leaf {
    public final function seal() {}
}
"#,
            3,
            12,
            "unsupported class member modifier: abstract, final, and readonly member modifiers are not implemented",
        ),
        (
            r#"<?php
class Value {
    public readonly $id;
}
"#,
            3,
            12,
            "unsupported class member modifier: abstract, final, and readonly member modifiers are not implemented",
        ),
        (
            r#"<?php
class Box {
    public $name = "Ada";
}
"#,
            3,
            18,
            "unsupported property default: property default values are not implemented",
        ),
        (
            r#"<?php
class Box {
    public $name, $email;
}
"#,
            3,
            17,
            "unsupported property declaration: multiple properties in one declaration are not implemented",
        ),
        (
            r#"<?php
class Box {
    public const VERSION = 1;
}
"#,
            3,
            12,
            "unsupported class constant declaration: class constant metadata and lookup are not implemented",
        ),
        (
            r#"<?php
class Box {
    use Labels;
}
"#,
            3,
            5,
            "unsupported trait use: trait composition inside classes is not implemented",
        ),
        (
            r#"<?php
class Box {
    private const string NAME = "box";
}
"#,
            3,
            13,
            "unsupported class constant declaration: class constant metadata and lookup are not implemented",
        ),
        (
            r#"<?php
class Box {
    public function label() {
        return $this->name;
    }
}
"#,
            4,
            16,
            "unsupported object context: $this requires method execution and object binding, which are not implemented",
        ),
        (
            r#"<?php
echo $this;
"#,
            2,
            6,
            "unsupported object context: $this requires method execution and object binding, which are not implemented",
        ),
        (
            r#"<?php
class Box {}
$box = new Box();
$copy = clone $box;
"#,
            4,
            9,
            "unsupported clone expression: object handle copying and __clone dispatch are not implemented",
        ),
        (
            r#"<?php
class Box {}
$box = new Box();
$copy = CLONE $box;
"#,
            4,
            9,
            "unsupported clone expression: object handle copying and __clone dispatch are not implemented",
        ),
        (
            r#"<?php
class Box {}
$box = new Box();
echo $box instanceof Box;
"#,
            4,
            11,
            "unsupported instanceof expression: class/interface relationship checks are not implemented",
        ),
        (
            r#"<?php
class Box {}
$box = new Box();
echo $box INSTANCEOF Box;
"#,
            4,
            11,
            "unsupported instanceof expression: class/interface relationship checks are not implemented",
        ),
        (
            r#"<?php
echo Box::class;
"#,
            2,
            9,
            "unsupported class name constant: ::class resolution is not implemented",
        ),
        (
            r#"<?php
echo Box::CLASS;
"#,
            2,
            9,
            "unsupported class name constant: ::class resolution is not implemented",
        ),
        (
            r#"<?php
self::$value;
"#,
            2,
            5,
            "unsupported magic static receiver: self, parent, and static resolution is not implemented",
        ),
        (
            r#"<?php
parent::make();
"#,
            2,
            7,
            "unsupported magic static receiver: self, parent, and static resolution is not implemented",
        ),
        (
            r#"<?php
static::class;
"#,
            2,
            7,
            "unsupported magic static receiver: self, parent, and static resolution is not implemented",
        ),
        (
            r#"<?php
Box::$cache;
"#,
            2,
            4,
            "unsupported static property access: static property storage is not implemented",
        ),
        (
            r#"<?php
Box::make();
"#,
            2,
            4,
            "unsupported static method call: static method dispatch is not implemented",
        ),
        (
            r#"<?php
Box::VERSION;
"#,
            2,
            4,
            "unsupported class constant access: class constants are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}
