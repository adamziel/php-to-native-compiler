use php_compiler::error::{Diagnostic, Phase};
use php_compiler::{class_metadata_source, run_source, run_source_with_source_file};
use php_runtime::Visibility;

const LLVM_CLASS_NAME_CONSTANT_REJECTION: &str = "LLVM class-name constant lowering rejects ClassName::class, self::class, parent::class, and static::class until native class-name resolution, active class/parent and late-static-binding context, namespace/import canonicalization, autoload-free class lookup interaction, references/copy-on-write, and exact native class-name constant diagnostics exist; phpc run handles current bounded class-name constant behavior";
const LLVM_STATIC_MEMBER_REJECTION: &str = "LLVM static-member lowering rejects class constants, static property reads/writes, and dynamic static-property receivers until native class constant tables, static property storage, class context and late-static-binding resolution, visibility checks, autoload/class lookup, references/copy-on-write, and exact native static-member errors exist; phpc run handles current bounded static-member behavior";
const LLVM_NATIVE_ARRAY_NON_LOCAL_ASSIGNMENT_REJECTION: &str = "LLVM native array non-local assignment lowering rejects object, dynamic-object, non-direct object, and static property assignment targets until non-local owner cells, magic property writes, typed/static property state, assignment-expression results, references/copy-on-write, and exact diagnostics share one assignment owner contract; local variables and native array offset assignments use their shared native lvalue assignment contracts";
const CORE_CLASS_NAMES: &[&str] = &[
    "Exception",
    "Error",
    "Uri\\InvalidUriException",
    "Uri\\WhatWg\\InvalidUrlException",
    "RequestParseBodyException",
    "stdClass",
    "PhpToken",
    "mysqli",
    "mysqli_result",
    "mysqli_stmt",
    "mysqli_driver",
    "PDO",
    "PDOStatement",
    "RoundingMode",
    "Uri\\UriComparisonMode",
    "Uri\\Rfc3986\\UriHostType",
    "Uri\\Rfc3986\\UriType",
    "Uri\\Rfc3986\\Uri",
    "Uri\\WhatWg\\UrlHostType",
    "Uri\\WhatWg\\Url",
    "BcMath\\Number",
    "DateObjectError",
    "DateException",
    "HashContext",
    "DateTimeZone",
    "ReflectionException",
    "Attribute",
    "ReflectionClass",
    "ReflectionObject",
    "ReflectionFunction",
    "ReflectionMethod",
    "ReflectionParameter",
    "ReflectionType",
    "ReflectionNamedType",
    "ReflectionUnionType",
    "ReflectionIntersectionType",
    "ReflectionProperty",
    "ReflectionClassConstant",
    "ReflectionAttribute",
    "TypeError",
    "ArgumentCountError",
    "ValueError",
    "ArithmeticError",
    "DivisionByZeroError",
    "RuntimeException",
    "OutOfRangeException",
    "UnexpectedValueException",
    "OutOfBoundsException",
    "Directory",
    "SplFixedArray",
    "ArrayObject",
    "ArrayIterator",
    "SplDoublyLinkedList",
    "SplQueue",
    "SplStack",
    "SplObjectStorage",
    "SplFileObject",
    "EmptyIterator",
    "InfiniteIterator",
    "LimitIterator",
    "ReflectionExtension",
    "ReflectionZendExtension",
    "DateMalformedIntervalStringException",
    "DateInterval",
    "DateTime",
    "DateTimeImmutable",
    "DOMException",
    "DOMNode",
    "DOMAttr",
    "DOMElement",
    "DOMDocument",
    "DOMDocumentType",
    "ErrorException",
    "Reflection",
    "Generator",
];
const CORE_INTERFACE_NAMES: &[&str] = &[
    "Traversable",
    "IteratorAggregate",
    "Iterator",
    "Serializable",
    "ArrayAccess",
    "Countable",
    "Stringable",
    "SplObserver",
    "SplSubject",
    "DateTimeInterface",
];

fn expected_print_r_array(values: &[&str]) -> String {
    let mut output = String::from("Array\n(\n");
    for (index, value) in values.iter().enumerate() {
        output.push_str(&format!("    [{index}] => {value}\n"));
    }
    output.push_str(")\n");
    output
}

fn parse_error(source: &str) -> Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn runtime_error(source: &str) -> Diagnostic {
    match run_source(source) {
        Err(error) => {
            assert_eq!(error.phase, Phase::Runtime);
            error
        }
        Ok(execution) => {
            assert_eq!(execution.exit_code, 255);
            if let Some((message, line)) = php_fatal_stdout_message(&execution.stdout) {
                assert_eq!(execution.stderr, "");
                Diagnostic::new(Phase::Runtime, line, 1, message)
            } else if let Some((message, line)) = php_startup_fatal_message(&execution.stderr) {
                assert_eq!(execution.stdout, "");
                Diagnostic::new(Phase::Runtime, line, 1, message)
            } else {
                panic!("expected runtime error or PHP fatal execution, got {execution:?}");
            }
        }
    }
}

fn php_fatal_stdout_message(stdout: &str) -> Option<(String, usize)> {
    let rest = stdout.strip_prefix("Fatal error: Uncaught ")?;
    let (_, after_kind) = rest.split_once(": ")?;
    let (message, after_message) = after_kind.split_once(" in Command line code:")?;
    let (line_text, _) = after_message.split_once('\n')?;
    let line = line_text.parse().ok()?;
    Some((message.to_string(), line))
}

fn php_startup_fatal_message(stderr: &str) -> Option<(String, usize)> {
    let rest = stderr.strip_prefix("Fatal error: ")?;
    let (message, line_text) = rest.rsplit_once(" in Command line code on line ")?;
    let line = line_text.parse().ok()?;
    Some((message.to_string(), line))
}

fn assert_php_startup_fatal(source: &str, source_file: &str, line: usize, message: &str) {
    let execution = run_source_with_source_file(source, source_file).unwrap();
    assert_eq!(execution.stdout, "");
    assert_eq!(
        execution.stderr,
        format!("Fatal error: {message} in {source_file} on line {line}")
    );
    assert_eq!(execution.exit_code, 255);
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
    let class_names = classes
        .classes()
        .iter()
        .map(|class| class.name())
        .collect::<Vec<_>>();
    let mut expected_class_names = CORE_CLASS_NAMES.to_vec();
    expected_class_names.push("Box");
    assert_eq!(class_names, expected_class_names);
    let mut normalized_class_names = class_names
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    normalized_class_names.sort();
    normalized_class_names.dedup();
    assert_eq!(normalized_class_names.len(), class_names.len());

    let exception = classes.lookup_class("Exception").unwrap();
    for property in ["message", "code", "previous"] {
        let metadata = exception.property(property).unwrap();
        assert_eq!(metadata.visibility(), Visibility::Protected);
        assert!(!metadata.is_static());
    }
    let error = classes.lookup_class("Error").unwrap();
    let message = error.property("message").unwrap();
    assert_eq!(message.visibility(), Visibility::Public);
    assert!(!message.is_static());
    let type_error = classes.lookup_class("TypeError").unwrap();
    assert_eq!(type_error.parent_id(), Some(error.id()));
    let value_error = classes.lookup_class("ValueError").unwrap();
    assert_eq!(value_error.parent_id(), Some(error.id()));
    let arithmetic_error = classes.lookup_class("ArithmeticError").unwrap();
    assert_eq!(arithmetic_error.parent_id(), Some(error.id()));
    let division_by_zero_error = classes.lookup_class("DivisionByZeroError").unwrap();
    assert_eq!(
        division_by_zero_error.parent_id(),
        Some(arithmetic_error.id())
    );
    let runtime_exception = classes.lookup_class("RuntimeException").unwrap();
    assert_eq!(runtime_exception.parent_id(), Some(exception.id()));
    let out_of_range_exception = classes.lookup_class("OutOfRangeException").unwrap();
    assert_eq!(
        out_of_range_exception.parent_id(),
        Some(runtime_exception.id())
    );
    let out_of_bounds_exception = classes.lookup_class("OutOfBoundsException").unwrap();
    assert_eq!(
        out_of_bounds_exception.parent_id(),
        Some(runtime_exception.id())
    );
    let spl_fixed_array = classes.lookup_class("SplFixedArray").unwrap();
    assert!(spl_fixed_array.method("offsetGet").is_some());
    assert!(spl_fixed_array.method("fromArray").is_some());
    let array_object = classes.lookup_class("ArrayObject").unwrap();
    assert!(array_object.method("getIterator").is_some());
    assert!(array_object.constant("ARRAY_AS_PROPS").is_some());
    assert_eq!(
        array_object.property("storage").unwrap().visibility(),
        Visibility::Private
    );
    let array_iterator = classes.lookup_class("ArrayIterator").unwrap();
    assert!(array_iterator.method("seek").is_some());
    assert!(array_iterator.constant("STD_PROP_LIST").is_some());
    assert_eq!(
        array_iterator.property("storage").unwrap().visibility(),
        Visibility::Private
    );
    let spl_doubly_linked_list = classes.lookup_class("SplDoublyLinkedList").unwrap();
    assert!(spl_doubly_linked_list.constant("IT_MODE_LIFO").is_some());
    assert_eq!(
        spl_doubly_linked_list
            .property("flags")
            .unwrap()
            .visibility(),
        Visibility::Private
    );
    assert_eq!(
        spl_doubly_linked_list
            .property("dllist")
            .unwrap()
            .visibility(),
        Visibility::Private
    );
    assert_eq!(
        classes.lookup_class("SplQueue").unwrap().parent_id(),
        Some(spl_doubly_linked_list.id())
    );
    assert_eq!(
        classes.lookup_class("SplStack").unwrap().parent_id(),
        Some(spl_doubly_linked_list.id())
    );
    let spl_file_object = classes.lookup_class("SplFileObject").unwrap();
    assert!(spl_file_object.constant("READ_CSV").is_some());
    assert!(spl_file_object.method("setCsvControl").is_some());

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
fn class_declarations_record_single_parent_metadata() {
    let source = r#"<?php
class Base {}
class Child extends Base {}
echo "ready";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "ready");

    let classes = class_metadata_source(source).unwrap();
    let base = classes.lookup_class("base").unwrap();
    let child = classes.lookup_class("CHILD").unwrap();
    assert_eq!(child.parent_id(), Some(base.id()));
}

#[test]
fn namespaced_class_declarations_record_single_parent_metadata() {
    let source = r#"<?php
namespace Synthetic\WordPress;

class BaseLoader {}
class Loader extends BaseLoader {}

$loader = new Loader();
echo Loader::class, "\n";
echo get_parent_class($loader), "\n";
echo is_subclass_of($loader, BaseLoader::class) ? "yes" : "no";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Synthetic\\WordPress\\Loader\nSynthetic\\WordPress\\BaseLoader\nyes"
    );

    let classes = class_metadata_source(source).unwrap();
    let base = classes
        .lookup_class("Synthetic\\WordPress\\BaseLoader")
        .unwrap();
    let child = classes
        .lookup_class("Synthetic\\WordPress\\Loader")
        .unwrap();
    assert_eq!(child.parent_id(), Some(base.id()));
}

#[test]
fn class_inheritance_metadata_reports_unsupported_boundaries() {
    let missing_parent = runtime_error("<?php\nclass Child extends Missing {}\n");
    assert_eq!(missing_parent.line, 2);
    assert_eq!(missing_parent.column, 1);
    assert_eq!(missing_parent.message, "undefined class Missing");

    let namespaced_missing_parent = runtime_error(
        "<?php\nnamespace Synthetic\\WordPress;\nclass Loader extends BaseLoader {}\n",
    );
    assert_eq!(namespaced_missing_parent.line, 3);
    assert_eq!(namespaced_missing_parent.column, 1);
    assert_eq!(
        namespaced_missing_parent.message,
        "undefined class Synthetic\\WordPress\\BaseLoader"
    );

    let self_parent = runtime_error("<?php\nclass Box extends Box {}\n");
    assert_eq!(self_parent.line, 2);
    assert_eq!(self_parent.column, 1);
    assert_eq!(
        self_parent.message,
        "unsupported class inheritance for Box: cyclic inheritance is not implemented"
    );
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
fn dynamic_variable_class_name_instantiates_declared_classes() {
    let source = r#"<?php
class Box {
    public $value;

    public function __construct($value = "default") {
        $this->value = $value;
    }
}

$class = "box";
$box = new $class("dynamic");
echo get_class($box), "|", $box->value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "Box|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_variable_class_name_reports_stable_boundaries() {
    let non_string = runtime_error(
        r#"<?php
class Box {}
$class = 42;
$box = new $class();
"#,
    );
    assert_eq!(non_string.line, 4);
    assert_eq!(non_string.column, 8);
    assert_eq!(
        non_string.message,
        "unsupported object instantiation for dynamic class name: dynamic class variable must contain a string in the current subset, got int"
    );

    let missing = runtime_error(
        r#"<?php
$class = "Missing";
$box = new $class();
"#,
    );
    assert_eq!(missing.line, 3);
    assert_eq!(missing.column, 1);
    assert_eq!(missing.message, "Class \"Missing\" not found");
}

#[test]
fn public_instance_property_reads_and_writes_mutate_object_slots() {
    let source = r#"<?php
class Account {
    public $id;
}

class Profile extends Account {
    public $name;
    public $visits;
    protected $secret;
    private $token;
    private static $cache;
}

$profile = new profile();
echo "parent-initial:", $profile->id, "\n";
$profile->id = 7;
echo "initial:", $profile->name, "\n";
$profile->name = "Ada";
$profile->visits = 3;
echo $profile->id, "\n";
echo $profile->name, "\n";
echo $profile->visits + 2, "\n";
print_r($profile);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "parent-initial:\ninitial:\n7\nAda\n5\nProfile Object\n(\n    [id] => 7\n    [name] => Ada\n    [visits] => 3\n    [secret:protected] => \n    [token:Profile:private] => \n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn legacy_var_properties_are_public_untyped_slots() {
    let execution = run_source(
        r#"<?php
class LegacyBox {
    var $name = "seed";
    var $count;
}
$box = new LegacyBox();
echo $box->name, "|";
$box->count = 3;
echo $box->count, "|";
echo property_exists($box, "name") ? "prop" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed|3|prop");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_nested_array_assignments_materialize_current_subset() {
    let source = r#"<?php
class Bag {
    public $items;
}

$bag = new Bag();
$outer = "translation.mo";
$locale = "en_US";
$domain = "default";
$bag->items[$outer][$locale][$domain] = "loaded";
$bag->items[$outer]["fr_FR"]["default"] = "charge";
$bag->items[$outer][$locale][] = "fallback";
$bag->items[] = "root-append";
echo $bag->items[$outer][$locale][$domain], "\n";
echo $bag->items[$outer]["fr_FR"]["default"], "\n";
echo $bag->items[$outer][$locale][0], "\n";
echo $bag->items[0], "\n";
print_r($bag->items);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "loaded\ncharge\nfallback\nroot-append\nArray\n(\n    [translation.mo] => Array\n        (\n            [en_US] => Array\n                (\n                    [default] => loaded\n                    [0] => fallback\n                )\n\n            [fr_FR] => Array\n                (\n                    [default] => charge\n                )\n\n        )\n\n    [0] => root-append\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_nested_array_assignments_reject_non_array_roots() {
    let error = runtime_error(
        r#"<?php
class Bag {
    public $items;
}

$bag = new Bag();
$bag->items = "not-array";
$bag->items["key"]["child"] = "value";
"#,
    );

    assert_eq!(
        error.message,
        "invalid array access: cannot write offset on string"
    );
}

#[test]
fn object_property_nested_array_unset_removes_current_subset() {
    let source = r#"<?php
class Bag {
    public $items;
    public $empty;
}

$bag = new Bag();
$bag->items["translation.mo"]["en_US"]["default"] = "loaded";
$bag->items["translation.mo"]["en_US"]["fallback"] = "fallback";
$bag->items["translation.mo"]["fr_FR"]["default"] = "charge";
unset($bag->items["translation.mo"]["en_US"]["default"]);
unset($bag->items["missing"]["path"]);
unset($bag->empty["path"]);
echo $bag->items["translation.mo"]["en_US"]["fallback"], "\n";
echo $bag->items["translation.mo"]["fr_FR"]["default"], "\n";
print_r($bag->items);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "fallback\ncharge\nArray\n(\n    [translation.mo] => Array\n        (\n            [en_US] => Array\n                (\n                    [fallback] => fallback\n                )\n\n            [fr_FR] => Array\n                (\n                    [default] => charge\n                )\n\n        )\n\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_nested_array_unset_rejects_non_array_roots() {
    let error = runtime_error(
        r#"<?php
class Bag {
    public $items;
}

$bag = new Bag();
$bag->items = "not-array";
unset($bag->items["key"]["child"]);
"#,
    );

    assert_eq!(
        error.message,
        "invalid array access: cannot unset offset on string"
    );
}

#[test]
fn object_property_unset_nulls_visible_property_slot() {
    let source = r#"<?php
class Box {
    public $name;
    public $other;
}

$box = new Box();
$box->name = "Ada";
$box->other = "kept";
unset($box->name);
echo isset($box->name) ? "set" : "unset";
echo "|", empty($box->name) ? "empty" : "filled";
echo "|", $box->other;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "unset|empty|kept");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn multiple_object_property_unset_operands_run_left_to_right() {
    let source = r#"<?php
class Box {
    public $first;
    public $second;
}

$box = new Box();
$box->first = "one";
$box->second = "two";
unset($box->first, $box->second, $box->missing);
echo isset($box->first) ? "first" : "no-first";
echo "|", isset($box->second) ? "second" : "no-second";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "no-first|no-second");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_object_property_unset_uses_current_property_name() {
    let source = r#"<?php
class Box {
    public $name;
    public $other;
}

$box = new Box();
$property = "name";
$box->name = "Ada";
$box->other = "kept";
unset($box->$property);
echo isset($box->name) ? "set" : "unset";
echo "|", $box->other;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "unset|kept");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_public_property_names_read_write_existing_slots_and_stdclass_slots() {
    let source = r#"<?php
class Account {
    public $id;
}

class Profile extends Account {
    public $name;
}

$profile = new Profile();
$id = "id";
$name = "name";
$profile->$id = 7;
$profile->$name = "Ada";
echo $profile->id, "|", $profile->$id, "|", $profile->$name, "\n";

$data = new stdClass();
$key = "answer";
$data->$key = 42;
echo $data->answer, "|", $data->$key, "\n";

$intKey = 7;
$data->$intKey = "seven";
echo $data->$intKey;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "7|7|Ada\n42|42\nseven");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_public_property_names_materialize_wordpress_wpdb_slots() {
    let source = r#"<?php
class wpdb {}

$db = new wpdb();
$table = "categories";
$db->$table = "wp_categories";
echo $db->categories, "|", $db->$table;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "wp_categories|wp_categories");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn named_public_property_writes_materialize_stdclass_slots() {
    let source = r#"<?php
$data = new stdClass();
$data->answer = 42;
echo $data->answer, "|";

$data->args = array("x");
echo $data->args[0];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "42|x");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn braced_dynamic_property_names_read_write_current_expression_subset() {
    let source = r#"<?php
class Account {
    public $id;
}

$name = "id";
$account = new Account();
$account->{$name} = 7;
echo $account->{ "i" . "d" };
echo "|";

$data = new stdClass();
$slot = "answer";
$data->{$slot} = 42;
echo $data->{ $slot };
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "7|42");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn keyword_object_property_names_read_write_current_subset() {
    let source = r#"<?php
class KeywordBox {
    public $public;
    public $class;
    public $function;
    public $match;
}

$box = new KeywordBox();
$box->public = "visibility";
$box->class = "class-name";
echo $box->public, "|", $box->class, "|";
$box->function = "callable";
$box->match = "expression";
echo $box->function, "|", $box->match;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "visibility|class-name|callable|expression"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn keyword_method_names_parse_before_runtime_dispatch() {
    let execution = run_source(
        r#"<?php
$data = new stdClass();
$data->public();
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("Call to undefined method stdClass::public()"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn dynamic_property_names_do_not_materialize_missing_slots_on_declared_classes() {
    let error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$name = "missing";
$box->$name = 1;
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined property Box::$missing");
}

#[test]
fn dynamic_property_names_reject_unsupported_name_values() {
    let error = runtime_error(
        r#"<?php
$data = new stdClass();
$name = [];
$data->$name = 1;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid property access: dynamic property names only support strings and integers in the current subset, got array"
    );
}

#[test]
fn same_class_non_public_instance_properties_read_and_write_from_methods() {
    let source = r#"<?php
class Box {
    private $secret;
    protected $label;

    public function set($secret, $label) {
        $this->secret = $secret;
        $this->label = $label;
    }

    public function describe() {
        return $this->secret . ":" . $this->label;
    }

    public function copyTo($other) {
        $other->secret = $this->secret;
        $other->label = "copy";
    }
}

$first = new Box();
$second = new Box();
$first->set("one", "main");
echo $first->describe(), "\n";
$first->copyTo($second);
echo $second->describe();
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "one:main\none:copy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn same_class_non_public_instance_properties_support_isset_and_empty() {
    let source = r#"<?php
class Box {
    private $secret;
    protected $label;

    public function set($secret, $label) {
        $this->secret = $secret;
        $this->label = $label;
    }

    public function checks($other) {
        echo isset($this->secret) ? "this-secret:set\n" : "this-secret:unset\n";
        echo empty($this->secret) ? "this-secret:empty\n" : "this-secret:not-empty\n";
        echo isset($other->label) ? "peer-label:set\n" : "peer-label:unset\n";
        echo empty($other->label) ? "peer-label:empty" : "peer-label:not-empty";
    }
}

$first = new Box();
$second = new Box();
$first->set("0", "main");
$second->set(null, "");
$first->checks($second);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "this-secret:set\nthis-secret:empty\npeer-label:set\npeer-label:empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn same_class_non_public_instance_properties_support_read_modify_write() {
    let source = r#"<?php
class Counter {
    private $count;
    protected $score;

    public function seed($count, $score) {
        $this->count = $count;
        $this->score = $score;
    }

    public function bump($other) {
        $this->count += 4;
        $this->score *= 3;
        echo $this->count, ":", $this->score, "\n";
        echo $other->count++, "\n";
        echo ++$other->score, "\n";
        $other->count .= "!";
    }

    public function describe() {
        return $this->count . ":" . $this->score;
    }
}

$first = new Counter();
$second = new Counter();
$first->seed(6, 2);
$second->seed(10, 20);
$first->bump($second);
echo $second->describe();
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "10:6\n10\n21\n11!:21");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn same_class_non_public_instance_properties_support_null_coalescing() {
    let source = r#"<?php
class Box {
    private $secret;
    protected $label;

    public function seed($secret, $label) {
        $this->secret = $secret;
        $this->label = $label;
    }

    public function coalesce($other) {
        echo ($this->secret ?? "secret-fallback"), "\n";
        echo ($this->missing ?? "missing-fallback"), "\n";
        $this->secret ??= "secret-assigned";
        $this->label ??= "label-replaced";
        $other->secret ??= "peer-secret";
        $other->label ??= "peer-label";
        echo $this->secret, ":", $this->label, "\n";
        echo $other->secret, ":", $other->label;
    }
}

$first = new Box();
$second = new Box();
$first->seed(null, "kept");
$second->seed("existing", null);
$first->coalesce($second);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "secret-fallback\nmissing-fallback\nsecret-assigned:kept\nexisting:peer-label"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_non_public_instance_slots_use_declaring_class_context() {
    let source = r#"<?php
class Base {
    private $token;
    protected $shared;

    public function seedBase($token, $shared) {
        $this->token = $token;
        $this->shared = $shared;
    }

    public function describeBase() {
        return $this->token . ":" . $this->shared;
    }
}

class Child extends Base {
    private $childToken;
    protected $childShared;

    public function seedChild($token, $shared) {
        $this->childToken = $token;
        $this->childShared = $shared;
    }

    public function describeChild() {
        return $this->childToken . ":" . $this->childShared;
    }
}

$child = new Child();
$child->seedBase("base-token", "base-shared");
$child->seedChild("child-token", "child-shared");
echo $child->describeBase(), "\n";
echo $child->describeChild(), "\n";
print_r($child);
echo "done";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "base-token:base-shared\nchild-token:child-shared\nChild Object\n(\n    [token:Base:private] => base-token\n    [shared:protected] => base-shared\n    [childToken:Child:private] => child-token\n    [childShared:protected] => child-shared\n)\ndone"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn child_context_can_access_parent_declared_protected_properties() {
    let source = r#"<?php
class Base {
    protected $shared;
    protected $count;

    public function seedBase($shared, $count) {
        $this->shared = $shared;
        $this->count = $count;
    }

    public function describeBase() {
        return $this->shared . ":" . $this->count;
    }
}

class Child extends Base {
    public function updateFromChild($other) {
        echo $this->shared, "\n";
        echo isset($other->shared) ? "peer-set\n" : "peer-unset\n";
        echo empty($other->shared) ? "peer-empty\n" : "peer-filled\n";
        $this->count += 2;
        ++$other->count;
        $other->shared ??= "filled";
        echo $this->describeBase(), "\n";
        echo $other->describeBase();
    }
}

$first = new Child();
$second = new Child();
$first->seedBase("first", 4);
$second->seedBase(null, 9);
$first->updateFromChild($second);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "first\npeer-unset\npeer-empty\nfirst:6\nfilled:10"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_handles_preserve_identity_across_supported_value_copies() {
    let source = r#"<?php
class Box {
    public $value;
}

function set_value($object, $value) {
    $object->value = $value;
    return $object;
}

$box = new Box();
$alias = $box;
$alias->value = "alias";
echo $box->value, "\n";

$items = [$box];
$fromArray = $items[0];
$fromArray->value = "array";
echo $box->value, "\n";

set_value($box, "function");
echo $box->value, "\n";

$returned = set_value($box, "return");
echo $box->value, "\n";

foreach ([$box] as $item) {
    $item->value = "foreach";
}
echo $box->value, "\n";

var_dump($box === $alias);
var_dump($box === $returned);
var_dump($box === new Box());
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "alias\narray\nfunction\nreturn\nforeach\nbool(true)\nbool(true)\nbool(false)\n"
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
fn empty_public_instance_properties_checks_current_truthiness() {
    let source = r#"<?php
class Profile {
    public $name;
    public $isFalse;
    public $zero;
    public $emptyString;
    public $zeroString;
    public $items;
    public $filled;
    private $token;
}

$profile = new Profile();
$profile->isFalse = false;
$profile->zero = 0;
$profile->emptyString = "";
$profile->zeroString = "0";
$profile->items = [];
$profile->filled = "Ada";

if (empty($profile->name)) {
    echo "null-slot:empty\n";
}
if (empty($profile->isFalse)) {
    echo "false:empty\n";
}
if (empty($profile->zero)) {
    echo "zero:empty\n";
}
if (empty($profile->emptyString)) {
    echo "empty-string:empty\n";
}
if (empty($profile->zeroString)) {
    echo "zero-string:empty\n";
}
if (empty($profile->items)) {
    echo "empty-array:empty\n";
}
if (empty($profile->filled)) {
    echo "filled:empty\n";
} else {
    echo "filled:not-empty\n";
}
if (empty($profile->missing)) {
    echo "missing-property:empty\n";
}
$value = 42;
if (empty($value->name)) {
    echo "scalar-target:empty\n";
}
if (empty($missing->name)) {
    echo "missing-target:empty";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "null-slot:empty\nfalse:empty\nzero:empty\nempty-string:empty\nzero-string:empty\nempty-array:empty\nfilled:not-empty\nmissing-property:empty\nscalar-target:empty\nmissing-target:empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_get_and_isset_run_for_missing_direct_properties() {
    let source = r#"<?php
class Bag {
    public $name = "declared";

    public function __get($property) {
        echo "get:$property\n";
        if ($property === "count") {
            return 0;
        }
        return "value:" . $property;
    }

    public function __isset($property) {
        echo "isset:$property\n";
        return $property === "title" || $property === "count";
    }
}

$bag = new Bag();
echo $bag->name, "\n";
echo $bag->title, "\n";
echo isset($bag->name) ? "name:set\n" : "name:unset\n";
echo isset($bag->title) ? "title:set\n" : "title:unset\n";
echo isset($bag->missing) ? "missing:set\n" : "missing:unset\n";
echo empty($bag->title) ? "title:empty\n" : "title:not-empty\n";
echo empty($bag->count) ? "count:empty\n" : "count:not-empty\n";
echo empty($bag->missing) ? "missing:empty" : "missing:not-empty";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "declared\n",
            "get:title\n",
            "value:title\n",
            "name:set\n",
            "isset:title\n",
            "title:set\n",
            "isset:missing\n",
            "missing:unset\n",
            "isset:title\n",
            "get:title\n",
            "title:not-empty\n",
            "isset:count\n",
            "get:count\n",
            "count:empty\n",
            "isset:missing\n",
            "missing:empty",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_get_runs_for_inaccessible_and_dynamic_property_reads() {
    let source = r#"<?php
class MagicReadBox {
    private $secret = "private";
    protected $settings = "protected";

    public function __get($property) {
        echo "get:$property\n";
        if ($property === "secret") {
            return "magic:" . $this->secret;
        }
        if ($property === "settings") {
            return "magic:" . $this->settings;
        }
        return "missing:" . $property;
    }
}

$box = new MagicReadBox();
echo $box->secret, "\n";
$property = "settings";
echo $box->{$property}, "\n";
$property = "dynamic";
echo $box->{$property}, "\n";
echo $box->missing;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "get:secret\n",
            "magic:private\n",
            "get:settings\n",
            "magic:protected\n",
            "get:dynamic\n",
            "missing:dynamic\n",
            "get:missing\n",
            "missing:missing",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_returning_magic_get_reads_by_value_for_normal_property_reads() {
    let source = r#"<?php
$slot = "initial";
$dynamicSlot = "dynamic-initial";

class RefMagicReadBox {
    private $secret = "declared";

    public function &__get($property) {
        echo "get:$property\n";
        global $slot;
        return $slot;
    }
}

class RefMagicDynamicReadBox {
    protected $dynamicSecret = "declared";

    public function &__get($property) {
        echo "get:$property\n";
        global $dynamicSlot;
        return $dynamicSlot;
    }
}

$box = new RefMagicReadBox();
$copy = $box->secret;
$slot = "changed";
echo $copy, "|", $box->secret, "\n";

$property = "dynamicSecret";
$dynamicBox = new RefMagicDynamicReadBox();
$dynamicCopy = $dynamicBox->{$property};
$dynamicSlot = "dynamic-changed";
echo $dynamicCopy, "|", $dynamicBox->{$property};
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "get:secret\n",
            "initial|get:secret\n",
            "changed\n",
            "get:dynamicSecret\n",
            "dynamic-initial|get:dynamicSecret\n",
            "dynamic-changed",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_binds_magic_get_array_offset_source() {
    let source = r#"<?php
$store = ["slot" => "initial", "nested" => ["leaf" => "nested-initial"]];
$dynamicStore = ["slot" => "dynamic-initial"];

class RefMagicArraySlotBox {
    public function &__get($property) {
        echo "get:$property\n";
        global $store;
        return $store;
    }
}

class RefMagicDynamicArraySlotBox {
    public function &__get($property) {
        echo "get:$property\n";
        global $dynamicStore;
        return $dynamicStore;
    }
}

$box = new RefMagicArraySlotBox();
$alias =& $box->missing["slot"];
$alias = "from-alias";
echo $store["slot"], "|";
$store["slot"] = "from-store";
echo $alias, "\n";

$nested =& $box->missing["nested"]["leaf"];
$nested = "from-nested";
echo $store["nested"]["leaf"], "|";
$store["nested"]["leaf"] = "from-store-nested";
echo $nested, "\n";

$property = "dynamicMissing";
$dynamicBox = new RefMagicDynamicArraySlotBox();
$dynamic =& $dynamicBox->{$property}["slot"];
$dynamic = "from-dynamic";
echo $dynamicStore["slot"], "|";
$dynamicStore["slot"] = "from-dynamic-store";
echo $dynamic;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "get:missing\n",
            "from-alias|from-store\n",
            "get:missing\n",
            "from-nested|from-store-nested\n",
            "get:dynamicMissing\n",
            "from-dynamic|from-dynamic-store",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_binds_magic_get_array_append_source() {
    let source = r#"<?php
$store = [];
$dynamicStore = [];

class RefMagicArrayAppendBox {
    public function &__get($property) {
        echo "get:$property\n";
        global $store;
        return $store;
    }
}

class RefMagicDynamicArrayAppendBox {
    public function &__get($property) {
        echo "get:$property\n";
        global $dynamicStore;
        return $dynamicStore;
    }
}

$box = new RefMagicArrayAppendBox();
$alias =& $box->missing[];
$alias = "from-alias";
echo $store[0], "|";
$store[0] = "from-store";
echo $alias, "\n";

$property = "dynamicMissing";
$dynamicBox = new RefMagicDynamicArrayAppendBox();
$dynamic =& $dynamicBox->{$property}[];
$dynamic = "from-dynamic";
echo $dynamicStore[0], "|";
$dynamicStore[0] = "from-dynamic-store";
echo $dynamic;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "get:missing\n",
            "from-alias|from-store\n",
            "get:dynamicMissing\n",
            "from-dynamic|from-dynamic-store",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_set_runs_for_missing_direct_property_writes() {
    let source = r#"<?php
class Bag {
    public $name = "declared";
    public $log = [];

    public function __set($property, $value) {
        echo "set:$property=$value\n";
        $this->log[$property] = $value;
        return "ignored";
    }

    public function __get($property) {
        return $this->log[$property];
    }
}

$bag = new Bag();
$bag->name = "direct";
echo $bag->name, "\n";
$result = ($bag->title = "Hello");
echo "result:$result\n";
echo $bag->title;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "direct\nset:title=Hello\nresult:Hello\nHello"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_unset_runs_for_missing_direct_property_unset() {
    let source = r#"<?php
class Bag {
    public $name;
    public $log = [];

    public function __unset($property) {
        echo "unset:$property\n";
        $this->log[$property] = "gone";
    }

    public function __get($property) {
        return $this->log[$property];
    }
}

$bag = new Bag();
$bag->name = "Ada";
unset($bag->name);
echo isset($bag->name) ? "name:set\n" : "name:unset\n";
unset($bag->title);
echo $bag->title;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "name:unset\nunset:title\ngone");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_call_runs_for_missing_instance_methods() {
    let source = r#"<?php
class Router {
    public function known() {
        return "known";
    }

    public function __call($method, $args) {
        echo "call:$method\n";
        return $method . ":" . $args[0] . ":" . $args[1];
    }
}

$router = new Router();
echo $router->known(), "\n";
echo $router->route("posts", 7);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "known\ncall:route\nroute:posts:7");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_call_static_runs_for_missing_static_methods() {
    let source = r#"<?php
class Router {
    public static function known() {
        return "known";
    }

    public static function throughSelf() {
        return self::route("self", 1);
    }

    public static function throughStatic() {
        return static::route("late", 2);
    }

    public static function __callStatic($method, $args) {
        echo "static:$method\n";
        return $method . ":" . $args[0] . ":" . $args[1] . ":" . get_called_class();
    }
}

class Child extends Router {}

$class = "Router";
$object = new Router();
echo Router::known(), "\n";
echo Router::route("posts", 7), "\n";
echo Router::throughSelf(), "\n";
echo Child::throughStatic(), "\n";
echo $class::route("class", 3), "\n";
echo $object::route("object", 4);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "known\n",
            "static:route\n",
            "route:posts:7:Router\n",
            "static:route\n",
            "route:self:1:Router\n",
            "static:route\n",
            "route:late:2:Child\n",
            "static:route\n",
            "route:class:3:Router\n",
            "static:route\n",
            "route:object:4:Router",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn invalid_magic_call_signature_emits_php_startup_fatal() {
    let execution = run_source_with_source_file(
        r#"<?php
class Test {
    function __call() {
    }
}
echo "unreached";
"#,
        "tests/classes/__call_002.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "");
    assert_eq!(
        execution.stderr,
        "Fatal error: Method Test::__call() must take exactly 2 arguments in tests/classes/__call_002.php on line 3"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn invalid_property_magic_signature_emits_php_startup_fatal() {
    let execution = run_source_with_source_file(
        r#"<?php
class MagicBox {
    public function __unset(int $name) {}
}
echo "unreached";
"#,
        "tests/classes/magic_property_signature.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "");
    assert_eq!(
        execution.stderr,
        "Fatal error: MagicBox::__unset(): Parameter #1 ($name) must be of type string when declared in tests/classes/magic_property_signature.php on line 3"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn non_public_magic_method_emits_php_startup_warning_and_runs() {
    let execution = run_source_with_source_file(
        r#"<?php
class MagicBox {
    protected function __unset(string $name) {}
}
echo "reached";
"#,
        "Zend/tests/magic_methods/magic_methods_002.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "reached");
    assert_eq!(
        execution.stderr,
        "Warning: The magic method MagicBox::__unset() must have public visibility in Zend/tests/magic_methods/magic_methods_002.php on line 3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn var_dump_uses_debug_info_array_properties() {
    let execution = run_source(
        r#"<?php
class Foo {
    public function __debugInfo() {
        return array("a" => 1, "\0*\0b" => 2, "\0Foo\0c" => 3);
    }
}

var_dump(new Foo());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "object(Foo)#1 (3) {\n",
            "  [\"a\"]=>\n",
            "  int(1)\n",
            "  [\"b\":protected]=>\n",
            "  int(2)\n",
            "  [\"c\":\"Foo\":private]=>\n",
            "  int(3)\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn var_dump_debug_info_null_return_deprecates_and_dumps_empty_object() {
    let execution = run_source(
        r#"<?php
set_error_handler(function($errno, $message) {
    echo "deprecated:", $message, "\n";
    return true;
}, E_DEPRECATED);

class Bar {
    public function __debugInfo() {
        return null;
    }
}

var_dump(new Bar());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "deprecated:Returning null from Bar::__debugInfo() is deprecated, return an empty array instead\n",
            "object(Bar)#2 (0) {\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn var_dump_debug_info_non_array_return_emits_php_fatal() {
    let execution = run_source_with_source_file(
        r#"<?php
class C {
    public $val;
    public function __debugInfo() {
        return $this->val;
    }
    public function __construct($val) {
        $this->val = $val;
    }
}

$c = new C("foo");
var_dump($c);
echo "unreached";
"#,
        "Zend/tests/debug_info/debug_info-error-str.php",
    )
    .unwrap();

    assert!(execution
        .stdout
        .starts_with("Fatal error: __debuginfo() must return an array in Zend/tests/debug_info/debug_info-error-str.php on line "));
    assert!(!execution.stdout.contains("unreached"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn magic_to_string_runs_for_echo_print_cast_and_concat() {
    let source = r#"<?php
class Label {
    public $value = "core";

    public function __toString() {
        echo "toString\n";
        return "label:" . $this->value;
    }
}

$label = new Label();
echo $label, "\n";
echo (string) $label, "\n";
echo "prefix-" . $label, "\n";
print $label;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "toString\nlabel:core\ntoString\nlabel:core\ntoString\nprefix-label:core\ntoString\nlabel:core"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_to_string_runs_for_concat_assignment() {
    let source = r#"<?php
class Label {
    public $value = "core";

    public function __toString() {
        echo "toString:$this->value\n";
        return "label:" . $this->value;
    }
}

class Box {
    public $text = "box:";
}

$label = new Label();
$text = "prefix:";
$text .= $label;
echo $text, "\n";
$value = $label;
$value .= ":tail";
echo $value, "\n";
$box = new Box();
$box->text .= $label;
echo $box->text;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "toString:core\nprefix:label:core\ntoString:core\nlabel:core:tail\ntoString:core\nbox:label:core"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_to_string_runs_for_interpolated_strings() {
    let source = r#"<?php
class Label {
    public $value = "core";

    public function __toString() {
        echo "toString:$this->value\n";
        return "label:$this->value";
    }
}

class Holder {
    public $label;
}

$label = new Label();
$items = ["label" => $label];
$box = new Holder();
$box->label = $label;
echo "plain:$label\n";
echo "array:{$items['label']}\n";
echo "property:$box->label\n";
echo "chain:{$box->label}\n";
echo <<<TEXT
heredoc:$label
TEXT;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "toString:core\nplain:label:core\ntoString:core\narray:label:core\ntoString:core\nproperty:label:core\ntoString:core\nchain:label:core\ntoString:core\nheredoc:label:core"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_offsets_dispatch_to_user_methods() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        echo "exists:$offset\n";
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:" . ($offset === null ? "null" : $offset) . ":$value\n";
        if ($offset === null) {
            $this->items[] = $value;
        } else {
            $this->items[$offset] = $value;
        }
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        echo "unset:$offset\n";
        unset($this->items[$offset]);
    }
}

$bag = new Bag();
$bag["name"] = "Ada";
echo $bag["name"], "\n";
echo isset($bag["name"]) ? "isset\n" : "missing\n";
echo empty($bag["name"]) ? "empty\n" : "not-empty\n";
echo $bag["missing"] ?? "fallback", "\n";
unset($bag["name"]);
echo isset($bag["name"]) ? "isset\n" : "missing\n";
$bag[] = "tail";
echo $bag[0];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "set:name:Ada\nget:name\nAda\nexists:name\nisset\nexists:name\nget:name\nnot-empty\nexists:missing\nfallback\nunset:name\nexists:name\nmissing\nset:null:tail\nget:0\ntail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_and_isset_undefined_offsets_keep_warning_without_null_offset_deprecation() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        echo "exists:" . ($offset === null ? "null" : $offset) . "\n";
        return false;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return null;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:" . ($offset === null ? "null" : $offset) . ":$value\n";
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Bag();
$bag[$undef] = 1;
var_dump(isset($array[$missing]));
"#;

    let execution = run_source(source).unwrap();
    assert!(execution
        .stdout
        .contains("Warning: Undefined variable $undef"));
    assert!(execution
        .stdout
        .contains("Warning: Undefined variable $missing"));
    assert!(
        !execution
            .stdout
            .contains("Using null as an array offset is deprecated"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("set::1"), "{}", execution.stdout);
    assert!(
        execution.stdout.ends_with("bool(false)\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_offsets_support_compound_assignment() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    public $items = ["n" => 2, "s" => "a"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:$offset:$value\n";
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$bag = new Bag();
echo ($bag["n"] += 5), "\n";
echo ($bag["s"] .= "b"), "\n";
echo $bag["n"], ":", $bag["s"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "get:n\nset:n:7\n7\nget:s\nset:s:ab\nab\nget:n\n7:get:s\nab"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_offsets_support_increment_decrement() {
    let source = r#"<?php
error_reporting(0);

class Bag implements ArrayAccess {
    public $items = ["n" => 2, "f" => 1.5];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:$offset:$value\n";
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$bag = new Bag();
echo $bag["n"]++, "\n";
echo ++$bag["n"], "\n";
echo $bag["f"]--, "\n";
echo --$bag["f"], "\n";
$bag["n"]++;
echo $bag["n"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "get:n\n2\nget:n\n3\nget:f\n1.5\nget:f\n0.5\nget:n\nget:n\n2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_access_offsets_dispatch_to_user_methods() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        echo "exists:$offset\n";
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:$offset:$value\n";
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        echo "unset:$offset\n";
        unset($this->items[$offset]);
    }
}

class Holder {
    public $bag;
}

$holder = new Holder();
$holder->bag = new Bag();
$holder->bag["name"] = "Ada";
echo $holder->bag["name"], "\n";
echo isset($holder->bag["name"]) ? "isset\n" : "missing\n";
echo empty($holder->bag["name"]) ? "empty\n" : "not-empty\n";
echo $holder->bag["missing"] ?? "fallback", "\n";
unset($holder->bag["name"]);
echo isset($holder->bag["name"]) ? "isset" : "missing";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "set:name:Ada\nget:name\nAda\nexists:name\nisset\nexists:name\nget:name\nnot-empty\nexists:missing\nfallback\nunset:name\nexists:name\nmissing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_access_append_dispatches_to_offset_set_null() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:" . ($offset === null ? "null" : $offset) . ":$value\n";
        if ($offset === null) {
            $this->items[] = $value;
        } else {
            $this->items[$offset] = $value;
        }
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Holder {
    public $bag;
}

$holder = new Holder();
$holder->bag = new Bag();
$holder->bag[] = "tail";
echo $holder->bag[0];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "set:null:tail\nget:0\ntail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_access_offsets_support_compound_assignment() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    public $items = ["n" => 2, "s" => "a"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:$offset:$value\n";
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Holder {
    public $bag;
}

$holder = new Holder();
$holder->bag = new Bag();
echo ($holder->bag["n"] += 5), "\n";
echo ($holder->bag["s"] .= "b"), "\n";
echo $holder->bag["n"], ":", $holder->bag["s"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "get:n\nset:n:7\n7\nget:s\nset:s:ab\nab\nget:n\n7:get:s\nab"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_access_offsets_support_increment_decrement() {
    let source = r#"<?php
error_reporting(0);

class Bag implements ArrayAccess {
    public $items = ["n" => 2, "f" => 1.5];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:$offset:$value\n";
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Holder {
    public $bag;
}

$holder = new Holder();
$holder->bag = new Bag();
echo $holder->bag["n"]++, "\n";
echo ++$holder->bag["n"], "\n";
echo $holder->bag["f"]--, "\n";
echo --$holder->bag["f"], "\n";
$holder->bag["n"]++;
echo $holder->bag["n"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "get:n\n2\nget:n\n3\nget:f\n1.5\nget:f\n0.5\nget:n\nget:n\n2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn empty_non_public_property_access_remains_explicitly_unsupported() {
    let error = runtime_error(
        r#"<?php
class Box {
    private $secret;
}

$box = new Box();
echo empty($box->secret);
"#,
    );

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "Cannot access private property Box::$secret");
}

#[test]
fn complex_object_property_empty_operands_remain_explicitly_unsupported() {
    let error = runtime_error(
        r#"<?php
class Box {
    public $name;
}

function make_box() {
    return new Box();
}

echo empty(make_box()->name);
"#,
    );

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 10);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call empty(): only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported"
    );
}

#[test]
fn core_exception_constructor_initializes_bounded_state_for_subclasses() {
    let execution = run_source(
        r#"<?php
class InspectableException extends Exception {
    public function describe() {
        return $this->message . "|" . $this->code . "|" . get_class($this->previous);
    }
}
$previous = new Exception("root", 4);
$exception = new InspectableException("leaf", 7, $previous);
echo $exception->describe(), "\n";
echo $exception instanceof Exception ? "exception" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Error: Call to undefined method Error::describe() in Command line code:9\nStack trace:\n#0 {main}\n  thrown in Command line code on line 9"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn core_exception_constructor_initializes_default_state() {
    let execution = run_source(
        r#"<?php
class InspectableDefaultException extends Exception {
    public function describe() {
        return $this->message . "|" . $this->code . "|" . ($this->previous === null ? "null" : "object");
    }
}
$exception = new InspectableDefaultException;
echo $exception->describe();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Error: Call to undefined method Error::describe() in Command line code:8\nStack trace:\n#0 {main}\n  thrown in Command line code on line 8"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn get_class_returns_declared_class_name_for_minimal_objects() {
    let source = r#"<?php
class Box {}
class Profile {
    public $name;
}

$box = new box();
$profile = new PROFILE();
echo get_class($box), "\n";
$call = "get_class";
echo $call($profile), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "Box\nProfile\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_requires_object_argument() {
    let execution = run_source(
        r#"<?php
try {
    get_class(42);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:get_class(): Argument #1 ($object) must be of type object, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_object_checks_current_minimal_object_values() {
    let source = r#"<?php
class Box {}

$box = new box();
if (is_object($box)) {
    echo "box:object\n";
}
if (!is_object(42)) {
    echo "int:not-object\n";
}
if (!is_object(["box"])) {
    echo "array:not-object\n";
}
$call = "is_object";
if ($call($box)) {
    echo "dynamic:object\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "box:object\nint:not-object\narray:not-object\ndynamic:object\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_debug_type_reports_current_value_type_names() {
    let source = r#"<?php
class Box {}

$box = new box();
$values = [
    null,
    false,
    7,
    3.5,
    "x",
    ["nested"],
    $box,
];
foreach ($values as $value) {
    echo get_debug_type($value), "\n";
}
$call = "get_debug_type";
echo $call($box), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "null\nbool\nint\nfloat\nstring\narray\nBox\nBox\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn class_exists_checks_declared_class_metadata() {
    let source = r#"<?php
class Box {}

if (class_exists("box")) {
    echo "box:exists\n";
}
if (class_exists("BOX", false)) {
    echo "box:false-autoload\n";
}
if (!class_exists("Missing")) {
    echo "missing:not-exists\n";
}
$call = "class_exists";
if ($call("Box", true)) {
    echo "dynamic:exists\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "box:exists\nbox:false-autoload\nmissing:not-exists\ndynamic:exists\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn metadata_exists_accepts_scalar_autoload_flags() {
    let source = r#"<?php
class Box {}

var_dump(class_exists("BOX", 1));
var_dump(class_exists("Missing", 0));
var_dump(interface_exists("Box", "1"));
var_dump(trait_exists("Box", "0"));
var_dump(enum_exists("Box", 0.5));

$call = "class_exists";
var_dump($call("box", "false"));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn metadata_exists_coerces_scalar_names_and_rejects_non_scalars() {
    let source = r#"<?php
class Box {}
interface Contract {}
trait Shared {}
enum Mode { case On; }

var_dump(class_exists(1));
var_dump(class_exists(false));
var_dump(class_exists(null));
var_dump(class_exists("BOX"));
var_dump(interface_exists(true));
var_dump(interface_exists("Contract"));
var_dump(trait_exists(0.5));
var_dump(trait_exists("Shared"));
var_dump(enum_exists(42));
var_dump(enum_exists("Mode"));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let name_error = runtime_error("<?php\nvar_dump(class_exists([]));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call class_exists(): class-like name argument must be string-compatible scalar in the current subset, got array"
    );

    let autoload_error = runtime_error("<?php\nvar_dump(class_exists(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call class_exists(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn interface_exists_reports_declared_interface_metadata() {
    let source = r#"<?php
namespace App;
class Box {}
interface Logger {}
interface Hookable {}

if (!interface_exists("App\\Box")) {
    echo "class:not-interface\n";
}
if (interface_exists("App\\Logger")) {
    echo "logger:interface\n";
}
if (interface_exists("App\\Hookable")) {
    echo "namespaced:interface\n";
}
if (!interface_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}
$call = "interface_exists";
if ($call("APP\\LOGGER", true)) {
    echo "dynamic:interface\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "class:not-interface\nlogger:interface\nnamespaced:interface\nmissing:false-autoload\ndynamic:interface\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn stringable_core_interface_reflects_to_string_metadata() {
    let source = r#"<?php
class Label {
    public function __toString() {
        return "label";
    }
}

class ChildLabel extends Label {}

class ExplicitLabel implements Stringable {
    public function __toString() {
        return "explicit";
    }
}

class Plain {}

$label = new Label();
$child = new ChildLabel();
$explicit = new ExplicitLabel();
$plain = new Plain();

echo interface_exists("Stringable") ? "interface\n" : "missing\n";
echo $label instanceof Stringable ? "instanceof\n" : "no-instanceof\n";
echo is_a($child, "Stringable") ? "child:is-a\n" : "child:no\n";
echo is_subclass_of("ChildLabel", "Stringable") ? "child:subclass\n" : "child:no-subclass\n";
echo is_a($explicit, "Stringable") ? "explicit:is-a\n" : "explicit:no\n";
echo is_a($plain, "Stringable") ? "plain:is-a\n" : "plain:no\n";
$implements = class_implements($child);
echo isset($implements["Stringable"]) ? "implements:stringable\n" : "implements:missing\n";
echo in_array("Stringable", get_declared_interfaces(), true) ? "declared\n" : "not-declared\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "interface\ninstanceof\nchild:is-a\nchild:subclass\nexplicit:is-a\nplain:no\nimplements:stringable\ndeclared\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn classes_record_interface_implementation_metadata() {
    let execution = run_source(
        r#"<?php
interface Logger {}
interface Hookable {}
class Service implements Logger, Hookable {}
class ChildService extends Service {}

$service = new Service();
$child = new ChildService();
if (is_a($service, "Logger")) {
    echo "service:logger\n";
}
if (is_subclass_of($service, "Hookable")) {
    echo "service:hookable\n";
}
if (is_a($child, "Logger")) {
    echo "child:inherits-interface\n";
}
if (is_a("ChildService", "Hookable", true)) {
    echo "string:inherits-interface\n";
}

final class WP_Hook implements Iterator, ArrayAccess {
    #[ReturnTypeWillChange]
    public function current() { return null; }
    #[ReturnTypeWillChange]
    public function key() { return null; }
    #[ReturnTypeWillChange]
    public function next() { return null; }
    #[ReturnTypeWillChange]
    public function rewind() { return null; }
    #[ReturnTypeWillChange]
    public function valid() { return false; }
}
$hook = new WP_Hook();
if (is_a($hook, "Iterator")) {
    echo "unresolved-builtin:iterator\n";
}
if (interface_exists("Iterator")) {
    echo "core-builtin:declared\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "service:logger\nservice:hookable\nchild:inherits-interface\nstring:inherits-interface\nunresolved-builtin:iterator\ncore-builtin:declared\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn interface_required_method_presence_is_enforced_for_concrete_classes() {
    let execution = run_source(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log($message) {
        return "log:" . $message;
    }
}

$service = new Service();
echo $service->log("ok");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "log:ok");
    assert_eq!(execution.exit_code, 0);

    let missing_method_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {}
"#,
    );
    assert_eq!(missing_method_error.line, 6);
    assert_eq!(missing_method_error.column, 1);
    assert_eq!(
        missing_method_error.message,
        "unsupported class inheritance for Service: concrete class Service must implement interface method Logger::log()"
    );

    let non_public_method_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    protected function log($message) {}
}
"#,
    );
    assert_eq!(non_public_method_error.line, 6);
    assert_eq!(non_public_method_error.column, 1);
    assert_eq!(
        non_public_method_error.message,
        "unsupported class inheritance for Service: concrete class Service must implement interface method Logger::log()"
    );
}

#[test]
fn interface_required_method_static_compatibility_is_enforced_for_concrete_classes() {
    let execution = run_source(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log($message) {
        return "log:" . $message;
    }
}

$service = new Service();
echo $service->log("ok");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "log:ok");
    assert_eq!(execution.exit_code, 0);

    let static_method_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public static function log($message) {}
}
"#,
    );
    assert_eq!(static_method_error.line, 6);
    assert_eq!(static_method_error.column, 1);
    assert_eq!(
        static_method_error.message,
        "unsupported class inheritance for Service: concrete class Service must implement interface method Logger::log() as non static method; found static Service::log()"
    );

    let inherited_static_method_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

abstract class Base implements Logger {
    public static function log($message) {}
}

class Child extends Base {}
"#,
    );
    assert_eq!(inherited_static_method_error.line, 10);
    assert_eq!(inherited_static_method_error.column, 1);
    assert_eq!(
        inherited_static_method_error.message,
        "unsupported class inheritance for Child: concrete class Child must implement interface method Logger::log() as non static method; found static Base::log()"
    );
}

#[test]
fn property_override_attribute_validates_parent_interface_and_trait_properties() {
    let parent_property = run_source(
        r#"<?php
class Base {
    protected mixed $value;
}
class Child extends Base {
    #[\Override]
    public mixed $value;
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(parent_property.stdout, "Done");
    assert_eq!(parent_property.exit_code, 0);

    let interface_property = run_source(
        r#"<?php
interface Contract {
    public mixed $value { get; }
}
class Service implements Contract {
    #[\Override]
    public mixed $value;
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(interface_property.stdout, "Done");
    assert_eq!(interface_property.exit_code, 0);

    let trait_property = run_source(
        r#"<?php
trait ProvidesValue {
    #[\Override]
    public mixed $value;
}
interface Contract {
    public mixed $value { get; }
}
class Service implements Contract {
    use ProvidesValue;
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(trait_property.stdout, "Done");
    assert_eq!(trait_property.exit_code, 0);

    assert_php_startup_fatal(
        r#"<?php
class Standalone {
    #[\Override]
    public mixed $value;
}
"#,
        "tests/classes/property_override_missing_parent.php",
        4,
        "Standalone::$value has #[\\Override] attribute, but no matching parent property exists",
    );

    assert_php_startup_fatal(
        r#"<?php
class Base {
    private mixed $value;
}
class Child extends Base {
    #[\Override]
    public mixed $value;
}
"#,
        "tests/classes/property_override_private_parent.php",
        7,
        "Child::$value has #[\\Override] attribute, but no matching parent property exists",
    );

    assert_php_startup_fatal(
        r#"<?php
interface Contract {
    #[\Override]
    public mixed $value { get; }
}
"#,
        "tests/classes/property_override_interface_missing_parent.php",
        4,
        "Contract::$value has #[\\Override] attribute, but no matching parent property exists",
    );

    assert_php_startup_fatal(
        r#"<?php
trait ProvidesValue {
    #[\Override]
    public mixed $value;
}
class Service {
    use ProvidesValue;
}
"#,
        "tests/classes/property_override_trait_missing_parent.php",
        4,
        "Service::$value has #[\\Override] attribute, but no matching parent property exists",
    );
}

#[test]
fn property_override_attribute_validates_promoted_constructor_properties() {
    let interface_property = run_source(
        r#"<?php
interface Contract {
    public mixed $value { get; }
}
class Service implements Contract {
    public function __construct(
        #[\Override]
        public mixed $value,
    ) {}
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(interface_property.stdout, "Done");
    assert_eq!(interface_property.exit_code, 0);

    assert_php_startup_fatal(
        r#"<?php
class Service {
    public function __construct(
        #[\Override]
        public mixed $value,
    ) {}
}
echo "Done";
"#,
        "tests/classes/property_override_promoted_missing_parent.php",
        5,
        "Service::$value has #[\\Override] attribute, but no matching parent property exists",
    );

    let promoted_property = run_source(
        r#"<?php
class Service {
    public function __construct(
        public mixed $value,
    ) {}
}
new Service("value");
"#,
    )
    .unwrap();
    assert_eq!(promoted_property.stdout, "");
    assert_eq!(promoted_property.stderr, "");
    assert_eq!(promoted_property.exit_code, 0);
}

#[test]
fn method_override_attribute_validates_class_interface_trait_and_constructor_methods() {
    let parent_method = run_source(
        r#"<?php
class Base {
    public function run(): void {}
}
class Child extends Base {
    #[\Override]
    public function run(): void {}
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(parent_method.stdout, "Done");
    assert_eq!(parent_method.exit_code, 0);

    let interface_method = run_source(
        r#"<?php
interface Contract {
    public function run(): void;
}
class Service implements Contract {
    #[\Override]
    public function run(): void {}
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(interface_method.stdout, "Done");
    assert_eq!(interface_method.exit_code, 0);

    let trait_method = run_source(
        r#"<?php
trait ProvidesRun {
    #[\Override]
    public function run(): void {}
}
interface Contract {
    public function run(): void;
}
class Service implements Contract {
    use ProvidesRun;
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(trait_method.stdout, "Done");
    assert_eq!(trait_method.exit_code, 0);

    let abstract_constructor = run_source(
        r#"<?php
abstract class Base {
    public abstract function __construct();
}
class Child extends Base {
    #[\Override]
    public function __construct() {}
}
echo "Done";
"#,
    )
    .unwrap();
    assert_eq!(abstract_constructor.stdout, "Done");
    assert_eq!(abstract_constructor.exit_code, 0);

    assert_php_startup_fatal(
        r#"<?php
class Standalone {
    #[\Override]
    public static function c(): void {}
}
"#,
        "tests/classes/method_override_missing_parent.php",
        4,
        "Standalone::c() has #[\\Override] attribute, but no matching parent method exists",
    );

    assert_php_startup_fatal(
        r#"<?php
interface Contract {
    #[\Override]
    public function run(): void;
}
"#,
        "tests/classes/method_override_interface_missing_parent.php",
        4,
        "Contract::run() has #[\\Override] attribute, but no matching parent method exists",
    );

    assert_php_startup_fatal(
        r#"<?php
trait ProvidesRun {
    #[\Override]
    public function run(): void {}
}
class Service {
    use ProvidesRun;
}
"#,
        "tests/classes/method_override_trait_missing_parent.php",
        4,
        "Service::run() has #[\\Override] attribute, but no matching parent method exists",
    );

    assert_php_startup_fatal(
        r#"<?php
class Base {
    public function __construct() {}
}
class Child extends Base {
    #[\Override]
    public function __construct() {}
}
"#,
        "tests/classes/method_override_concrete_parent_constructor.php",
        7,
        "Child::__construct() has #[\\Override] attribute, but no matching parent method exists",
    );
}

#[test]
fn abstract_trait_methods_can_satisfy_method_override_attributes() {
    let execution = run_source(
        r#"<?php
trait RequiresRun {
    public abstract function run(): void;
}
class Service {
    use RequiresRun;

    #[\Override]
    public function run(): void {}
}
echo "abstract override ok";
"#,
    )
    .unwrap();
    assert_eq!(execution.stdout, "abstract override ok");
    assert_eq!(execution.exit_code, 0);

    let inherited_trait_requirement = run_source(
        r#"<?php
trait RequiresRun {
    public abstract function run(): void;
}
trait UsesRequirement {
    use RequiresRun;
}
class Service {
    use UsesRequirement;

    #[\Override]
    public function run(): void {}
}
echo "nested abstract override ok";
"#,
    )
    .unwrap();
    assert_eq!(
        inherited_trait_requirement.stdout,
        "nested abstract override ok"
    );
    assert_eq!(inherited_trait_requirement.exit_code, 0);
}

#[test]
fn static_interface_methods_are_declared_validated_and_callable() {
    let execution = run_source(
        r#"<?php
interface FactoryContract {
    public static function make($name = "core");
}

interface PluginFactory extends FactoryContract {
    public static function boot($hook);
}

class Plugin implements PluginFactory {
    public static function make($name = "core") {
        return "make:" . $name;
    }

    public static function boot($hook) {
        return "boot:" . $hook;
    }
}

echo Plugin::make(), "\n";
echo Plugin::make("wp"), "\n";
echo Plugin::boot("init"), "\n";
echo is_a("Plugin", "FactoryContract", true) ? "factory\n" : "missing\n";
echo is_subclass_of("Plugin", "PluginFactory", true) ? "plugin-factory" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "make:core\nmake:wp\nboot:init\nfactory\nplugin-factory"
    );
    assert_eq!(execution.exit_code, 0);

    let non_static_implementation_error = runtime_error(
        r#"<?php
interface FactoryContract {
    public static function make();
}

class Plugin implements FactoryContract {
    public function make() {}
}
"#,
    );
    assert_eq!(non_static_implementation_error.line, 6);
    assert_eq!(non_static_implementation_error.column, 1);
    assert_eq!(
        non_static_implementation_error.message,
        "unsupported class inheritance for Plugin: concrete class Plugin must implement interface method FactoryContract::make() as static method; found non static Plugin::make()"
    );

    let static_non_static_implementation_error = runtime_error(
        r#"<?php
interface Logger {
    public function log();
}

class Plugin implements Logger {
    public static function log() {}
}
"#,
    );
    assert_eq!(static_non_static_implementation_error.line, 6);
    assert_eq!(static_non_static_implementation_error.column, 1);
    assert_eq!(
        static_non_static_implementation_error.message,
        "unsupported class inheritance for Plugin: concrete class Plugin must implement interface method Logger::log() as non static method; found static Plugin::log()"
    );

    let child_interface_staticness_error = runtime_error(
        r#"<?php
interface FactoryContract {
    public static function make();
}

interface PluginFactory extends FactoryContract {
    public function make();
}
"#,
    );
    assert_eq!(child_interface_staticness_error.line, 6);
    assert_eq!(child_interface_staticness_error.column, 1);
    assert_eq!(
        child_interface_staticness_error.message,
        "unsupported class inheritance for PluginFactory: interface method PluginFactory::make() must keep staticness of parent interface method FactoryContract::make(); expected static, found non static"
    );
}

#[test]
fn interface_required_method_parameter_compatibility_is_enforced_for_concrete_classes() {
    let execution = run_source(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log($message, $context = "default") {
        return "log:" . $message . ":" . $context;
    }
}

$service = new Service();
echo $service->log("ok"), "\n";
echo $service->log("ok", "custom");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "log:ok:default\nlog:ok:custom");
    assert_eq!(execution.exit_code, 0);

    let extra_required_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log($message, $context) {}
}
"#,
    );
    assert_eq!(extra_required_error.line, 6);
    assert_eq!(extra_required_error.column, 1);
    assert_eq!(
        extra_required_error.message,
        "unsupported class inheritance for Service: method Service::log() cannot require more parameters than interface method Logger::log()"
    );

    let optional_interface_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message = "default");
}

class Service implements Logger {
    public function log($message) {}
}
"#,
    );
    assert_eq!(optional_interface_error.line, 6);
    assert_eq!(optional_interface_error.column, 1);
    assert_eq!(
        optional_interface_error.message,
        "unsupported class inheritance for Service: method Service::log() cannot require more parameters than interface method Logger::log()"
    );

    let inherited_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

abstract class Base implements Logger {
    public function log($message, $context) {}
}

class Child extends Base {}
"#,
    );
    assert_eq!(inherited_error.line, 10);
    assert_eq!(inherited_error.column, 1);
    assert_eq!(
        inherited_error.message,
        "unsupported class inheritance for Child: method Base::log() cannot require more parameters than interface method Logger::log()"
    );
}

#[test]
fn interface_required_method_parameter_type_compatibility_is_enforced_for_concrete_classes() {
    let execution = run_source(
        r#"<?php
interface Logger {
    public function log(string $message);
}

class ExactLogger implements Logger {
    public function log(string $message) {}
}

class BroadLogger implements Logger {
    public function log($message) {}
}

echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);

    let added_type_error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log(string $message) {}
}
"#,
    );
    assert_eq!(added_type_error.line, 6);
    assert_eq!(added_type_error.column, 1);
    assert_eq!(
        added_type_error.message,
        "unsupported class inheritance for Service: method Service::log() cannot add parameter type string for parameter $message when interface method Logger::log() has no parameter type"
    );

    let changed_type_error = runtime_error(
        r#"<?php
interface Logger {
    public function log(string $message);
}

class Service implements Logger {
    public function log(int $message) {}
}
"#,
    );
    assert_eq!(changed_type_error.line, 6);
    assert_eq!(changed_type_error.column, 1);
    assert_eq!(
        changed_type_error.message,
        "unsupported class inheritance for Service: method Service::log() parameter $message type int is incompatible with interface method Logger::log() parameter type string"
    );

    let inherited_changed_type_error = runtime_error(
        r#"<?php
interface Logger {
    public function log(string $message);
}

abstract class Base implements Logger {
    public function log(int $message) {}
}

class Child extends Base {}
"#,
    );
    assert_eq!(inherited_changed_type_error.line, 10);
    assert_eq!(inherited_changed_type_error.column, 1);
    assert_eq!(
        inherited_changed_type_error.message,
        "unsupported class inheritance for Child: method Base::log() parameter $message type int is incompatible with interface method Logger::log() parameter type string"
    );
}

#[test]
fn interface_required_method_return_type_compatibility_is_enforced_for_concrete_classes() {
    let execution = run_source(
        r#"<?php
interface Provider {
    public function label(): string;
}

interface UntypedProvider {
    public function id();
}

class ExactProvider implements Provider {
    public function label(): string {
        return "label";
    }
}

class AddingProvider implements UntypedProvider {
    public function id(): string {
        return "id";
    }
}

echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);

    let omitted_return_type_error = runtime_error(
        r#"<?php
interface Provider {
    public function label(): string;
}

class Service implements Provider {
    public function label() {}
}
"#,
    );
    assert_eq!(omitted_return_type_error.line, 6);
    assert_eq!(omitted_return_type_error.column, 1);
    assert_eq!(
        omitted_return_type_error.message,
        "unsupported class inheritance for Service: method Service::label() must declare return type string to match interface method Provider::label()"
    );

    let changed_return_type_error = runtime_error(
        r#"<?php
interface Provider {
    public function label(): string;
}

class Service implements Provider {
    public function label(): int {}
}
"#,
    );
    assert_eq!(changed_return_type_error.line, 6);
    assert_eq!(changed_return_type_error.column, 1);
    assert_eq!(
        changed_return_type_error.message,
        "unsupported class inheritance for Service: method Service::label() return type int is incompatible with interface method Provider::label() return type string"
    );

    let inherited_changed_return_type_error = runtime_error(
        r#"<?php
interface Provider {
    public function label(): string;
}

abstract class Base implements Provider {
    public function label(): int {}
}

class Child extends Base {}
"#,
    );
    assert_eq!(inherited_changed_return_type_error.line, 10);
    assert_eq!(inherited_changed_return_type_error.column, 1);
    assert_eq!(
        inherited_changed_return_type_error.message,
        "unsupported class inheritance for Child: method Base::label() return type int is incompatible with interface method Provider::label() return type string"
    );
}

#[test]
fn inherited_interface_required_method_presence_is_enforced_for_concrete_classes() {
    let execution = run_source(
        r#"<?php
interface Logger {
    public function log($message);
}

abstract class Base implements Logger {}

class Child extends Base {
    public function log($message) {
        return "child:" . $message;
    }
}

$child = new Child();
echo $child->log("ok");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child:ok");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
interface Logger {
    public function log($message);
}

abstract class Base implements Logger {}
class Child extends Base {}
"#,
    );
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: concrete class Child must implement interface method Logger::log()"
    );
}

#[test]
fn interface_inheritance_required_methods_and_relationships_are_enforced() {
    let execution = run_source(
        r#"<?php
interface Hookable {
    public function register_hooks();
}

interface PluginContract extends Hookable {
    public function label();
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

trait HasLabel {
    public function label() {
        return "label:" . get_class($this);
    }
}

class Plugin implements PluginContract {
    use HasHooks, HasLabel {
        HasHooks::hooks as public register_hooks;
    }
}

$plugin = new Plugin();
echo $plugin instanceof Hookable ? "instanceof-parent\n" : "missing-parent\n";
echo is_a($plugin, "Hookable") ? "is-a-parent\n" : "missing-is-a\n";
echo is_subclass_of($plugin, "Hookable") ? "subclass-parent\n" : "missing-subclass\n";
echo $plugin->register_hooks(), "\n";
echo $plugin->label();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "instanceof-parent\nis-a-parent\nsubclass-parent\nhooks:Plugin\nlabel:Plugin"
    );
    assert_eq!(execution.exit_code, 0);

    let missing_parent_method = runtime_error(
        r#"<?php
interface Hookable {
    public function register_hooks();
}

interface PluginContract extends Hookable {
    public function label();
}

class Plugin implements PluginContract {
    public function label() {}
}
"#,
    );
    assert_eq!(missing_parent_method.line, 10);
    assert_eq!(missing_parent_method.column, 1);
    assert_eq!(
        missing_parent_method.message,
        "unsupported class inheritance for Plugin: concrete class Plugin must implement interface method Hookable::register_hooks()"
    );

    let missing_parent_interface = runtime_error(
        r#"<?php
interface PluginContract extends Hookable {}
"#,
    );
    assert_eq!(missing_parent_interface.line, 2);
    assert_eq!(missing_parent_interface.column, 1);
    assert_eq!(
        missing_parent_interface.message,
        "unsupported class inheritance for PluginContract: interface PluginContract extends missing or unsupported parent interface Hookable"
    );
}

#[test]
fn forward_parent_interface_resolution_enforces_methods_relationships_and_constants() {
    let execution = run_source(
        r#"<?php
interface PluginContract extends Hookable {
    const CHILD = "contract";
    public function boot();
}

trait HasHooks {
    public function hook_impl() {
        return "hook:" . get_class($this);
    }
}

class Plugin implements PluginContract {
    use HasHooks {
        hook_impl as public register_hooks;
    }

    public function boot() {
        return self::PARENT . ":" . static::CHILD;
    }
}

interface Hookable {
    const PARENT = "base";
    public function register_hooks();
}

$plugin = new Plugin();
echo PluginContract::PARENT, "\n";
echo Plugin::PARENT, "\n";
echo $plugin instanceof Hookable ? "instanceof-base\n" : "missing-instanceof\n";
echo is_a("Plugin", "Hookable", true) ? "is-a-base\n" : "missing-is-a\n";
echo is_subclass_of("Plugin", "Hookable", true) ? "subclass-base\n" : "missing-subclass\n";
echo $plugin->boot(), "\n";
echo $plugin->register_hooks();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base\nbase\ninstanceof-base\nis-a-base\nsubclass-base\nbase:contract\nhook:Plugin"
    );
    assert_eq!(execution.exit_code, 0);

    let cycle = runtime_error(
        r#"<?php
interface First extends Second {}
interface Second extends First {}
"#,
    );
    assert_eq!(cycle.line, 2);
    assert_eq!(cycle.column, 1);
    assert_eq!(
        cycle.message,
        "unsupported class inheritance for First: interface First has cyclic parent interface inheritance"
    );
}

#[test]
fn multiple_interface_inheritance_flattens_required_methods_and_relationships() {
    let execution = run_source(
        r#"<?php
interface Hookable {
    public function register_hooks();
}

interface Labelable {
    public function label();
}

interface PluginContract extends Hookable, Labelable {
    public function boot();
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

trait HasLabel {
    public function label() {
        return "label:" . get_class($this);
    }
}

class Plugin implements PluginContract {
    use HasHooks, HasLabel {
        HasHooks::hooks as public register_hooks;
    }

    public function boot() {
        return "boot:" . get_class($this);
    }
}

$plugin = new Plugin();
echo $plugin instanceof Hookable ? "instanceof-hookable\n" : "missing-hookable\n";
echo $plugin instanceof Labelable ? "instanceof-labelable\n" : "missing-labelable\n";
echo is_a($plugin, "Hookable") ? "is-a-hookable\n" : "missing-is-a-hookable\n";
echo is_a($plugin, "Labelable") ? "is-a-labelable\n" : "missing-is-a-labelable\n";
echo is_subclass_of($plugin, "Hookable") ? "subclass-hookable\n" : "missing-subclass-hookable\n";
echo is_subclass_of($plugin, "Labelable") ? "subclass-labelable\n" : "missing-subclass-labelable\n";
echo $plugin->register_hooks(), "\n";
echo $plugin->label(), "\n";
echo $plugin->boot();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "instanceof-hookable\ninstanceof-labelable\nis-a-hookable\nis-a-labelable\nsubclass-hookable\nsubclass-labelable\nhooks:Plugin\nlabel:Plugin\nboot:Plugin"
    );
    assert_eq!(execution.exit_code, 0);

    let missing_parent_method = runtime_error(
        r#"<?php
interface Hookable {
    public function register_hooks();
}

interface Labelable {
    public function label();
}

interface PluginContract extends Hookable, Labelable {
    public function boot();
}

class Plugin implements PluginContract {
    public function register_hooks() {}
    public function boot() {}
}
"#,
    );
    assert_eq!(missing_parent_method.line, 14);
    assert_eq!(missing_parent_method.column, 1);
    assert_eq!(
        missing_parent_method.message,
        "unsupported class inheritance for Plugin: concrete class Plugin must implement interface method Labelable::label()"
    );
}

#[test]
fn interface_inheritance_method_signature_compatibility_is_enforced() {
    let execution = run_source(
        r#"<?php
interface BaseHook {
    public function dispatch(string $hook): string;
    public function summarize($context);
    public function optional($value);
}

interface PluginHook extends BaseHook {
    public function dispatch($hook, $priority = 10): string;
    public function summarize($context): string;
    public function optional($value, $fallback = null);
}

trait HookMethods {
    public function dispatch($hook, $priority = 10): string {
        return $hook . ":" . $priority;
    }

    public function summarize($context): string {
        return "summary:" . $context;
    }

    public function optional($value, $fallback = null) {
        return $value . ":" . $fallback;
    }
}

class Plugin implements PluginHook {
    use HookMethods;
}

$plugin = new Plugin();
echo $plugin instanceof BaseHook ? "base\n" : "missing\n";
echo method_exists($plugin, "dispatch") ? "dispatch-method\n" : "missing\n";
echo method_exists($plugin, "summarize") ? "summary-method\n" : "missing\n";
echo $plugin->optional("value", "fallback");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base\ndispatch-method\nsummary-method\nvalue:fallback"
    );
    assert_eq!(execution.exit_code, 0);

    let extra_required = runtime_error(
        r#"<?php
interface BaseHook {
    public function dispatch($hook);
}

interface PluginHook extends BaseHook {
    public function dispatch($hook, $priority);
}
"#,
    );
    assert_eq!(extra_required.line, 6);
    assert_eq!(extra_required.column, 1);
    assert_eq!(
        extra_required.message,
        "unsupported class inheritance for PluginHook: interface method PluginHook::dispatch() cannot require more parameters than parent interface method BaseHook::dispatch()"
    );

    let added_parameter_type = runtime_error(
        r#"<?php
interface BaseHook {
    public function dispatch($hook);
}

interface PluginHook extends BaseHook {
    public function dispatch(string $hook);
}
"#,
    );
    assert_eq!(added_parameter_type.line, 6);
    assert_eq!(added_parameter_type.column, 1);
    assert_eq!(
        added_parameter_type.message,
        "unsupported class inheritance for PluginHook: interface method PluginHook::dispatch() cannot add parameter type string for parameter $hook when parent interface method BaseHook::dispatch() has no parameter type"
    );

    let missing_return_type = runtime_error(
        r#"<?php
interface BaseHook {
    public function dispatch(): string;
}

interface PluginHook extends BaseHook {
    public function dispatch();
}
"#,
    );
    assert_eq!(missing_return_type.line, 6);
    assert_eq!(missing_return_type.column, 1);
    assert_eq!(
        missing_return_type.message,
        "unsupported class inheritance for PluginHook: interface method PluginHook::dispatch() must declare return type string to match parent interface method BaseHook::dispatch()"
    );

    let parent_conflict = runtime_error(
        r#"<?php
interface FirstHook {
    public function dispatch($hook);
}

interface SecondHook {
    public function dispatch($hook, $priority);
}

interface PluginHook extends FirstHook, SecondHook {}
"#,
    );
    assert_eq!(parent_conflict.line, 10);
    assert_eq!(parent_conflict.column, 1);
    assert_eq!(
        parent_conflict.message,
        "unsupported class inheritance for PluginHook: interface method SecondHook::dispatch() cannot require more parameters than parent interface method FirstHook::dispatch()"
    );
}

#[test]
fn interface_constants_resolve_through_interfaces_and_implementing_classes() {
    let source = r#"<?php
interface HookDefaults {
    public const ACTION = "init";
    const PRIORITY = 10;
}

interface ChildDefaults extends HookDefaults {
    const GROUP = "plugins";
}

class Plugin implements ChildDefaults {
    public static function summary() {
        return self::ACTION . ":" . static::GROUP . ":" . static::PRIORITY;
    }
}

class OverridePlugin extends Plugin {
    public const PRIORITY = 20;
}

echo HookDefaults::ACTION, "\n";
echo ChildDefaults::ACTION, "\n";
echo Plugin::ACTION, "\n";
echo Plugin::GROUP, "\n";
echo Plugin::summary(), "\n";
echo OverridePlugin::summary(), "\n";
echo defined("ChildDefaults::PRIORITY") ? "defined\n" : "missing\n";
echo constant("Plugin::ACTION"), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "init\ninit\ninit\nplugins\ninit:plugins:10\ninit:plugins:20\ndefined\ninit\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn interface_constant_boundaries_are_stable() {
    let typed = parse_error(
        r#"<?php
interface Logger {
    public const string NAME = "logger";
}
"#,
    );
    assert_eq!(typed.line, 3);
    assert_eq!(typed.column, 18);
    assert_eq!(
        typed.message,
        "unsupported interface constant declaration: typed interface constants are not implemented"
    );

    let non_public = parse_error(
        r#"<?php
interface Logger {
    protected const NAME = "logger";
}
"#,
    );
    assert_eq!(non_public.line, 3);
    assert_eq!(non_public.column, 15);
    assert_eq!(
        non_public.message,
        "unsupported interface constant declaration: only public interface constants are implemented"
    );

    let duplicate = runtime_error(
        r#"<?php
interface Logger {
    const NAME = "logger";
    public const NAME = "duplicate";
}
"#,
    );
    assert_eq!(duplicate.line, 4);
    assert_eq!(duplicate.column, 18);
    assert_eq!(
        duplicate.message,
        "class Logger already defines constant NAME"
    );

    let ambiguous = runtime_error(
        r#"<?php
interface Primary {
    const NAME = "primary";
}
interface Secondary {
    const NAME = "secondary";
}
class Plugin implements Primary, Secondary {}
echo Plugin::NAME;
"#,
    );
    assert_eq!(
        ambiguous.message,
        "unsupported call Plugin::NAME: interface constant resolution is ambiguous between Primary::NAME, Secondary::NAME"
    );

    let ambiguous_defined = runtime_error(
        r#"<?php
interface Primary {
    const NAME = "primary";
}
interface Secondary {
    const NAME = "secondary";
}
class Plugin implements Primary, Secondary {}
var_dump(defined("Plugin::NAME"));
"#,
    );
    assert_eq!(
        ambiguous_defined.message,
        "unsupported call Plugin::NAME: interface constant resolution is ambiguous between Primary::NAME, Secondary::NAME"
    );
}

#[test]
fn core_interface_catalog_reports_bounded_internal_interfaces() {
    let execution = run_source(
        r#"<?php
foreach (array("Traversable", "IteratorAggregate", "Iterator", "Serializable", "ArrayAccess", "Countable", "Stringable", "DateTimeInterface") as $name) {
    echo interface_exists($name) ? $name . ":yes\n" : $name . ":no\n";
}

echo interface_exists("DefinitelyMissingInterface") ? "missing:yes" : "missing:no";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Traversable:yes\nIteratorAggregate:yes\nIterator:yes\nSerializable:yes\nArrayAccess:yes\nCountable:yes\nStringable:yes\nDateTimeInterface:yes\nmissing:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn interface_exists_requires_string_name_and_bool_autoload_arguments() {
    let scalar_name = run_source("<?php\nvar_dump(interface_exists(42));\n").unwrap();
    assert_eq!(scalar_name.stdout, "bool(false)\n");
    assert_eq!(scalar_name.exit_code, 0);

    let name_error = runtime_error("<?php\nvar_dump(interface_exists([]));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call interface_exists(): class-like name argument must be string-compatible scalar in the current subset, got array"
    );

    let autoload_error = runtime_error("<?php\nvar_dump(interface_exists(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call interface_exists(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn trait_exists_reports_declared_trait_metadata() {
    let source = r#"<?php
namespace App;

class Box {}
trait Logger {}
trait Hookable {}

if (!trait_exists("App\\Box")) {
    echo "class:not-trait\n";
}
if (trait_exists("App\\Logger")) {
    echo "logger:trait\n";
}
if (trait_exists("App\\Hookable")) {
    echo "hookable:trait\n";
}
if (!trait_exists("App\\Missing", false)) {
    echo "missing:false-autoload\n";
}
$call = "trait_exists";
if ($call("APP\\LOGGER", true)) {
    echo "dynamic:trait\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "class:not-trait\nlogger:trait\nhookable:trait\nmissing:false-autoload\ndynamic:trait\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn class_trait_use_composes_public_instance_methods() {
    let source = r#"<?php
trait Logger {
    public function label($value = "default") {
        return "trait:" . $value;
    }

    function implicitPublic() {
        return get_class($this);
    }
}

class Widget {
    use Logger;
}

$widget = new Widget();
echo $widget->label("ok"), "\n";
echo $widget->label(), "\n";
echo $widget->implicitPublic(), "\n";

$methods = get_class_methods($widget);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "trait:ok\ntrait:default\nWidget\nArray\n(\n    [0] => label\n    [1] => implicitPublic\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Widget").unwrap();
    assert!(class.method("label").is_some());
    assert!(class.method("implicitpublic").is_some());
}

#[test]
fn class_trait_use_composes_multiple_public_instance_traits_from_one_declaration() {
    let source = r#"<?php
interface Bootable {
    public function boot();
}

trait HasBoot {
    public function boot() {
        return "boot:" . get_class($this);
    }
}

trait HasLabel {
    public function label($value = "default") {
        return "label:" . $value;
    }
}

class Plugin implements Bootable {
    use HasBoot, HasLabel;
}

$plugin = new Plugin();
echo $plugin->boot(), "\n";
echo $plugin->label("ok"), "\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "boot:Plugin\nlabel:ok\nArray\n(\n    [0] => boot\n    [1] => label\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.method("boot").is_some());
    assert!(class.method("label").is_some());
}

#[test]
fn class_trait_use_composes_public_trait_constants() {
    let source = r#"<?php
trait HookDefaults {
    public const ACTION = "init";
    const PRIORITY = 10;

    public function hookKey() {
        return self::ACTION . ":" . static::PRIORITY;
    }
}

class Plugin {
    use HookDefaults;

    public static function action() {
        return self::ACTION;
    }
}

class ChildPlugin extends Plugin {
    public const PRIORITY = 20;
}

echo Plugin::ACTION, "\n";
echo Plugin::PRIORITY, "\n";
echo Plugin::action(), "\n";

$plugin = new Plugin();
echo $plugin->hookKey(), "\n";

$child = new ChildPlugin();
echo $child->hookKey(), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "init\n10\ninit\ninit:10\ninit:20\n");
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    let action = class.constant("ACTION").unwrap();
    assert_eq!(action.visibility(), Visibility::Public);
    let priority = class.constant("PRIORITY").unwrap();
    assert_eq!(priority.visibility(), Visibility::Public);
}

#[test]
fn class_trait_use_rejects_conflicting_trait_constants() {
    let error = runtime_error(
        r#"<?php
trait PrimaryConfig {
    public const OPTION = "primary";
}

trait FallbackConfig {
    public const OPTION = "fallback";
}

class Plugin {
    use PrimaryConfig, FallbackConfig;
}
"#,
    );

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(
        error.message,
        "class Plugin already defines constant OPTION"
    );

    let class_override = runtime_error(
        r#"<?php
trait PrimaryConfig {
    public const OPTION = "primary";
}

class Plugin {
    use PrimaryConfig;
    public const OPTION = "class";
}
"#,
    );

    assert_eq!(
        class_override.message,
        "class Plugin already defines constant OPTION"
    );
}

#[test]
fn class_trait_use_alias_adaptation_composes_public_instance_method_aliases() {
    let source = r#"<?php
interface Registrable {
    public function register_hooks();
}

trait HasHooks {
    public function hooks($suffix = "default") {
        return "hooks:" . $suffix . ":" . get_class($this);
    }
}

class Plugin implements Registrable {
    use HasHooks {
        hooks as register_hooks;
    }
}

$plugin = new Plugin();
echo $plugin->hooks("direct"), "\n";
echo $plugin->register_hooks("alias"), "\n";
echo method_exists($plugin, "register_hooks") ? "alias-method\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "hooks:direct:Plugin\nhooks:alias:Plugin\nalias-method\nArray\n(\n    [0] => register_hooks\n    [1] => hooks\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.method("hooks").is_some());
    assert!(class.method("register_hooks").is_some());
}

#[test]
fn class_trait_use_alias_adaptation_accepts_explicit_public_aliases() {
    let source = r#"<?php
interface Registrable {
    public function register_hooks();
}

trait HasHooks {
    public function hooks($suffix = "default") {
        return "hooks:" . $suffix . ":" . get_class($this);
    }
}

trait HasLabel {
    public function label() {
        return "label:" . get_class($this);
    }
}

class Plugin implements Registrable {
    use HasHooks, HasLabel {
        HasHooks::hooks as public register_hooks;
    }
}

$plugin = new Plugin();
echo $plugin->hooks("direct"), "\n";
echo $plugin->register_hooks("alias"), "\n";
echo $plugin->label(), "\n";
echo method_exists($plugin, "register_hooks") ? "alias-method\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "hooks:direct:Plugin\nhooks:alias:Plugin\nlabel:Plugin\nalias-method\nArray\n(\n    [0] => register_hooks\n    [1] => hooks\n    [2] => label\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.method("hooks").is_some());
    assert!(class.method("register_hooks").is_some());
    assert!(class.method("label").is_some());
}

#[test]
fn class_trait_use_static_aliases_preserve_static_method_metadata() {
    let source = r#"<?php
trait StaticHookTools {
    public static function boot($suffix = "direct") {
        return "boot:" . $suffix;
    }
}

class StaticPlugin {
    use StaticHookTools {
        boot as boot_alias;
        boot as protected hidden_boot;
    }

    public static function callHidden($suffix) {
        return self::hidden_boot($suffix);
    }
}

echo StaticPlugin::boot("direct"), "\n";
echo StaticPlugin::boot_alias("alias"), "\n";
echo StaticPlugin::callHidden("hidden"), "\n";
echo is_callable(array("StaticPlugin", "boot")) ? "boot-callable\n" : "boot-missing\n";
echo is_callable(array("StaticPlugin", "boot_alias")) ? "alias-callable\n" : "alias-missing\n";
echo is_callable(array("StaticPlugin", "hidden_boot")) ? "bad-hidden\n" : "hidden-filtered\n";

$methods = get_class_methods("StaticPlugin");
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "boot:direct\nboot:alias\nboot:hidden\nboot-callable\nalias-callable\nhidden-filtered\nArray\n(\n    [0] => boot_alias\n    [1] => boot\n    [2] => callHidden\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("StaticPlugin").unwrap();
    let boot = class.method("boot").unwrap();
    assert_eq!(boot.visibility(), Visibility::Public);
    assert!(boot.is_static());

    let alias = class.method("boot_alias").unwrap();
    assert_eq!(alias.visibility(), Visibility::Public);
    assert!(alias.is_static());

    let hidden = class.method("hidden_boot").unwrap();
    assert_eq!(hidden.visibility(), Visibility::Protected);
    assert!(hidden.is_static());
}

#[test]
fn class_trait_use_insteadof_selects_public_instance_method_conflict_winner() {
    let source = r#"<?php
interface NamedPlugin {
    public function label();
}

trait PrimaryLabel {
    public function label() {
        return "primary:" . get_class($this);
    }
}

trait FallbackLabel {
    public function label() {
        return "fallback:" . get_class($this);
    }
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

class Plugin implements NamedPlugin {
    use PrimaryLabel, FallbackLabel, HasHooks {
        PrimaryLabel::label insteadof FallbackLabel;
    }
}

$plugin = new Plugin();
echo $plugin->label(), "\n";
echo $plugin->hooks(), "\n";
echo method_exists($plugin, "label") ? "label-method\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "primary:Plugin\nhooks:Plugin\nlabel-method\nArray\n(\n    [0] => label\n    [1] => hooks\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.method("label").is_some());
    assert!(class.method("hooks").is_some());
}

#[test]
fn class_trait_use_reports_unresolved_public_method_conflicts() {
    let error = runtime_error(
        r#"<?php
trait PrimaryLabel {
    public function label() {
        return "primary";
    }
}

trait FallbackLabel {
    public function label() {
        return "fallback";
    }
}

class Plugin {
    use PrimaryLabel, FallbackLabel;
}
"#,
    );

    assert_eq!(error.line, 9);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported trait use: trait method FallbackLabel::label conflicts with PrimaryLabel::label; add an insteadof adaptation or class override"
    );
}

#[test]
fn class_trait_use_insteadof_selects_winner_over_multiple_losers() {
    let source = r#"<?php
interface NamedPlugin {
    public function label();
    public function hooks();
}

trait PrimaryLabel {
    public function label() {
        return "primary:" . get_class($this);
    }
}

trait FallbackLabel {
    public function label() {
        return "fallback:" . get_class($this);
    }
}

trait LegacyLabel {
    public function label() {
        return "legacy:" . get_class($this);
    }
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

class Plugin implements NamedPlugin {
    use PrimaryLabel, FallbackLabel, LegacyLabel, HasHooks {
        PrimaryLabel::label insteadof FallbackLabel, LegacyLabel;
    }
}

$plugin = new Plugin();
echo $plugin->label(), "\n";
echo $plugin->hooks(), "\n";
echo method_exists($plugin, "label") ? "label-method\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "primary:Plugin\nhooks:Plugin\nlabel-method\nArray\n(\n    [0] => label\n    [1] => hooks\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.method("label").is_some());
    assert!(class.method("hooks").is_some());
    assert_eq!(class.methods().len(), 2);
}

#[test]
fn class_methods_override_trait_methods_and_satisfy_interfaces() {
    let source = r#"<?php
interface HookContract {
    public function label($prefix);
    public function boot();
}

trait DefaultHooks {
    public function label($prefix, $fallback = null) {
        return $prefix . ":trait";
    }

    public function boot() {
        return "trait-boot";
    }
}

trait FallbackHooks {
    public function boot() {
        return "fallback-boot";
    }
}

class Plugin implements HookContract {
    use DefaultHooks, FallbackHooks;

    public function label($prefix) {
        return $prefix . ":class";
    }

    public function boot() {
        return "class-boot";
    }
}

$plugin = new Plugin();
echo $plugin->label("wp"), "\n";
echo $plugin->boot(), "\n";
echo method_exists($plugin, "label") ? "label-method\n" : "missing\n";
echo method_exists($plugin, "boot") ? "boot-method\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "wp:class\nclass-boot\nlabel-method\nboot-method\nArray\n(\n    [0] => label\n    [1] => boot\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert_eq!(class.methods().len(), 2);
    assert!(class.method("label").is_some());
    assert!(class.method("boot").is_some());
}

#[test]
fn trait_alias_visibility_adaptations_are_callable_from_class_context() {
    let source = r#"<?php
trait HookTools {
    public function boot() {
        return "boot:" . get_class($this);
    }

    public function secret() {
        return "secret:" . get_class($this);
    }
}

class Plugin {
    use HookTools {
        boot as protected protected_boot;
        secret as private private_secret;
    }

    public function callProtected() {
        return $this->protected_boot();
    }

    public function callPrivate() {
        return $this->private_secret();
    }
}

$plugin = new Plugin();
echo $plugin->boot(), "\n";
echo $plugin->secret(), "\n";
echo $plugin->callProtected(), "\n";
echo $plugin->callPrivate(), "\n";
echo method_exists($plugin, "protected_boot") ? "protected-exists\n" : "missing\n";
echo method_exists($plugin, "private_secret") ? "private-exists\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "boot:Plugin\nsecret:Plugin\nboot:Plugin\nsecret:Plugin\nprotected-exists\nprivate-exists\nArray\n(\n    [0] => boot\n    [1] => secret\n    [2] => callProtected\n    [3] => callPrivate\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert_eq!(
        class.method("protected_boot").unwrap().visibility(),
        Visibility::Protected
    );
    assert_eq!(
        class.method("private_secret").unwrap().visibility(),
        Visibility::Private
    );
}

#[test]
fn trait_visibility_only_adaptations_change_original_method_visibility() {
    let source = r#"<?php
trait HookTools {
    public function boot() {
        return "boot:" . get_class($this);
    }

    public function secret() {
        return "secret:" . get_class($this);
    }
}

class Plugin {
    use HookTools {
        boot as protected;
        secret as private;
    }

    public function callBoot() {
        return $this->boot();
    }

    public function callSecret() {
        return $this->secret();
    }
}

$plugin = new Plugin();
echo $plugin->callBoot(), "\n";
echo $plugin->callSecret(), "\n";
echo method_exists($plugin, "boot") ? "boot-exists\n" : "missing\n";
echo method_exists($plugin, "secret") ? "secret-exists\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "boot:Plugin\nsecret:Plugin\nboot-exists\nsecret-exists\nArray\n(\n    [0] => callBoot\n    [1] => callSecret\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert_eq!(
        class.method("boot").unwrap().visibility(),
        Visibility::Protected
    );
    assert_eq!(
        class.method("secret").unwrap().visibility(),
        Visibility::Private
    );
}

#[test]
fn unqualified_multi_trait_alias_and_visibility_adaptations_resolve_unique_methods() {
    let source = r#"<?php
trait HookTools {
    public function boot() {
        return "boot:" . get_class($this);
    }
}

trait LabelTools {
    public function label() {
        return "label:" . get_class($this);
    }
}

class Plugin {
    use HookTools, LabelTools {
        boot as protected;
        label as private hiddenLabel;
    }

    public function callBoot() {
        return $this->boot();
    }

    public function callHidden() {
        return $this->hiddenLabel();
    }
}

$plugin = new Plugin();
echo $plugin->label(), "\n";
echo $plugin->callBoot(), "\n";
echo $plugin->callHidden(), "\n";
echo method_exists($plugin, "boot") ? "boot-exists\n" : "missing\n";
echo method_exists($plugin, "hiddenLabel") ? "hidden-exists\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "label:Plugin\nboot:Plugin\nlabel:Plugin\nboot-exists\nhidden-exists\nArray\n(\n    [0] => label\n    [1] => callBoot\n    [2] => callHidden\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert_eq!(
        class.method("boot").unwrap().visibility(),
        Visibility::Protected
    );
    assert_eq!(
        class.method("hiddenLabel").unwrap().visibility(),
        Visibility::Private
    );
    assert_eq!(
        class.method("label").unwrap().visibility(),
        Visibility::Public
    );
}

#[test]
fn unqualified_multi_trait_alias_adaptation_reports_ambiguous_methods() {
    let error = runtime_error(
        r#"<?php
trait FirstLabel { public function label() {} }
trait SecondLabel { public function label() {} }
class Plugin {
    use FirstLabel, SecondLabel {
        label as labelAlias;
        FirstLabel::label insteadof SecondLabel;
    }
}
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 9);
    assert!(
        error
            .message
            .contains("unqualified trait alias label is ambiguous"),
        "{error:?}"
    );
}

#[test]
fn trait_visibility_only_adaptation_requires_existing_trait_method() {
    let error = runtime_error(
        r#"<?php
trait HasHooks {
    public function hooks() {}
}

class Plugin {
    use HasHooks {
        missing as private;
    }
}
"#,
    );

    assert_eq!(error.line, 8);
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "unsupported trait use: trait visibility adaptation HasHooks::missing targets a missing method"
    );
}

#[test]
fn class_trait_use_insteadof_winner_can_be_public_aliased() {
    let source = r#"<?php
interface NamedPlugin {
    public function label();
    public function label_alias();
}

trait PrimaryLabel {
    public function label() {
        return "primary:" . get_class($this);
    }
}

trait FallbackLabel {
    public function label() {
        return "fallback:" . get_class($this);
    }
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

class Plugin implements NamedPlugin {
    use PrimaryLabel, FallbackLabel, HasHooks {
        PrimaryLabel::label insteadof FallbackLabel;
        PrimaryLabel::label as public label_alias;
    }
}

$plugin = new Plugin();
echo $plugin->label(), "\n";
echo $plugin->label_alias(), "\n";
echo $plugin->hooks(), "\n";
echo method_exists($plugin, "label_alias") ? "alias-method\n" : "missing\n";

$methods = get_class_methods($plugin);
print_r($methods);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "primary:Plugin\nprimary:Plugin\nhooks:Plugin\nalias-method\nArray\n(\n    [0] => label_alias\n    [1] => label\n    [2] => hooks\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.method("label").is_some());
    assert!(class.method("label_alias").is_some());
    assert!(class.method("hooks").is_some());
}

#[test]
fn class_trait_use_composes_compatible_public_instance_properties() {
    let source = r#"<?php
trait HasOptions {
    public $options = array("autoload" => "yes");
}

trait HasSameOptions {
    public $options = array("autoload" => "yes");
}

class Plugin {
    use HasOptions, HasSameOptions;
}

$plugin = new Plugin();
echo $plugin->options["autoload"], "\n";
$plugin->options["autoload"] = "no";
echo $plugin->options["autoload"], "\n";

$class = new ReflectionClass("Plugin");
echo $class->hasProperty("options") ? "has-options\n" : "missing\n";
$property = $class->getProperty("options");
echo $property->getDeclaringClass()->getName(), "\n";
echo $property->hasDefaultValue() ? "default\n" : "no-default\n";
print_r($property->getDefaultValue());
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "yes\nno\nhas-options\nPlugin\ndefault\nArray\n(\n    [autoload] => yes\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let classes = class_metadata_source(source).unwrap();
    let class = classes.lookup_class("Plugin").unwrap();
    assert!(class.property("options").is_some());
}

#[test]
fn class_trait_use_rejects_incompatible_public_instance_properties() {
    let error = runtime_error(
        r#"<?php
trait PrimaryOptions {
    public $options = array("autoload" => "yes");
}

trait FallbackOptions {
    public $options = array("autoload" => "no");
}

class Plugin {
    use PrimaryOptions, FallbackOptions;
}
"#,
    );

    assert_eq!(error.line, 10);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "PrimaryOptions and FallbackOptions define the same property ($options) in the composition of Plugin. However, the definition differs and is considered incompatible. Class was composed"
    );
}

#[test]
fn class_trait_use_alias_requires_existing_trait_method() {
    let error = runtime_error(
        r#"<?php
trait HasHooks {
    public function hooks() {}
}

class Plugin {
    use HasHooks {
        missing as register_hooks;
    }
}
"#,
    );

    assert_eq!(error.line, 8);
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "unsupported trait use: trait alias HasHooks::missing targets a missing method"
    );
}

#[test]
fn class_trait_use_requires_already_declared_trait() {
    let error = runtime_error(
        r#"<?php
class Widget {
    use MissingTrait;
}
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 5);
    assert_eq!(error.message, "undefined class MissingTrait");
}

#[test]
fn trait_exists_requires_string_name_and_bool_autoload_arguments() {
    let scalar_name = run_source("<?php\nvar_dump(trait_exists(42));\n").unwrap();
    assert_eq!(scalar_name.stdout, "bool(false)\n");
    assert_eq!(scalar_name.exit_code, 0);

    let name_error = runtime_error("<?php\nvar_dump(trait_exists([]));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call trait_exists(): class-like name argument must be string-compatible scalar in the current subset, got array"
    );

    let autoload_error = runtime_error("<?php\nvar_dump(trait_exists(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call trait_exists(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn enum_exists_reports_declared_enum_metadata() {
    let source = r#"<?php
namespace App;

class Box {}
enum Mode { case Front; }
enum Status {}

if (!enum_exists("App\\Box")) {
    echo "class:not-enum\n";
}
if (enum_exists("App\\Mode")) {
    echo "mode:enum\n";
}
if (class_exists("App\\Mode")) {
    echo "mode:class-like\n";
}
if (!interface_exists("App\\Mode") && !trait_exists("App\\Mode")) {
    echo "mode:not-interface-trait\n";
}
if (!enum_exists("App\\Missing", false)) {
    echo "missing:false-autoload\n";
}
$call = "enum_exists";
if ($call("APP\\STATUS", true)) {
    echo "dynamic:enum\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "class:not-enum\nmode:enum\nmode:class-like\nmode:not-interface-trait\nmissing:false-autoload\ndynamic:enum\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn enum_exists_requires_string_name_and_bool_autoload_arguments() {
    let scalar_name = run_source("<?php\nvar_dump(enum_exists(42));\n").unwrap();
    assert_eq!(scalar_name.stdout, "bool(false)\n");
    assert_eq!(scalar_name.exit_code, 0);

    let name_error = runtime_error("<?php\nvar_dump(enum_exists([]));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call enum_exists(): class-like name argument must be string-compatible scalar in the current subset, got array"
    );

    let autoload_error = runtime_error("<?php\nvar_dump(enum_exists(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call enum_exists(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn property_exists_checks_declared_property_metadata() {
    let source = r#"<?php
class Base {
    public $baseName;
    protected $baseSecret;
    private $baseToken;
    public static $baseShared;
}

class Box extends Base {
    public $name;
    protected $secret;
    private static $cache;
}

$box = new box();
if (property_exists($box, "baseName")) {
    echo "object:baseName\n";
}
if (property_exists($box, "baseSecret")) {
    echo "object:baseSecret\n";
}
if (!property_exists($box, "baseToken")) {
    echo "object:baseToken-private-false\n";
}
if (property_exists("Box", "baseShared")) {
    echo "class:baseShared\n";
}
if (property_exists($box, "name")) {
    echo "object:name\n";
}
if (property_exists($box, "secret")) {
    echo "object:secret\n";
}
if (property_exists($box, "cache")) {
    echo "object:static\n";
}
if (property_exists("BOX", "cache")) {
    echo "class:static\n";
}
if (!property_exists("Box", "missing")) {
    echo "class:missing\n";
}
if (!property_exists("Missing", "name")) {
    echo "missing-class:false\n";
}
$call = "property_exists";
if ($call($box, "name")) {
    echo "dynamic:exists\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:baseName\nobject:baseSecret\nobject:baseToken-private-false\nclass:baseShared\nobject:name\nobject:secret\nobject:static\nclass:static\nclass:missing\nmissing-class:false\ndynamic:exists\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn property_exists_requires_object_or_string_and_string_property_arguments() {
    let target_error = runtime_error("<?php\nvar_dump(property_exists(42, \"name\"));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 1);
    assert_eq!(
        target_error.message,
        "property_exists(): Argument #1 ($object_or_class) must be of type object|string, int given, called"
    );

    let property_error = runtime_error("<?php\nvar_dump(property_exists(\"Box\", 42));\n");

    assert_eq!(property_error.line, 2);
    assert_eq!(property_error.column, 1);
    assert_eq!(
        property_error.message,
        "property_exists(): Argument #2 ($property) must be of type string, int given, called"
    );
}

#[test]
fn method_exists_checks_declared_method_metadata() {
    let source = r#"<?php
class Box {
    public function open() {}
    protected function seal() {}
    private static function cache() {}
}

$box = new box();
if (method_exists($box, "open")) {
    echo "object:open\n";
}
if (method_exists($box, "SEAL")) {
    echo "object:seal\n";
}
if (method_exists($box, "cache")) {
    echo "object:static\n";
}
if (method_exists("BOX", "CACHE")) {
    echo "class:static\n";
}
if (!method_exists("Box", "missing")) {
    echo "class:missing\n";
}
if (!method_exists("Missing", "open")) {
    echo "missing-class:false\n";
}
$call = "method_exists";
if ($call($box, "OPEN")) {
    echo "dynamic:exists\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:open\nobject:seal\nobject:static\nclass:static\nclass:missing\nmissing-class:false\ndynamic:exists\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn method_exists_requires_object_or_string_and_string_method_arguments() {
    let target_error = runtime_error("<?php\nvar_dump(method_exists(42, \"open\"));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 1);
    assert_eq!(
        target_error.message,
        "method_exists(): Argument #1 ($object_or_class) must be of type object|string, int given, called"
    );

    let method_error = runtime_error("<?php\nvar_dump(method_exists(\"Box\", 42));\n");

    assert_eq!(method_error.line, 2);
    assert_eq!(method_error.column, 1);
    assert_eq!(
        method_error.message,
        "method_exists(): Argument #2 ($method) must be of type string, int given, called"
    );
}

#[test]
fn property_and_method_exists_autoload_missing_class_strings() {
    let source = r#"<?php
spl_autoload_register(function ($name) {
    echo "autoload:$name\n";
    if ($name === "AutoMeta") {
        class AutoMeta {
            public static $bob;
            public function run() {}
        }
    }
});

var_dump(property_exists("AutoMeta", "bob"));
var_dump(method_exists("AutoMeta", "run"));
var_dump(property_exists("", "bob"));
var_dump(method_exists("", "run"));
var_dump(method_exists("MissingMeta", "run"));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "autoload:AutoMeta\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nautoload:MissingMeta\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_callable_reports_magic_object_and_static_array_callables() {
    let source = r#"<?php
class HookProxy {
    public function __call($name, $args) {
        return $name . ":" . implode(",", $args);
    }
}

class ChildHookProxy extends HookProxy {}

class StaticHookProxy {
    public static function __callStatic($name, $args) {
        return $name . ":" . implode(",", $args);
    }
}

class StaticChildHookProxy extends StaticHookProxy {}

$proxy = new ChildHookProxy();

echo method_exists($proxy, "register") ? "object-method\n" : "object-missing\n";
echo is_callable(array($proxy, "register")) ? "object-callable\n" : "object-not-callable\n";
echo $proxy->register("init", 10), "\n";

echo method_exists("StaticChildHookProxy", "resolve") ? "static-method\n" : "static-missing\n";
echo is_callable(array("StaticChildHookProxy", "resolve")) ? "static-callable\n" : "static-not-callable\n";
echo StaticChildHookProxy::resolve("save_post", 20);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object-missing\nobject-callable\nregister:init,10\nstatic-missing\nstatic-callable\nresolve:save_post,20"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_methods_lists_public_declared_methods_in_declaration_order() {
    let source = r#"<?php
class Box {
    public function open() {}
    protected function seal() {}
    private static function cache() {}
    public static function make() {}
}

$box = new box();
$object_methods = get_class_methods($box);
print_r($object_methods);
echo count($object_methods), "|", $object_methods[0], "|", $object_methods[1], "\n";

$class_methods = get_class_methods("BOX");
echo $class_methods[0], "|", $class_methods[1], "\n";

$call = "get_class_methods";
$dynamic = $call($box);
echo $dynamic[0], "|", $dynamic[1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => open\n    [1] => make\n)\n2|open|make\nopen|make\nopen|make"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_methods_lists_methods_visible_from_current_scope_and_interfaces() {
    let source = r#"<?php
class C {
    private function privC() {}
    protected function protC() {}
    public function pubC() {}

    public static function testFromC() {
        echo "C:C=", implode(",", get_class_methods("C")), "\n";
        echo "C:D=", implode(",", get_class_methods("D")), "\n";
        echo "C:X=", implode(",", get_class_methods("X")), "\n";
    }
}

class D extends C {
    private function privD() {}
    protected function protD() {}
    public function pubD() {}

    public static function testFromD() {
        echo "D:C=", implode(",", get_class_methods("C")), "\n";
        echo "D:D=", implode(",", get_class_methods("D")), "\n";
        echo "D:X=", implode(",", get_class_methods("X")), "\n";
    }
}

class X {
    private function privX() {}
    protected function protX() {}
    public function pubX() {}

    public static function testFromX() {
        echo "X:C=", implode(",", get_class_methods("C")), "\n";
        echo "X:D=", implode(",", get_class_methods("D")), "\n";
        echo "X:X=", implode(",", get_class_methods("X")), "\n";
    }
}

interface I {
    public function pubI();
}

class IC implements I {
    public function pubI() {}
    private function privIC() {}
    protected function protIC() {}
    public function pubIC() {}

    public static function testFromIC() {
        echo "IC:I=", implode(",", get_class_methods("I")), "\n";
        echo "IC:IC=", implode(",", get_class_methods("IC")), "\n";
    }
}

echo "global:D=", implode(",", get_class_methods("D")), "\n";
C::testFromC();
D::testFromD();
X::testFromX();
echo "global:I=", implode(",", get_class_methods("I")), "\n";
echo "global:IC=", implode(",", get_class_methods("IC")), "\n";
IC::testFromIC();
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "global:D=pubD,testFromD,pubC,testFromC\n\
C:C=privC,protC,pubC,testFromC\n\
C:D=protD,pubD,testFromD,privC,protC,pubC,testFromC\n\
C:X=pubX,testFromX\n\
D:C=protC,pubC,testFromC\n\
D:D=privD,protD,pubD,testFromD,protC,pubC,testFromC\n\
D:X=pubX,testFromX\n\
X:C=pubC,testFromC\n\
X:D=pubD,testFromD,pubC,testFromC\n\
X:X=privX,protX,pubX,testFromX\n\
global:I=pubI\n\
global:IC=pubI,pubIC,testFromIC\n\
IC:I=pubI\n\
IC:IC=pubI,privIC,protIC,pubIC,testFromIC\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_methods_requires_object_or_valid_class_name_argument() {
    let execution = run_source(
        r#"<?php
foreach (array(42, "Missing") as $value) {
    try {
        get_class_methods($value);
    } catch (TypeError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:get_class_methods(): Argument #1 ($object_or_class) must be an object or a valid class name, int given\n\
TypeError:get_class_methods(): Argument #1 ($object_or_class) must be an object or a valid class name, string given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_vars_lists_public_declared_properties_with_null_defaults() {
    let source = r#"<?php
class Base {
    public $baseName;
    protected $baseSecret;
    public static $baseShared;
}

class Box extends Base {
    public $name;
    protected $secret;
    private $token;
    public static $shared;
    private static $cache;
}

$vars = get_class_vars("BOX");
print_r($vars);
echo count($vars), "|", array_key_exists("name", $vars), "|", array_key_exists("shared", $vars), "\n";

$call = "get_class_vars";
$dynamic = $call("Box");
echo count($dynamic), "|", array_key_exists("secret", $dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => \n    [baseName] => \n    [shared] => \n    [baseShared] => \n)\n4|1|1\n4|"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_vars_requires_declared_class_string_argument() {
    let source = r#"<?php
foreach ([42, "Missing"] as $class) {
    try {
        var_dump(get_class_vars($class));
    } catch (TypeError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "TypeError:get_class_vars(): Argument #1 ($class) must be a valid class name, 42 given\nTypeError:get_class_vars(): Argument #1 ($class) must be a valid class name, Missing given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_object_vars_lists_current_public_instance_property_values() {
    let source = r#"<?php
class Base {
    public $baseName;
    protected $baseSecret;
}

class Box extends Base {
    public $name;
    protected $secret;
    private $token;
    public $count;
    public static $shared;
}

$box = new box();
$box->baseName = "Root";
$box->name = "Ada";
$box->count = 3;
$vars = get_object_vars($box);
print_r($vars);
echo count($vars), "|", $vars["baseName"], "|", $vars["name"], "|", $vars["count"], "|", array_key_exists("secret", $vars), "\n";

$call = "get_object_vars";
$dynamic = $call($box);
echo count($dynamic), "|", $dynamic["baseName"], "|", $dynamic["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [baseName] => Root\n    [name] => Ada\n    [count] => 3\n)\n3|Root|Ada|3|\n3|Root|Ada"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_object_vars_uses_current_method_visibility_context() {
    let source = r#"<?php
class ParentBox {
    private $name = "parent-name";
    public $shared = "shared";

    public function parentVars() {
        $vars = get_object_vars($this);
        echo "parent:";
        foreach ($vars as $key => $value) {
            echo $key, "=", $value, ";";
        }
        echo "\n";
    }
}

class ChildBox extends ParentBox {
    public $name = "child-name";
    private $token = "child-token";

    public function childVars() {
        $vars = get_object_vars($this);
        echo "child:";
        foreach ($vars as $key => $value) {
            echo $key, "=", $value, ";";
        }
        echo "\n";
    }
}

$box = new ChildBox();
$box->parentVars();
$box->childVars();
$vars = get_object_vars($box);
echo "external:";
foreach ($vars as $key => $value) {
    echo $key, "=", $value, ";";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "parent:name=parent-name;shared=shared;\nchild:shared=shared;name=child-name;token=child-token;\nexternal:shared=shared;name=child-name;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn allow_dynamic_properties_child_can_shadow_inherited_private_property() {
    let source = r#"<?php
class ParentSlot {
    private $prop = "parent";

    public function parentDump() {
        echo "parent-foreach:";
        foreach ($this as $key => $value) {
            echo $key, "=", $value, ";";
        }
        echo "\n";

        echo "parent-vars:";
        foreach (get_object_vars($this) as $key => $value) {
            echo $key, "=", $value, ";";
        }
        echo "\n";
    }
}

#[AllowDynamicProperties]
class ChildSlot extends ParentSlot {
    public function childDump() {
        echo "child-foreach:";
        foreach ($this as $key => $value) {
            echo $key, "=", $value, ";";
        }
        echo "\n";

        echo "child-vars:";
        foreach (get_object_vars($this) as $key => $value) {
            echo $key, "=", $value, ";";
        }
        echo "\n";
    }
}

$box = new ChildSlot();
$box->prop = "dynamic";
$box->parentDump();
$box->childDump();

echo "external:";
foreach (get_object_vars($box) as $key => $value) {
    echo $key, "=", $value, ";";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "parent-foreach:prop=parent;\nparent-vars:prop=parent;\nchild-foreach:prop=dynamic;\nchild-vars:prop=dynamic;\nexternal:prop=dynamic;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_object_vars_requires_object_argument() {
    let target_error = runtime_error("<?php\nvar_dump(get_object_vars(42));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call get_object_vars(): argument must be object, got int"
    );
}

#[test]
fn get_mangled_object_vars_lists_current_mangled_instance_property_values() {
    let source = r#"<?php
class Box {
    public $name;
    protected $secret;
    private $token;
    public $count;
    public static $shared;
}

$box = new box();
$box->name = "Ada";
$box->count = 3;
$vars = get_mangled_object_vars($box);
$keys = array_keys($vars);
echo count($vars), "\n";
echo strlen($keys[0]), "|", $keys[0] === "name", "|", $vars[$keys[0]], "\n";
echo strlen($keys[1]), "|", $keys[1] === "secret", "|", $vars[$keys[1]] === null, "\n";
echo strlen($keys[2]), "|", $keys[2] === "token", "|", $vars[$keys[2]] === null, "\n";
echo strlen($keys[3]), "|", $keys[3] === "count", "|", $vars[$keys[3]], "\n";
echo array_key_exists("secret", $vars), "|", array_key_exists("token", $vars), "\n";

$call = "get_mangled_object_vars";
$dynamic = $call($box);
$dynamicKeys = array_keys($dynamic);
echo count($dynamic), "|", strlen($dynamicKeys[1]), "|", strlen($dynamicKeys[2]), "|", $dynamic[$dynamicKeys[0]];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "4\n4|1|Ada\n9||1\n10||1\n5|1|3\n|\n4|9|10|Ada"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_mangled_object_vars_requires_object_argument() {
    let target_error = runtime_error("<?php\nvar_dump(get_mangled_object_vars(42));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call get_mangled_object_vars(): argument must be object, got int"
    );
}

#[test]
fn is_a_checks_current_single_parent_relationships() {
    let source = r#"<?php
class Box {}
class Child extends Box {}
class Crate {}

$box = new box();
$child = new Child();
if (is_a($box, "Box")) {
    echo "object:box\n";
}
if (is_a($child, "Box")) {
    echo "object:child-is-box\n";
}
if (is_a($box, "box")) {
    echo "object:case-insensitive\n";
}
if (!is_a($box, "Crate")) {
    echo "object:not-crate\n";
}
if (!is_a("Box", "Box")) {
    echo "string:default-false\n";
}
if (is_a("CHILD", "box", true)) {
    echo "string:allowed-child\n";
}
if (!is_a("Missing", "Box", true)) {
    echo "missing-source:false\n";
}
if (!is_a($box, "Missing")) {
    echo "missing-target:false\n";
}
$call = "is_a";
if ($call($box, "BOX")) {
    echo "dynamic:object\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:box\nobject:child-is-box\nobject:case-insensitive\nobject:not-crate\nstring:default-false\nstring:allowed-child\nmissing-source:false\nmissing-target:false\ndynamic:object\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_a_requires_string_class_name_and_bool_allow_string_arguments() {
    let class_error =
        runtime_error("<?php\nclass Box {}\n$box = new Box();\nvar_dump(is_a($box, 42));\n");

    assert_eq!(class_error.line, 4);
    assert_eq!(class_error.column, 10);
    assert_eq!(
        class_error.message,
        "unsupported call is_a(): class name argument must be string in the current subset, got int"
    );

    let allow_string_error = runtime_error("<?php\nvar_dump(is_a(\"Box\", \"Box\", 1));\n");

    assert_eq!(allow_string_error.line, 2);
    assert_eq!(allow_string_error.column, 10);
    assert_eq!(
        allow_string_error.message,
        "unsupported call is_a(): allow_string argument must be bool in the current subset, got int"
    );
}

#[test]
fn is_subclass_of_checks_current_single_parent_metadata() {
    let source = r#"<?php
class Box {}
class Child extends Box {}
class Crate {}

$box = new box();
$child = new Child();
if (is_subclass_of($child, "Box")) {
    echo "object:child-true\n";
}
if (!is_subclass_of($box, "Box")) {
    echo "object:exact-false\n";
}
if (!is_subclass_of($box, "Crate")) {
    echo "object:other-false\n";
}
if (!is_subclass_of("Box", "Box")) {
    echo "string:default-false\n";
}
if (is_subclass_of("CHILD", "box")) {
    echo "string:default-child-true\n";
}
if (!is_subclass_of("CHILD", "box", false)) {
    echo "string:disallowed-child-false\n";
}
if (is_subclass_of("CHILD", "box", true)) {
    echo "string:allowed-child-true\n";
}
if (!is_subclass_of("Missing", "Box", true)) {
    echo "missing-source:false\n";
}
if (!is_subclass_of($box, "Missing")) {
    echo "missing-target:false\n";
}
$call = "is_subclass_of";
if ($call($child, "BOX")) {
    echo "dynamic:true\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:child-true\nobject:exact-false\nobject:other-false\nstring:default-false\nstring:default-child-true\nstring:disallowed-child-false\nstring:allowed-child-true\nmissing-source:false\nmissing-target:false\ndynamic:true\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_subclass_of_requires_supported_argument_types() {
    let source_execution = run_source("<?php\nvar_dump(is_subclass_of(42, \"Box\"));\n").unwrap();
    assert_eq!(source_execution.stdout, "bool(false)\n");
    assert_eq!(source_execution.stderr, "");
    assert_eq!(source_execution.exit_code, 0);

    let class_error = runtime_error(
        "<?php\nclass Box {}\n$box = new Box();\nvar_dump(is_subclass_of($box, 42));\n",
    );

    assert_eq!(class_error.line, 4);
    assert_eq!(class_error.column, 10);
    assert_eq!(
        class_error.message,
        "unsupported call is_subclass_of(): class name argument must be string in the current subset, got int"
    );

    let allow_string_error =
        runtime_error("<?php\nvar_dump(is_subclass_of(\"Box\", \"Box\", 1));\n");

    assert_eq!(allow_string_error.line, 2);
    assert_eq!(allow_string_error.column, 10);
    assert_eq!(
        allow_string_error.message,
        "unsupported call is_subclass_of(): allow_string argument must be bool in the current subset, got int"
    );
}

#[test]
fn get_parent_class_reports_current_single_parent_metadata() {
    let source = r#"<?php
class Box {}
class Child extends Box {}

$box = new box();
$child = new Child();
if (get_parent_class($child) === "Box") {
    echo "object:child-parent\n";
}
if (!get_parent_class($box)) {
    echo "object:false\n";
}
if (get_parent_class("CHILD") === "Box") {
    echo "string:child-parent\n";
}
if (!get_parent_class("BOX")) {
    echo "string:false\n";
}
$call = "get_parent_class";
if ($call($child) === "Box") {
    echo "dynamic:child-parent";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:child-parent\nobject:false\nstring:child-parent\nstring:false\ndynamic:child-parent"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_parent_class_requires_object_or_string_argument() {
    let execution = run_source(
        r#"<?php
spl_autoload_register(function ($class) {
    echo "autoload:$class\n";
});

foreach (array(42, "Missing") as $value) {
    try {
        get_parent_class($value);
    } catch (TypeError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:get_parent_class(): Argument #1 ($object_or_class) must be an object or a valid class name, int given\n\
autoload:Missing\n\
TypeError:get_parent_class(): Argument #1 ($object_or_class) must be an object or a valid class name, string given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_declared_classes_returns_current_program_classes_in_declaration_order() {
    let source = r#"<?php
class Box {}
class Profile {}

$declared = get_declared_classes();
print_r($declared);
echo count($declared), "|", $declared[0], "|", $declared[1], "|", $declared[2], "\n";

$call = "get_declared_classes";
$dynamic = $call();
echo $dynamic[0], "|", $dynamic[1], "|", $dynamic[2];
"#;

    let execution = run_source(source).unwrap();
    let mut declared = CORE_CLASS_NAMES.to_vec();
    declared.extend(["Box", "Profile"]);
    let mut expected = expected_print_r_array(&declared);
    expected.push_str(&format!(
        "{}|Exception|Error|Uri\\InvalidUriException\n",
        declared.len()
    ));
    expected.push_str("Exception|Error|Uri\\InvalidUriException");
    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_declared_classes_reports_declared_enums_as_class_like_metadata() {
    let source = r#"<?php
namespace App;

enum Mode { case Front; }
enum Status {}

$declared = get_declared_classes();
print_r($declared);
echo count($declared), "\n";
"#;

    let execution = run_source(source).unwrap();
    let mut declared = CORE_CLASS_NAMES.to_vec();
    declared.extend(["App\\Mode", "App\\Status", "App\\Mode", "App\\Status"]);
    let mut expected = expected_print_r_array(&declared);
    expected.push_str(&format!("{}\n", declared.len()));
    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn core_reflection_exception_metadata_is_declared_and_extends_exception() {
    let execution = run_source(
        r#"<?php
echo class_exists("ReflectionException") ? "exists\n" : "missing\n";
echo is_subclass_of("ReflectionException", "Exception") ? "extends\n" : "no-parent\n";
echo get_parent_class("ReflectionException"), "\n";
$reflection = new ReflectionClass("ReflectionException");
echo $reflection->getName(), "|", $reflection->getParentClass()->getName(), "|", ($reflection->isInstantiable() ? "1" : "0"), "\n";
$classes = get_declared_classes();
echo array_search("ReflectionException", $classes, true);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        format!(
            "exists\nextends\nException\nReflectionException|Exception|1\n{}",
            CORE_CLASS_NAMES
                .iter()
                .position(|name| *name == "ReflectionException")
                .unwrap()
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_object_reflects_object_instances_as_reflection_class_subtype() {
    let execution = run_source(
        r#"<?php
class Base {}
class Plugin extends Base {
    public $name = "hook";
    public function run() { return "ok"; }
}

$plugin = new Plugin();
$reflection = new ReflectionObject($plugin);
echo get_class($reflection), "|", is_a($reflection, "ReflectionClass"), "\n";
echo $reflection->getName(), "|", $reflection->getParentClass()->getName(), "|", $reflection->isInstance($plugin), "|", $reflection->isSubclassOf(Base::class), "\n";
echo $reflection->hasMethod("run"), "|", get_class($reflection->getMethod("run")), "|", $reflection->hasProperty("name"), "|", $reflection->name;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ReflectionObject|1\nPlugin|Base|1|1\n1|ReflectionMethod|1|Plugin"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_declared_classes_requires_no_arguments() {
    let error = runtime_error("<?php\nvar_dump(get_declared_classes(42));\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "arity mismatch for get_declared_classes(): expected 0 argument(s), got 1"
    );
}

#[test]
fn get_declared_interfaces_reports_declared_interface_table() {
    let source = r#"<?php
namespace App;
class Box {}
interface Logger {}
interface Hookable {}

$declared = get_declared_interfaces();
print_r($declared);
echo count($declared), "\n";

$call = "get_declared_interfaces";
$dynamic = $call();
echo count($dynamic);
"#;

    let execution = run_source(source).unwrap();
    let mut declared = CORE_INTERFACE_NAMES.to_vec();
    declared.extend(["App\\Logger", "App\\Hookable"]);
    let mut expected = expected_print_r_array(&declared);
    expected.push_str(&format!("{0}\n{0}", declared.len()));
    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_declared_interfaces_requires_no_arguments() {
    let error = runtime_error("<?php\nvar_dump(get_declared_interfaces(42));\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "arity mismatch for get_declared_interfaces(): expected 0 argument(s), got 1"
    );
}

#[test]
fn class_implements_reports_ordered_interface_metadata() {
    let source = r#"<?php
interface RootHook {}
interface ChildHook extends RootHook {}
interface ParentHook {}

class ParentService implements ParentHook {}
class Service extends ParentService implements ChildHook {}

$service = new Service();
print_r(class_implements($service));
print_r(class_implements("Service", false));

$call = "class_implements";
$dynamic = $call("Service");
echo count($dynamic), "\n";
echo isset($dynamic["RootHook"]) ? "root\n" : "missing\n";
echo class_implements("Missing", false) ? "missing-true" : "missing-false";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [ParentHook] => ParentHook\n    [ChildHook] => ChildHook\n    [RootHook] => RootHook\n)\nArray\n(\n    [ParentHook] => ParentHook\n    [ChildHook] => ChildHook\n    [RootHook] => RootHook\n)\n3\nroot\nmissing-false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn class_implements_requires_object_or_string_and_bool_autoload_arguments() {
    let execution = run_source(
        r#"<?php
try {
    class_implements(42);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:class_implements(): Argument #1 ($object_or_class) must be of type object|string, int given\n"
    );
    assert_eq!(execution.exit_code, 0);

    let autoload_error = runtime_error("<?php\nvar_dump(class_implements(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call class_implements(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn class_uses_reports_direct_trait_metadata() {
    let source = r#"<?php
namespace App;

trait RegistersHooks {}
trait AddsFilters {}
trait ParentOnly {}

class BasePlugin {
    use ParentOnly;
}

class Plugin extends BasePlugin {
    use RegistersHooks, AddsFilters;
}

$plugin = new Plugin();
print_r(class_uses($plugin));
print_r(class_uses("App\\Plugin", false));

$call = "class_uses";
$dynamic = $call("App\\Plugin");
echo count($dynamic), "\n";
echo isset($dynamic["App\\RegistersHooks"]) ? "registers\n" : "missing\n";
echo isset($dynamic["App\\ParentOnly"]) ? "parent-present\n" : "parent-not-listed\n";
echo class_uses("App\\Missing", false) ? "missing-true" : "missing-false";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [App\\RegistersHooks] => App\\RegistersHooks\n    [App\\AddsFilters] => App\\AddsFilters\n)\nArray\n(\n    [App\\RegistersHooks] => App\\RegistersHooks\n    [App\\AddsFilters] => App\\AddsFilters\n)\n2\nregisters\nparent-not-listed\nmissing-false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_direct_trait_metadata() {
    let execution = run_source(
        r#"<?php
trait RegistersHooks {}
trait AddsFilters {}

class Plugin {
    use RegistersHooks, AddsFilters;
}

interface Hookable {}

function yn($value) {
    return $value ? "1" : "0";
}

$class = new ReflectionClass(Plugin::class);
foreach ($class->getTraitNames() as $index => $name) {
    echo "name|", $index, "|", $name, "\n";
}
foreach ($class->getTraits() as $key => $trait) {
    echo "trait|", $key, "|", get_class($trait), "|", $trait->getName(), "|", yn($trait->isTrait()), "|", $trait->getShortName(), "\n";
}

$interface = new ReflectionClass(Hookable::class);
echo "interface|", count($interface->getTraitNames()), "|", count($interface->getTraits()), "\n";

$trait = new ReflectionClass(RegistersHooks::class);
echo "trait-empty|", count($trait->getTraitNames()), "|", count($trait->getTraits());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "name|0|RegistersHooks\nname|1|AddsFilters\ntrait|RegistersHooks|ReflectionClass|RegistersHooks|1|RegistersHooks\ntrait|AddsFilters|ReflectionClass|AddsFilters|1|AddsFilters\ninterface|0|0\ntrait-empty|0|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn class_uses_requires_object_or_string_and_bool_autoload_arguments() {
    let execution = run_source(
        r#"<?php
try {
    class_uses(42);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:class_uses(): Argument #1 ($object_or_class) must be of type object|string, int given\n"
    );
    assert_eq!(execution.exit_code, 0);

    let autoload_error = runtime_error("<?php\nvar_dump(class_uses(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call class_uses(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn class_parents_reports_parent_metadata_for_recursive_trait_helpers() {
    let source = r#"<?php
namespace App;

trait RootTrait {}
trait MidTrait {}
trait LeafTrait {}

class Root {
    use RootTrait;
}

class Mid extends Root {
    use MidTrait;
}

class Leaf extends Mid {
    use LeafTrait;
}

function class_uses_recursive_probe($class) {
    $traits = array();
    foreach (class_parents($class, false) as $parent) {
        foreach (class_uses($parent, false) as $trait) {
            $traits[$trait] = $trait;
        }
    }
    foreach (class_uses($class, false) as $trait) {
        $traits[$trait] = $trait;
    }
    return $traits;
}

$leaf = new Leaf();
print_r(class_parents($leaf));
print_r(class_parents("App\\Leaf", false));

$call = "class_parents";
$dynamic = $call("App\\Leaf");
echo count($dynamic), "\n";

print_r(class_uses_recursive_probe($leaf));
echo class_parents("App\\Missing", false) ? "missing-true" : "missing-false";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [App\\Mid] => App\\Mid\n    [App\\Root] => App\\Root\n)\nArray\n(\n    [App\\Mid] => App\\Mid\n    [App\\Root] => App\\Root\n)\n2\nArray\n(\n    [App\\MidTrait] => App\\MidTrait\n    [App\\RootTrait] => App\\RootTrait\n    [App\\LeafTrait] => App\\LeafTrait\n)\nmissing-false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn class_parents_requires_object_or_string_and_bool_autoload_arguments() {
    let execution = run_source(
        r#"<?php
try {
    class_parents(42);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "TypeError:class_parents(): Argument #1 ($object_or_class) must be of type object|string, int given\n"
    );
    assert_eq!(execution.exit_code, 0);

    let autoload_error = runtime_error("<?php\nvar_dump(class_parents(\"Box\", []));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call class_parents(): autoload argument must be bool-like scalar in the current subset, got array"
    );
}

#[test]
fn class_list_helpers_warn_and_return_false_for_missing_string_classes() {
    let source = r#"<?php
function capture_warning($errno, $message) {
    echo "warning:", $message, "\n";
}

spl_autoload_register(function ($class) {
    echo "autoload:", $class, "\n";
});

foreach (array("class_implements", "class_uses", "class_parents") as $function) {
    echo "--", $function, "--\n";
    set_error_handler("capture_warning");
    var_dump($function("Missing" . $function));
    var_dump($function("", true));
    var_dump($function("StillMissing" . $function, false));
    restore_error_handler();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "--class_implements--\n\
autoload:Missingclass_implements\n\
warning:class_implements(): Class Missingclass_implements does not exist and could not be loaded\n\
bool(false)\n\
warning:class_implements(): Class  does not exist and could not be loaded\n\
bool(false)\n\
warning:class_implements(): Class StillMissingclass_implements does not exist\n\
bool(false)\n\
--class_uses--\n\
autoload:Missingclass_uses\n\
warning:class_uses(): Class Missingclass_uses does not exist and could not be loaded\n\
bool(false)\n\
warning:class_uses(): Class  does not exist and could not be loaded\n\
bool(false)\n\
warning:class_uses(): Class StillMissingclass_uses does not exist\n\
bool(false)\n\
--class_parents--\n\
autoload:Missingclass_parents\n\
warning:class_parents(): Class Missingclass_parents does not exist and could not be loaded\n\
bool(false)\n\
warning:class_parents(): Class  does not exist and could not be loaded\n\
bool(false)\n\
warning:class_parents(): Class StillMissingclass_parents does not exist\n\
bool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trait_body_use_composes_public_members() {
    let execution = run_source(
        r#"<?php
trait HookLabels {
    public const NESTED = "nested";
    public function label($suffix) {
        return "nested:" . $suffix;
    }
}

trait HookTools {
    use HookLabels;
    public const SOURCE = "tools";
    public function register($hook) {
        return $this->label($hook) . ":" . self::SOURCE . ":" . self::NESTED;
    }
}

class Plugin {
    use HookTools;
}

$plugin = new Plugin();
echo $plugin->register("init"), "\n";
echo $plugin->label("admin"), "\n";
echo Plugin::SOURCE, "|", Plugin::NESTED, "\n";
$class = new ReflectionClass(Plugin::class);
echo implode(",", $class->getTraitNames()), "\n";
$names = array("register", "label");
$count = count($names);
foreach ($names as $index => $name) {
    $method = new ReflectionMethod(Plugin::class, $name);
    echo $name, "|", $method->getDeclaringClass()->getName(), $index + 1 === $count ? "" : "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "nested:init:tools:nested\nnested:admin\ntools|nested\nHookTools\nregister|Plugin\nlabel|Plugin"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trait_body_use_adaptations_compose_nested_public_methods() {
    let execution = run_source(
        r#"<?php
interface PluginContract {
    public function label_alias($hook);
}

trait PrimaryLabel {
    public function label($hook) {
        return "primary:" . $hook . ":" . get_class($this);
    }
}

trait FallbackLabel {
    public function label($hook) {
        return "fallback:" . $hook . ":" . get_class($this);
    }
}

trait HookTools {
    use PrimaryLabel, FallbackLabel {
        PrimaryLabel::label insteadof FallbackLabel;
        PrimaryLabel::label as public label_alias;
        PrimaryLabel::label as protected hidden_label;
    }

    public function boot($hook) {
        return $this->hidden_label($hook);
    }
}

class Plugin implements PluginContract {
    use HookTools;
}

$plugin = new Plugin();
echo $plugin->label("init"), "\n";
echo $plugin->label_alias("admin"), "\n";
echo $plugin->boot("rest"), "\n";
echo method_exists($plugin, "hidden_label") ? "hidden-exists\n" : "hidden-missing\n";

$methods = get_class_methods($plugin);
echo count($methods), "|";
echo in_array("label", $methods) ? "label" : "missing";
echo "|";
echo in_array("label_alias", $methods) ? "alias" : "missing";
echo "|";
echo in_array("boot", $methods) ? "boot\n" : "missing\n";

$hidden = new ReflectionMethod(Plugin::class, "hidden_label");
echo $hidden->getDeclaringClass()->getName(), "|";
echo $hidden->isProtected() ? "protected" : "not-protected";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "primary:init:Plugin\nprimary:admin:Plugin\nprimary:rest:Plugin\nhidden-exists\n3|label|alias|boot\nPlugin|protected"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_bounded_class_interface_and_trait_metadata() {
    let execution = run_source(
        r#"<?php
interface RootContract {
    public function root();
}

interface HookContract extends RootContract {
    public function boot($hook = null);
}

trait HookTools {
    public function helper() {
        return "helper";
    }
}

class BasePlugin {
    public function root() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public function boot($hook = null) {}
}

$plugin = new Plugin();
$class = new ReflectionClass($plugin);
echo $class->getName(), "\n";
echo $class->getShortName(), "\n";
echo $class->isInstantiable() ? "instantiable\n" : "not-instantiable\n";
echo $class->hasMethod("boot") ? "boot-method\n" : "missing-boot\n";
echo $class->hasMethod("helper") ? "helper-method\n" : "missing-helper\n";
print_r($class->getInterfaceNames());

$parent = $class->getParentClass();
echo $parent ? $parent->getName() . "\n" : "no-parent\n";

$interface = new ReflectionClass(HookContract::class);
echo $interface->isInterface() ? "interface\n" : "not-interface\n";
echo $interface->hasMethod("root") ? "root-method\n" : "missing-root\n";
print_r($interface->getInterfaceNames());

$trait = new ReflectionClass(HookTools::class);
echo $trait->isTrait() ? "trait\n" : "not-trait\n";
echo $trait->hasMethod("helper") ? "trait-helper" : "missing-helper";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Plugin\nPlugin\ninstantiable\nboot-method\nhelper-method\nArray\n(\n    [0] => HookContract\n    [1] => RootContract\n)\nBasePlugin\ninterface\nroot-method\nArray\n(\n    [0] => RootContract\n)\ntrait\ntrait-helper"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_get_interfaces_returns_named_reflection_objects() {
    let execution = run_source(
        r#"<?php
interface RootContract {}
interface LeafContract extends RootContract {}
class BasePlugin {}
class Plugin extends BasePlugin implements LeafContract {}

function line($label, $interfaces) {
    ksort($interfaces);
    foreach ($interfaces as $key => $interface) {
        echo $label, "|", $key, "|", get_class($interface), "|", $interface->name, "|", $interface->getName(), "\n";
    }
}

$class = new ReflectionClass(Plugin::class);
line("class", $class->getInterfaces());
$parent = $class->getParentClass();
echo "parent|", $parent->name, "|", $parent->getName(), "\n";

$interface = new ReflectionClass(LeafContract::class);
line("interface", $interface->getInterfaces());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "class|LeafContract|ReflectionClass|LeafContract|LeafContract\nclass|RootContract|ReflectionClass|RootContract|RootContract\nparent|BasePlugin|BasePlugin\ninterface|RootContract|ReflectionClass|RootContract|RootContract\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_bounded_class_source_metadata() {
    let execution = run_source_with_source_file(
        r#"<?php
/**
 * Base class metadata.
 */
class PluginBase {
}

/**
 * WordPress hook plugin metadata.
 */
class HookPlugin extends PluginBase {
    public function boot() {}
}

/**
 * Hook contract metadata.
 */
interface HookContract {
    public function boot();
}

/**
 * Hook trait metadata.
 */
trait HookTools {
    public function helper() {}
}

class PlainClass {}

function yn($value) {
    return $value ? "1" : "0";
}

function class_doc_line($label, $class) {
    $doc = $class->getDocComment();
    echo $label, "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

$suffix = "tests/fixtures/milestone1501/class_reflection_source_metadata.php";
$class = new ReflectionClass(HookPlugin::class);
echo "class-source|", substr($class->getFileName(), -strlen($suffix)), "|", $class->getStartLine(), "|", $class->getEndLine(), "\n";
class_doc_line("class-doc", $class);
$parent = $class->getParentClass();
echo "parent-lines|", $parent->getStartLine(), "|", $parent->getEndLine(), "\n";
$interface = new ReflectionClass(HookContract::class);
echo "interface-lines|", $interface->getStartLine(), "|", $interface->getEndLine(), "\n";
class_doc_line("interface-doc", $interface);
$trait = new ReflectionClass(HookTools::class);
echo "trait-lines|", $trait->getStartLine(), "|", $trait->getEndLine(), "\n";
class_doc_line("trait-doc", $trait);
$plain = new ReflectionClass(PlainClass::class);
echo "plain|", $plain->getStartLine(), "|", $plain->getEndLine(), "|", yn($plain->getDocComment() === false);
"#,
        "tests/fixtures/milestone1501/class_reflection_source_metadata.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "class-source|tests/fixtures/milestone1501/class_reflection_source_metadata.php|11|13\nclass-doc|1|/**\\n * WordPress hook plugin metadata.\\n */\nparent-lines|5|6\ninterface-lines|18|20\ninterface-doc|1|/**\\n * Hook contract metadata.\\n */\ntrait-lines|25|27\ntrait-doc|1|/**\\n * Hook trait metadata.\\n */\nplain|29|29|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_reports_bounded_method_modifier_metadata() {
    let execution = run_source(
        r#"<?php
interface HookContract {
    public static function register();
    public function boot($hook = null);
}

abstract class BasePlugin {
    abstract protected function compute();
    public final function seal() {}
}

trait HookTools {
    public function helper() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public function __construct() {}
    public function boot($hook = null) {}
    public static function register() {}
    protected function compute() {}
    private function hidden() {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $method) {
    echo $label, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "|", $method->getModifiers(), "|", yn($method->isPublic()), yn($method->isProtected()), yn($method->isPrivate()), yn($method->isStatic()), yn($method->isFinal()), yn($method->isAbstract()), yn($method->isConstructor()), "\n";
}

echo ReflectionMethod::IS_PUBLIC, "|", ReflectionMethod::IS_PROTECTED, "|", ReflectionMethod::IS_PRIVATE, "|", ReflectionMethod::IS_STATIC, "|", ReflectionMethod::IS_FINAL, "|", ReflectionMethod::IS_ABSTRACT, "\n";
line("boot", new ReflectionMethod(Plugin::class, "boot"));
line("ctor", new ReflectionMethod(new Plugin(), "__construct"));
line("static", new ReflectionMethod(Plugin::class, "register"));
line("protected", new ReflectionMethod(Plugin::class, "compute"));
line("private", new ReflectionMethod(Plugin::class, "hidden"));
line("final", new ReflectionMethod(Plugin::class, "seal"));
line("abstract", new ReflectionMethod(BasePlugin::class, "compute"));
line("interface", new ReflectionMethod(HookContract::class, "register"));
$trait = new ReflectionMethod(HookTools::class, "helper");
echo "trait|", $trait->getName(), "|", $trait->getDeclaringClass()->getName(), "|", $trait->getModifiers(), "|", yn($trait->isPublic()), yn($trait->isProtected()), yn($trait->isPrivate()), yn($trait->isStatic()), yn($trait->isFinal()), yn($trait->isAbstract()), yn($trait->isConstructor());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|2|4|16|32|64\nboot|boot|Plugin|1|1000000\nctor|__construct|Plugin|1|1000001\nstatic|register|Plugin|17|1001000\nprotected|compute|Plugin|2|0100000\nprivate|hidden|Plugin|4|0010000\nfinal|seal|BasePlugin|33|1000100\nabstract|compute|BasePlugin|66|0100010\ninterface|register|HookContract|81|1001010\ntrait|helper|HookTools|1|1000000"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_get_method_returns_bounded_method_metadata() {
    let execution = run_source(
        r#"<?php
interface HookContract {
    public static function register();
}

class BasePlugin {
    protected function inherited() {}
}

trait HookTools {
    public function helper() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public static function register() {}
    private function hidden() {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $method, $end = "\n") {
    echo $label, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "|", $method->getModifiers(), "|", yn($method->isPublic()), yn($method->isProtected()), yn($method->isPrivate()), yn($method->isStatic()), $end;
}

$plugin = new ReflectionClass(Plugin::class);
line("static", $plugin->getMethod("register"));
line("private", $plugin->getMethod("hidden"));
line("inherited", $plugin->getMethod("inherited"));
line("trait-composed", $plugin->getMethod("helper"));

$contract = new ReflectionClass(HookContract::class);
line("interface", $contract->getMethod("register"));

$trait = new ReflectionClass(HookTools::class);
line("trait", $trait->getMethod("helper"), "");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "static|register|Plugin|17|1001\nprivate|hidden|Plugin|4|0010\ninherited|inherited|BasePlugin|2|0100\ntrait-composed|helper|Plugin|1|1000\ninterface|register|HookContract|81|1001\ntrait|helper|HookTools|1|1000"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_get_methods_returns_bounded_method_lists() {
    let execution = run_source(
        r#"<?php
interface RootContract {
    public function root($hook = "init");
}

interface HookContract extends RootContract {
    public static function register();
}

class BasePlugin {
    protected function inherited() {}
    private function baseHidden() {}
    public final function seal() {}
}

trait HookTools {
    public function helper() {}
    public function label() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public static function register() {}
    private function hidden() {}
    public function root($hook = "init") {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function dump_methods($label, $methods) {
    $lines = array();
    foreach ($methods as $method) {
        $lines[$method->getName()] = $method->getDeclaringClass()->getName() . ":" . $method->getModifiers() . ":" . yn($method->isStatic()) . yn($method->isAbstract());
    }
    foreach ($lines as $name => $line) {
        echo $label, "|", $name, "|", $line, "\n";
    }
}

function method_line($label, $method, $ending = "\n") {
    echo $label, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), ":", $method->getModifiers(), ":", yn($method->isStatic()), yn($method->isAbstract()), $ending;
}

$plugin = new ReflectionClass(Plugin::class);
dump_methods("all", $plugin->getMethods());
dump_methods("public", $plugin->getMethods(ReflectionMethod::IS_PUBLIC));
dump_methods("static", $plugin->getMethods(ReflectionMethod::IS_STATIC));
echo "zero|", count($plugin->getMethods(0)), "\n";

$interface = new ReflectionClass(HookContract::class);
dump_methods("interface", $interface->getMethods());

$trait = new ReflectionClass(HookTools::class);
$traitMethods = $trait->getMethods();
method_line("trait", $traitMethods[0]);
method_line("trait", $traitMethods[1], "");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "all|register|Plugin:17:10\nall|hidden|Plugin:4:00\nall|root|Plugin:1:00\nall|inherited|BasePlugin:2:00\nall|seal|BasePlugin:33:00\nall|helper|Plugin:1:00\nall|label|Plugin:1:00\npublic|register|Plugin:17:10\npublic|root|Plugin:1:00\npublic|seal|BasePlugin:33:00\npublic|helper|Plugin:1:00\npublic|label|Plugin:1:00\nstatic|register|Plugin:17:10\nzero|0\ninterface|register|HookContract:81:11\ninterface|root|RootContract:65:01\ntrait|helper|HookTools:1:00\ntrait|label|HookTools:1:00"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_bounded_recursive_trait_metadata() {
    let execution = run_source(
        r#"<?php
trait BaseHooks {
    public function baseHook($suffix = "ok") {
        return "base:" . $suffix;
    }
}

trait HasHooks {
    use BaseHooks;

    public function directHook() {
        return "direct";
    }
}

class Plugin {
    use HasHooks;
}

function yn($value) {
    return $value ? "1" : "0";
}

$trait = new ReflectionClass(HasHooks::class);
echo "names|", implode(",", $trait->getTraitNames()), "\n";
foreach ($trait->getTraits() as $name => $reflected) {
    echo "trait|", $name, "|", $reflected->getName(), "|", yn($reflected->isTrait()), "\n";
}
foreach ($trait->getMethods() as $method) {
    echo "method|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "|", yn($method->isPublic()), yn($method->isAbstract()), "\n";
}
echo "has|", yn($trait->hasMethod("baseHook")), "\n";
$method = $trait->getMethod("baseHook");
echo "get|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "\n";
$constructed = new ReflectionMethod(HasHooks::class, "baseHook");
echo "construct|", $constructed->getName(), "|", $constructed->getDeclaringClass()->getName(), "\n";
$plugin = new Plugin();
echo "call|", $plugin->baseHook("wp"), "|", $plugin->directHook();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "names|BaseHooks\ntrait|BaseHooks|BaseHooks|1\nmethod|directHook|HasHooks|10\nmethod|baseHook|HasHooks|10\nhas|1\nget|baseHook|HasHooks\nconstruct|baseHook|HasHooks\ncall|base:wp|direct"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_reports_bounded_source_metadata() {
    let execution = run_source_with_source_file(
        r#"<?php
class HookBase {
    /**
     * Parent hook metadata.
     */
    public function inherited($value) {
        return $value;
    }
}

class HookPlugin extends HookBase {
    /**
     * Registers WordPress hooks.
     */
    public function register($hook) {
        return $hook;
    }

    public function noDoc() {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function doc_line($label, $method) {
    $doc = $method->getDocComment();
    echo $label, "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

$method = new ReflectionMethod(HookPlugin::class, "register");
$suffix = "tests/fixtures/milestone1496/method_reflection_source_metadata.php";
echo "source|", substr($method->getFileName(), -strlen($suffix)), "|", $method->getStartLine(), "|", $method->getEndLine(), "\n";
doc_line("doc", $method);
$inherited = new ReflectionMethod(HookPlugin::class, "inherited");
doc_line("inherited", $inherited);
echo "inherited-lines|", $inherited->getStartLine(), "|", $inherited->getEndLine(), "\n";
$plain = new ReflectionMethod(HookPlugin::class, "noDoc");
echo "plain|", $plain->getStartLine(), "|", $plain->getEndLine(), "|", yn($plain->getDocComment() === false);
"#,
        "tests/fixtures/milestone1496/method_reflection_source_metadata.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "source|tests/fixtures/milestone1496/method_reflection_source_metadata.php|15|17\ndoc|1|/**\\n     * Registers WordPress hooks.\\n     */\ninherited|1|/**\\n     * Parent hook metadata.\\n     */\ninherited-lines|6|8\nplain|19|19|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_reports_internal_user_reference_and_destructor_metadata() {
    let execution = run_source(
        r#"<?php
class BaseHook {
    public function __destruct() {}
    public function &byRef() {}
}

class ChildHook extends BaseHook {}

class MyReflectionMethod extends ReflectionMethod {}

function yn($value) {
    return $value ? "1" : "0";
}

$destructor = new ReflectionMethod(ChildHook::class, "__destruct");
echo "destruct|", yn($destructor->isDestructor()), "|", yn($destructor->isUserDefined()), "|", yn($destructor->isInternal()), "|", $destructor->getDeclaringClass()->getName(), "\n";

$byRef = ReflectionMethod::createFromMethodName("ChildHook::byRef");
echo "byref|", yn($byRef->returnsReference()), "|", yn($byRef->isUserDefined()), "|", yn($byRef->isInternal()), "|", $byRef->getDeclaringClass()->getName(), "\n";

$subclassed = MyReflectionMethod::createFromMethodName("ChildHook::byRef");
echo "subclass|", get_class($subclassed), "|", yn($subclassed->returnsReference()), "\n";

$internal = new ReflectionMethod(ReflectionProperty::class, "__construct");
echo "internal|", yn($internal->isInternal()), "|", yn($internal->isUserDefined()), "|", yn($internal->getStartLine() === false), yn($internal->getEndLine() === false), yn($internal->getFileName() === false), "\n";

try {
    MyReflectionMethod::createFromMethodName("ChildHook::missing");
} catch (ReflectionException $e) {
    echo "missing|", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "destruct|1|1|0|BaseHook\nbyref|1|1|0|BaseHook\nsubclass|MyReflectionMethod|1\ninternal|1|0|111\nmissing|Method ChildHook::missing() does not exist"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_and_method_report_static_variables() {
    let execution = run_source(
        r#"<?php
function user_statics() {
    static $c;
    static $a = 1, $b = "hello";
}

class Plugin {
    public function boot() {
        static $seen = 2;
        static $empty;
    }

    public static function ping() {
        echo "method-closure-static\n";
    }

    public function label($value = "bound") {
        echo "method-closure-", $value, "\n";
    }
}

function dump_array($label, $values) {
    echo $label, "\n";
    foreach ($values as $key => $value) {
        echo $key, "=", ($value === null ? "NULL" : $value), "\n";
    }
}

function callback_line($value) {
    echo "function-closure-", $value, "\n";
}

dump_array("function", (new ReflectionFunction("user_statics"))->getStaticVariables());
dump_array("method", (new ReflectionMethod(Plugin::class, "boot"))->getStaticVariables());
echo "extract|", (new ReflectionFunction("extract"))->isInternal() ? "internal" : "user", "|", (new ReflectionFunction("extract"))->getStartLine() === false ? "no-line" : "line", "\n";
$functionClosure = (new ReflectionFunction("callback_line"))->getClosure();
$functionClosure("ok");
$staticClosure = (new ReflectionMethod(Plugin::class, "ping"))->getClosure();
$staticClosure();
$methodClosure = (new ReflectionMethod(Plugin::class, "label"))->getClosure(new Plugin());
$methodClosure();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "function\nc=NULL\na=1\nb=hello\nmethod\nseen=2\nempty=NULL\nextract|internal|no-line\nfunction-closure-ok\nmethod-closure-static\nmethod-closure-bound\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_parameter_reports_bounded_method_parameter_metadata() {
    let execution = run_source(
        r#"<?php
class Plugin {
    public function boot(string $hook, &$value = "seed", $count = 3, ...$rest) {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function default_value($parameter) {
    if (!$parameter->isDefaultValueAvailable()) {
        return "-";
    }
    return $parameter->getDefaultValue();
}

function line($label, $parameter, $ending = "\n") {
    echo $label, "|", $parameter->getName(), "|", $parameter->getPosition(), "|", $parameter->getDeclaringClass()->getName(), "|", $parameter->getDeclaringFunction()->getName(), "|", yn($parameter->isOptional()), yn($parameter->isDefaultValueAvailable()), "|", default_value($parameter), "|", yn($parameter->isPassedByReference()), yn($parameter->isVariadic()), yn($parameter->hasType()), $ending;
}

$method = new ReflectionMethod(Plugin::class, "boot");
echo "counts|", $method->getNumberOfParameters(), "|", $method->getNumberOfRequiredParameters(), "\n";
foreach ($method->getParameters() as $index => $parameter) {
    line("param" . $index, $parameter);
}
line("named", new ReflectionParameter(array(Plugin::class, "boot"), "value"));
line("indexed", new ReflectionParameter(array(new Plugin(), "boot"), 2), "");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "counts|4|1\nparam0|hook|0|Plugin|boot|00|-|001\nparam1|value|1|Plugin|boot|11|seed|100\nparam2|count|2|Plugin|boot|11|3|000\nparam3|rest|3|Plugin|boot|10|-|010\nnamed|value|1|Plugin|boot|11|seed|100\nindexed|count|2|Plugin|boot|11|3|000"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_parameter_exposes_name_property_and_to_string_summary() {
    let execution = run_source(
        r#"<?php
function boot($hook, &$value = "seed", ...$rest) {}
$function = new ReflectionFunction("boot");
foreach ($function->getParameters() as $parameter) {
    echo $parameter->name, "|", $parameter, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hook|Parameter #0 [ <required> $hook ]\nvalue|Parameter #1 [ <optional> &$value = 'seed' ]\nrest|Parameter #2 [ <optional> ...$rest ]\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_parameter_reports_bounded_named_type_metadata() {
    let execution = run_source(
        r#"<?php
class Plugin {
    public function boot(string $hook, ?int $count, Plugin $plugin = null, array $items = null, $raw = null) {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $parameter, $ending = "\n") {
    $type = $parameter->getType();
    if ($type === null) {
        echo $label, "|null|", yn($parameter->allowsNull()), $ending;
        return;
    }
    echo $label, "|", get_class($type), "|", $type->getName(), "|", yn($type->allowsNull()), yn($parameter->allowsNull()), yn($type->isBuiltin()), yn($type instanceof ReflectionType), $ending;
}

$method = new ReflectionMethod(Plugin::class, "boot");
foreach ($method->getParameters() as $parameter) {
    line($parameter->getName(), $parameter);
}
line("direct", new ReflectionParameter(array(new Plugin(), "boot"), "count"), "");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hook|ReflectionNamedType|string|0011\ncount|ReflectionNamedType|int|1111\nplugin|ReflectionNamedType|Plugin|1101\nitems|ReflectionNamedType|array|1111\nraw|null|1\ndirect|ReflectionNamedType|int|1111"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_named_type_and_function_objects_stringify() {
    let execution = run_source(
        r#"<?php
class Test {}
function typed(?Traversable $iterator): ?string {}
function object_type(?Test $test): ?Test {}
function described($test, $test2 = null) {}

function type_line($label, $type) {
    echo $label, "|", $type->getName(), "|", (string) $type, "\n";
}

$function = new ReflectionFunction("typed");
type_line("param-internal", $function->getParameters()[0]->getType());
type_line("return-internal", $function->getReturnType());

$function = new ReflectionFunction("object_type");
type_line("param-user", $function->getParameters()[0]->getType());
type_line("return-user", $function->getReturnType());

echo "function-start\n", (new ReflectionFunction("described")), "\nfunction-end";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "param-internal|Traversable|?Traversable\nreturn-internal|string|?string\nparam-user|Test|?Test\nreturn-user|Test|?Test\nfunction-start\nFunction [ <user> function described ] {\n\n  - Parameters [2] {\n    Parameter #0 [ <required> $test ]\n    Parameter #1 [ <optional> $test2 = NULL ]\n  }\n}\n\nfunction-end"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_parameter_reports_default_constant_and_pass_by_value_metadata() {
    let execution = run_source(
        r#"<?php
define("APP_CONST", "app");

class OtherDefaults {
    const VALUE = "other";
}

class Defaults {
    const VALUE = "self";
    public function method($plain = 1, $global = APP_CONST, $self = self::VALUE, $other = OtherDefaults::VALUE) {}
}

function user_params(&$array1, $array2) {}

function yn($value) {
    return $value ? "1" : "0";
}

function constant_line($parameter) {
    echo $parameter->getName(), "|", yn($parameter->isDefaultValueConstant());
    if ($parameter->isDefaultValueConstant()) {
        echo "|", $parameter->getDefaultValueConstantName();
    }
    echo "\n";
}

foreach ((new ReflectionMethod(Defaults::class, "method"))->getParameters() as $parameter) {
    constant_line($parameter);
}

foreach ((new ReflectionFunction("user_params"))->getParameters() as $parameter) {
    echo "user|", $parameter->getName(), "|", yn($parameter->isPassedByReference()), "|", yn($parameter->canBePassedByValue()), "\n";
}
foreach ((new ReflectionFunction("array_multisort"))->getParameters() as $parameter) {
    echo "multi|", $parameter->getName(), "|", yn($parameter->isPassedByReference()), "|", yn($parameter->canBePassedByValue()), "\n";
}
foreach ((new ReflectionFunction("sort"))->getParameters() as $parameter) {
    echo "sort|", $parameter->getName(), "|", yn($parameter->isPassedByReference()), "|", yn($parameter->canBePassedByValue()), "\n";
}

try {
    (new ReflectionFunction("user_params"))->getParameters()[0]->getDefaultValueConstantName();
} catch (ReflectionException $e) {
    echo "missing|", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "plain|0\nglobal|1|APP_CONST\nself|1|self::VALUE\nother|1|OtherDefaults::VALUE\nuser|array1|1|0\nuser|array2|0|1\nmulti|array|1|1\nmulti|rest|1|1\nsort|array|1|0\nsort|flags|0|1\nmissing|Internal error: Failed to retrieve the default value"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_parameter_constructor_reinitializes_and_reports_catchable_errors() {
    let execution = run_source(
        r#"<?php
class EmptyClass {}

$closure = function (int $x): void {};
$parameter = new ReflectionParameter($closure, "x");
echo "first|", $parameter->name, "|", $parameter->getType()->getName(), "\n";
$parameter->__construct("ord", "character");
echo "second|", $parameter->name, "|", $parameter->getType()->getName(), "\n";

try {
    new ReflectionParameter(array("MissingClass", "missing"), 0);
} catch (ReflectionException $e) {
    echo "missing-class|", $e->getMessage(), "\n";
}

try {
    new ReflectionParameter(array("EmptyClass", "missing"), 0);
} catch (ReflectionException $e) {
    echo "missing-method|", $e->getMessage(), "\n";
}

try {
    new ReflectionParameter(array(new EmptyClass, "missing"), 0);
} catch (ReflectionException $e) {
    echo "missing-object-method|", $e->getMessage(), "\n";
}

try {
    new ReflectionParameter(array("EmptyClass", "missing"));
} catch (TypeError $e) {
    echo "arity|", $e->getMessage(), "\n";
}

try {
    new ReflectionParameter(0, 0);
} catch (ReflectionException $e) {
    echo "bad-function|", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "first|x|int\nsecond|character|string\nmissing-class|Class \"MissingClass\" does not exist\nmissing-method|Method EmptyClass::missing() does not exist\nmissing-object-method|Method EmptyClass::missing() does not exist\narity|ReflectionParameter::__construct() expects exactly 2 arguments, 1 given\nbad-function|ReflectionParameter::__construct(): Argument #1 ($function) must be a string, an array(class, method), or a callable object, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_and_parameter_report_bounded_compound_type_metadata() {
    let execution = run_source(
        r#"<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract {}
class TaggedHook extends Hook implements TaggedContract {}
class OtherHook {}

class Plugin {
    public function select(HookContract|OtherHook|null $hook, HookContract&TaggedContract $tagged): HookContract|OtherHook|null {}
    public function tagged(): HookContract&TaggedContract {}
    public function raw($value) {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function type_names($type) {
    $names = array();
    foreach ($type->getTypes() as $inner) {
        $names[] = $inner->getName() . ":" . yn($inner->isBuiltin()) . ":" . yn($inner->allowsNull());
    }
    return implode(",", $names);
}

function line($label, $type, $ending = "\n") {
    if ($type === null) {
        echo $label, "|null", $ending;
        return;
    }
    echo $label, "|", get_class($type), "|", yn($type instanceof ReflectionType), "|", yn($type->allowsNull()), "|", type_names($type), $ending;
}

$method = new ReflectionMethod(Plugin::class, "select");
$params = $method->getParameters();
echo "method|", yn($method->hasReturnType()), "|", $method->getNumberOfParameters(), "\n";
line("return-union", $method->getReturnType());
line("param-union", $params[0]->getType());
line("param-intersection", $params[1]->getType());
line("return-intersection", (new ReflectionMethod(Plugin::class, "tagged"))->getReturnType());
echo "raw|", yn((new ReflectionMethod(Plugin::class, "raw"))->hasReturnType()), "|";
line("raw-return", (new ReflectionMethod(Plugin::class, "raw"))->getReturnType(), "");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "method|1|2\nreturn-union|ReflectionUnionType|1|1|HookContract:0:0,OtherHook:0:0,null:1:1\nparam-union|ReflectionUnionType|1|1|HookContract:0:0,OtherHook:0:0,null:1:1\nparam-intersection|ReflectionIntersectionType|1|0|HookContract:0:0,TaggedContract:0:0\nreturn-intersection|ReflectionIntersectionType|1|0|HookContract:0:0,TaggedContract:0:0\nraw|0|raw-return|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_reports_bounded_user_function_metadata() {
    let execution = run_source(
        r#"<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract, TaggedContract {}

function &select_hook(HookContract|array|null $hook, HookContract&TaggedContract $tagged, $fallback = "seed"): HookContract|array|null {
    return $hook;
}

function raw_hook($value) {}

function yn($value) {
    return $value ? "1" : "0";
}

function type_line($label, $type) {
    if ($type === null) {
        echo $label, "|null\n";
        return;
    }
    echo $label, "|", get_class($type), "|", yn($type->allowsNull()), "\n";
}

function param_line($label, $parameter) {
    $declaringClass = $parameter->getDeclaringClass();
    echo $label, "|", $parameter->getName(), "|", $parameter->getPosition(), "|", get_class($parameter->getDeclaringFunction()), "|", $parameter->getDeclaringFunction()->getName(), "|", yn($declaringClass === null), "|", yn($parameter->isDefaultValueAvailable()), "|", yn($parameter->hasType()), "\n";
}

$function = new ReflectionFunction("select_hook");
echo "fn|", $function->getName(), "|", get_class($function), "|", $function->getNumberOfParameters(), "|", $function->getNumberOfRequiredParameters(), "|", yn($function->hasReturnType()), "|", yn($function->returnsReference()), "\n";
type_line("return", $function->getReturnType());
foreach ($function->getParameters() as $index => $parameter) {
    param_line("param" . $index, $parameter);
}
param_line("direct", new ReflectionParameter("select_hook", "tagged"));
echo "raw|", yn((new ReflectionFunction("raw_hook"))->hasReturnType()), "|";
type_line("raw-return", (new ReflectionFunction("raw_hook"))->getReturnType());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|select_hook|ReflectionFunction|3|2|1|1\nreturn|ReflectionUnionType|1\nparam0|hook|0|ReflectionFunction|select_hook|1|0|1\nparam1|tagged|1|ReflectionFunction|select_hook|1|0|1\nparam2|fallback|2|ReflectionFunction|select_hook|1|1|0\ndirect|tagged|1|ReflectionFunction|select_hook|1|0|1\nraw|0|raw-return|null\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_reports_namespace_identity_and_extension_metadata() {
    let execution = run_source(
        r#"<?php
namespace App\Meta;

function sample() {}

function yn($value) {
    return $value ? "1" : "0";
}

$sort = new \ReflectionFunction("sort");
$user = new \ReflectionFunction("App\\Meta\\sample");
$closure = new \ReflectionFunction(function () {});
$dump = new \ReflectionFunction("var_dump");

echo "sort|", $sort->getName(), "|", $sort->getShortName(), "|", $sort->getNamespaceName(), "|", yn($sort->inNamespace()), "|", yn($sort->isInternal()), "|", yn($sort->isUserDefined()), "|", $sort->getExtensionName(), "\n";
echo "user|", $user->getName(), "|", $user->getShortName(), "|", $user->getNamespaceName(), "|", yn($user->inNamespace()), "|", yn($user->isInternal()), "|", yn($user->isUserDefined()), "|", ($user->getExtensionName() === false ? "false" : "wrong"), "\n";
echo "closure|", yn($closure->isClosure()), "|", yn($closure->isAnonymous()), "|", yn($closure->isInternal()), "\n";
echo "dump|", yn($dump->isDeprecated()), "|", $dump->getExtensionName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "sort|sort|sort||0|1|0|standard\nuser|App\\Meta\\sample|sample|App\\Meta|1|0|1|false\nclosure|1|1|0\ndump|0|standard"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_bounded_extension_owner_metadata() {
    let execution = run_source(
        r#"<?php
class MyClass
{
    public $varX;
    public $varY;
}

$dom = new ReflectionClass('domDocument');
$user = new ReflectionClass('MyClass');
$std = new ReflectionClass('stdClass');

echo method_exists('ReflectionClass', 'getExtensionName') ? "method-name" : "missing-name";
echo "|";
echo method_exists('ReflectionClass', 'getExtension') ? "method-object" : "missing-object";
echo "\n";
var_dump($dom->getExtensionName());
$extension = $dom->getExtension();
echo get_class($extension), "|", $extension->getName(), "\n";
var_dump($user->getExtensionName());
var_dump($user->getExtension());
var_dump($std->getExtensionName());
var_dump($std->getExtension());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "method-name|method-object\n",
            "string(3) \"dom\"\n",
            "ReflectionExtension|dom\n",
            "bool(false)\n",
            "NULL\n",
            "bool(false)\n",
            "NULL\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_get_attributes_reports_declared_targets_and_instances() {
    let execution = run_source(
        r#"<?php
#[Attribute]
class FirstAttr {
    public $value;
    public function __construct($value = "") {
        $this->value = $value;
    }
}

#[Attribute]
class SecondAttr extends FirstAttr {}

#[FirstAttr("class")]
class AttributeSubject {
    #[SecondAttr("property")]
    public $slot = null;

    #[FirstAttr("method")]
    public function run(#[SecondAttr("parameter")] $value) {}
}

#[SecondAttr("function")]
function attributed_helper() {}

$closure = #[FirstAttr("closure")] function () {};

function attr_line($label, $attrs) {
    $attr = $attrs[0];
    $args = $attr->getArguments();
    $instance = $attr->newInstance();
    echo $label, "|", count($attrs), "|", $attr->getName(), "|", $attr->getTarget(), "|", $args[0], "|", $instance->value, "\n";
}

attr_line("class", (new ReflectionClass(AttributeSubject::class))->getAttributes());
attr_line("function", (new ReflectionFunction("attributed_helper"))->getAttributes());
attr_line("method", (new ReflectionMethod(AttributeSubject::class, "run"))->getAttributes());
attr_line("property", (new ReflectionProperty(AttributeSubject::class, "slot"))->getAttributes());
$params = (new ReflectionMethod(AttributeSubject::class, "run"))->getParameters();
attr_line("parameter", $params[0]->getAttributes());
attr_line("closure", (new ReflectionFunction($closure))->getAttributes());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "class|1|FirstAttr|1|class|class\nfunction|1|SecondAttr|2|function|function\nmethod|1|FirstAttr|4|method|method\nproperty|1|SecondAttr|8|property|property\nparameter|1|SecondAttr|32|parameter|parameter\nclosure|1|FirstAttr|2|closure|closure\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_attribute_filters_repeats_named_args_and_core_attribute_metadata() {
    let execution = run_source(
        r#"<?php
#[Attribute]
class BaseAttr {}

#[Attribute]
class ChildAttr extends BaseAttr {
    public $value;
    public $named;
    public function __construct($value = "", $named = "") {
        $this->value = $value;
        $this->named = $named;
    }
}

#[ChildAttr("one", named: "named-one")]
#[ChildAttr("two")]
class FilteredSubject {}

$ref = new ReflectionClass(FilteredSubject::class);
$all = $ref->getAttributes();
$exact = $ref->getAttributes(ChildAttr::class);
$base = $ref->getAttributes(BaseAttr::class, ReflectionAttribute::IS_INSTANCEOF);
$args = $all[0]->getArguments();
$instance = $all[0]->newInstance();
$repeat = $all[0]->isRepeated() ? "1" : "0";
echo "filtered|", count($all), "|", count($exact), "|", count($base), "|", $repeat, "|", $args[0], "|", $args["named"], "|", $instance->named, "\n";

$core = (new ReflectionClass(Attribute::class))->getAttributes();
$coreArgs = $core[0]->getArguments();
echo "core|", count($core), "|", $core[0]->getName(), "|", $core[0]->getTarget(), "|", $coreArgs[0], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "filtered|2|2|2|1|one|named-one|named-one\ncore|1|Attribute|1|1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn builtin_attribute_target_startup_diagnostics_match_php_subset() {
    assert_php_startup_fatal(
        r#"<?php
#[Attribute]
function a1() {}
"#,
        "Command line code",
        3,
        "Attribute \"Attribute\" cannot target function (allowed targets: class)",
    );
    assert_php_startup_fatal(
        r#"<?php
#[Attribute]
abstract class Demo {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\Attribute] to abstract class Demo",
    );
    assert_php_startup_fatal(
        r#"<?php
#[Attribute]
interface Demo {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\Attribute] to interface Demo",
    );
    assert_php_startup_fatal(
        r#"<?php
#[Attribute]
trait Demo {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\Attribute] to trait Demo",
    );
    assert_php_startup_fatal(
        r#"<?php
#[Attribute]
enum Demo {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\Attribute] to enum Demo",
    );
    assert_php_startup_fatal(
        r#"<?php
#[AllowDynamicProperties]
interface Test {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\AllowDynamicProperties] to interface Test",
    );
    assert_php_startup_fatal(
        r#"<?php
#[AllowDynamicProperties]
trait Test {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\AllowDynamicProperties] to trait Test",
    );
    assert_php_startup_fatal(
        r#"<?php
#[AllowDynamicProperties]
enum Test {}
"#,
        "Command line code",
        3,
        "Cannot apply #[\\AllowDynamicProperties] to enum Test",
    );
}

#[test]
fn reflection_function_and_method_invoke_by_value_callbacks() {
    let execution = run_source(
        r#"<?php
function render_hook($tag, $suffix = "done") {
    return $tag . ":" . $suffix;
}

class HookRunner {
    public $log = array();

    public function append($hook, $priority = 10) {
        $this->log[] = $hook . ":" . $priority;
        return count($this->log);
    }
}

$function = new ReflectionFunction("render_hook");
echo $function->invoke("init"), "\n";
echo $function->invokeArgs(array("save_post", "later")), "\n";

$runner = new HookRunner();
$method = new ReflectionMethod(HookRunner::class, "append");
echo $method->invoke($runner, "init"), "|", implode(",", $runner->log), "\n";
echo $method->invokeArgs($runner, array("save_post", 20)), "|", implode(",", $runner->log);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "init:done\nsave_post:later\n1|init:10\n2|init:10,save_post:20"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_invokes_bounded_internal_string_builtins() {
    let execution = run_source(
        r#"<?php
$strlen = new ReflectionFunction("StrLen");
echo "name|", $strlen->getName(), "\n";
echo "file|", ($strlen->getFileName() ? "yes" : "no"), "\n";
echo "start|", ($strlen->getStartLine() ? "yes" : "no"), "\n";
echo "params|", $strlen->getNumberOfParameters() . "/" . $strlen->getNumberOfRequiredParameters(), "\n";
$params = $strlen->getParameters();
$param = $params[0];
echo "param|", $param->getName() . ":" . $param->getType()->getName(), "\n";
echo "return|", $strlen->getReturnType()->getName(), "\n";
echo "invoke|", $strlen->invoke("cache-key"), "\n";
echo "invokeArgs|", $strlen->invokeArgs(array("hook")), "\n";

$lower = new ReflectionFunction("strtolower");
echo "lower|", $lower->invoke("Save_Post"), "\n";
echo "lowerArgs|", $lower->invokeArgs(array("REST_API_INIT"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "name|strlen\nfile|no\nstart|no\nparams|1/1\nparam|string:string\nreturn|int\ninvoke|9\ninvokeArgs|4\nlower|save_post\nlowerArgs|rest_api_init"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_invokes_more_bounded_internal_wordpress_builtins() {
    let execution = run_source(
        r#"<?php
$strpos = new ReflectionFunction("StrPos");
$returnType = $strpos->getReturnType();
$returnNames = array();
foreach ($returnType->getTypes() as $part) {
    $returnNames[] = $part->getName();
}
echo "strpos|", $strpos->getName(), "|", $strpos->getNumberOfParameters(), "/", $strpos->getNumberOfRequiredParameters(), "|", get_class($returnType), ":", implode("|", $returnNames), "|", $strpos->invoke("wp-admin", "admin"), "\n";
$offset = $strpos->getParameters()[2];
echo "offset|", $offset->getName(), "|", ($offset->isOptional() ? "1" : "0"), "|", ($offset->isDefaultValueAvailable() ? "1" : "0"), "|", $offset->getDefaultValue(), "|", $offset->getType()->getName(), "\n";

$substr = new ReflectionFunction("substr");
echo "substr|", $substr->invoke("save_post", 0, 4), "|", $substr->getNumberOfParameters(), "/", $substr->getNumberOfRequiredParameters(), "\n";
echo "trim|", (new ReflectionFunction("trim"))->invoke("  init  "), "|", (new ReflectionFunction("trim"))->getNumberOfParameters(), "/", (new ReflectionFunction("trim"))->getNumberOfRequiredParameters(), "\n";
echo "ltrim|", (new ReflectionFunction("ltrim"))->invoke("  admin"), "\n";
echo "rtrim|", (new ReflectionFunction("rtrim"))->invoke("hook  "), "\n";
$contains = new ReflectionFunction("str_contains");
echo "contains|", ($contains->invokeArgs(array("wp-admin/includes", "admin")) ? "1" : "0"), "|", $contains->getReturnType()->getName(), "\n";
echo "starts|", ((new ReflectionFunction("str_starts_with"))->invoke("rest_api_init", "rest") ? "1" : "0"), "\n";
echo "ends|", ((new ReflectionFunction("str_ends_with"))->invoke("template_redirect", "redirect") ? "1" : "0"), "\n";
echo "case|", (new ReflectionFunction("strcasecmp"))->invoke("REST", "rest"), "\n";
echo "path|", (new ReflectionFunction("basename"))->invoke("/var/www/wp-config.php", ".php"), "|", (new ReflectionFunction("dirname"))->invoke("/var/www/wp-content/plugins", 2), "\n";
echo "format|", (new ReflectionFunction("sprintf"))->invoke("hook:%s:%d", "init", 10), "\n";
$sprintfValues = (new ReflectionFunction("sprintf"))->getParameters()[1];
echo "variadic|", $sprintfValues->getName(), "|", ($sprintfValues->isVariadic() ? "1" : "0"), "|", ($sprintfValues->isOptional() ? "1" : "0"), "|", ($sprintfValues->isDefaultValueAvailable() ? "1" : "0"), "\n";
echo "implode|", (new ReflectionFunction("implode"))->invokeArgs(array("-", array("mu", "plugin"))), "\n";
echo "defined|", ((new ReflectionFunction("defined"))->invoke("PHP_VERSION") ? "1" : "0"), "\n";
echo "function|", ((new ReflectionFunction("function_exists"))->invoke("str_contains") ? "1" : "0"), "\n";
$sapi = new ReflectionFunction("php_sapi_name");
echo "sapi|", $sapi->getNumberOfParameters(), "/", $sapi->getNumberOfRequiredParameters(), "|", $sapi->invoke();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "strpos|strpos|3/2|ReflectionUnionType:int|false|3\noffset|offset|1|1|0|int\nsubstr|save|3/2\ntrim|init|2/1\nltrim|admin\nrtrim|hook\ncontains|1|bool\nstarts|1\nends|1\ncase|0\npath|wp-config|/var/www\nformat|hook:init:10\nvariadic|values|1|1|0\nimplode|mu-plugin\ndefined|1\nfunction|1\nsapi|0/0|cli"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_invokes_type_count_and_callable_wordpress_builtins() {
    let execution = run_source(
        r#"<?php
function type_name($parameter) {
    $type = $parameter->getType();
    if ($type === null) {
        return "";
    }
    if ($type instanceof ReflectionUnionType) {
        $names = array();
        foreach ($type->getTypes() as $part) {
            $names[] = $part->getName();
        }
        return implode("|", $names);
    }
    return $type->getName();
}

function param_line($label, $parameter) {
    echo $label, "|", $parameter->getName(), "|", type_name($parameter), "|", ($parameter->isOptional() ? "1" : "0"), "|", ($parameter->isDefaultValueAvailable() ? "1" : "0"), "|", ($parameter->isPassedByReference() ? "1" : "0");
    if ($parameter->isDefaultValueAvailable()) {
        $default = $parameter->getDefaultValue();
        echo "|", $default === null ? "null" : ($default ? "true" : "false");
    }
    echo "\n";
}

$isArray = new ReflectionFunction("is_array");
echo "is_array|", $isArray->getNumberOfParameters(), "/", $isArray->getNumberOfRequiredParameters(), "|", $isArray->getReturnType()->getName(), "|", ($isArray->invoke(array("hook")) ? "1" : "0"), "|", ($isArray->invoke("hook") ? "1" : "0"), "\n";
param_line("is_array:param0", $isArray->getParameters()[0]);

$isObject = new ReflectionFunction("is_object");
echo "is_object|", ($isObject->invoke(new stdClass()) ? "1" : "0"), "|", ($isObject->invoke(array()) ? "1" : "0"), "\n";

$isString = new ReflectionFunction("is_string");
echo "is_string|", ($isString->invoke("save_post") ? "1" : "0"), "|", ($isString->invoke(42) ? "1" : "0"), "\n";

$isScalar = new ReflectionFunction("is_scalar");
echo "is_scalar|", ($isScalar->invoke("save_post") ? "1" : "0"), "|", ($isScalar->invoke(array()) ? "1" : "0"), "\n";

$count = new ReflectionFunction("count");
echo "count|", $count->getNumberOfParameters(), "/", $count->getNumberOfRequiredParameters(), "|", $count->getReturnType()->getName(), "|", $count->invoke(array("a", "b")), "\n";
param_line("count:param0", $count->getParameters()[0]);
param_line("count:param1", $count->getParameters()[1]);

$exists = new ReflectionFunction("array_key_exists");
echo "exists|", $exists->getNumberOfParameters(), "/", $exists->getNumberOfRequiredParameters(), "|", ($exists->invoke("hook", array("hook" => "init")) ? "1" : "0"), "|", ($exists->invoke("missing", array("hook" => "init")) ? "1" : "0"), "\n";
param_line("exists:param0", $exists->getParameters()[0]);
param_line("exists:param1", $exists->getParameters()[1]);

$callable = new ReflectionFunction("is_callable");
echo "callable|", $callable->getNumberOfParameters(), "/", $callable->getNumberOfRequiredParameters(), "|", ($callable->invoke("strlen") ? "1" : "0"), "|", ($callable->invoke("missing_function") ? "1" : "0"), "|", ($callable->invoke("Class::method", true) ? "1" : "0"), "\n";
param_line("callable:param1", $callable->getParameters()[1]);
$callableName = $callable->getParameters()[2];
echo "callable:param2|", $callableName->getName(), "|", type_name($callableName), "|", ($callableName->isOptional() ? "1" : "0"), "|", ($callableName->isDefaultValueAvailable() ? "1" : "0"), "|", ($callableName->isPassedByReference() ? "1" : "0"), "|", ($callableName->getDefaultValue() === null ? "null" : "value");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "is_array|1/1|bool|1|0\nis_array:param0|value|mixed|0|0|0\nis_object|1|0\nis_string|1|0\nis_scalar|1|0\ncount|2/1|int|2\ncount:param0|value|Countable|array|0|0|0\ncount:param1|mode|int|1|1|0|false\nexists|2/2|1|0\nexists:param0|key||0|0|0\nexists:param1|array|array|0|0|0\ncallable|3/1|1|0|1\ncallable:param1|syntax_only|bool|1|1|0|false\ncallable:param2|callable_name||1|1|1|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_reflects_bounded_closure_metadata() {
    let execution = run_source_with_source_file(
        r#"<?php
$callback = function (string $hook, $priority = 10): string { return $hook; };
$function = new ReflectionFunction($callback);
$suffix = "tests/fixtures/milestone1587/closure_reflection_metadata.php";
echo "fn|", $function->getName(), "|", get_class($function), "|", $function->getNumberOfParameters(), "/", $function->getNumberOfRequiredParameters(), "|", ($function->returnsReference() ? "1" : "0"), "|", ($function->hasReturnType() ? "1" : "0"), "|", $function->getReturnType()->getName(), "\n";
echo "source|", substr($function->getFileName(), -strlen($suffix)), "|", $function->getStartLine(), "|", $function->getEndLine(), "|", ($function->getDocComment() === false ? "1" : "0"), "\n";
foreach ($function->getParameters() as $index => $parameter) {
    $type = $parameter->getType();
    $declaring = $parameter->getDeclaringFunction();
    echo "param", $index, "|", $parameter->getName(), "|", ($parameter->isOptional() ? "1" : "0"), "|", ($parameter->isDefaultValueAvailable() ? "1" : "0"), "|", ($parameter->isDefaultValueAvailable() ? $parameter->getDefaultValue() : ""), "|", ($type ? $type->getName() : ""), "|", $declaring->getName(), "\n";
}

$arrow = fn($value): int => 42;
$arrowReflection = new ReflectionFunction($arrow);
echo "arrow|", $arrowReflection->getName(), "|", $arrowReflection->getNumberOfParameters(), "/", $arrowReflection->getNumberOfRequiredParameters(), "|", $arrowReflection->getReturnType()->getName(), "|", $arrowReflection->getStartLine(), "|", $arrowReflection->getEndLine();
"#,
        "tests/fixtures/milestone1587/closure_reflection_metadata.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|{closure}|ReflectionFunction|2/1|0|1|string\nsource|tests/fixtures/milestone1587/closure_reflection_metadata.php|2|2|1\nparam0|hook|0|0||string|{closure}\nparam1|priority|1|1|10||{closure}\narrow|{closure}|1/1|int|13|13"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_invokes_bounded_closure_callbacks() {
    let execution = run_source(
        r#"<?php
$prefix = "wp";
$callback = function ($hook, $priority = 10) use ($prefix) {
    return $prefix . ":" . $hook . ":" . $priority;
};
$prefix = "changed";
$function = new ReflectionFunction($callback);
echo $function->invoke("init"), "\n";
echo $function->invokeArgs(array("save_post", 20));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "wp:init:10\nwp:save_post:20");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_invokes_by_reference_closure_captures() {
    let execution = run_source(
        r#"<?php
$counter = 10;
$callback = function ($hook) use (&$counter) {
    $counter = $counter + 1;
    return $hook . ":" . $counter;
};
$counter = 20;
$function = new ReflectionFunction($callback);
echo $function->invoke("init"), "|", $counter, "\n";
$counter = 30;
echo $function->invokeArgs(array("save_post")), "|", $counter, "\n";

function make_reflected_counter() {
    $local = "start";
    $callback = function ($next) use (&$local) {
        $local = $next;
        return $local;
    };
    return new ReflectionFunction($callback);
}

$reflected = make_reflected_counter();
echo $reflected->invoke("kept"), "\n";
echo $reflected->invokeArgs(array("updated"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "init:21|21\nsave_post:31|31\nkept\nupdated"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_invokes_public_static_methods() {
    let execution = run_source(
        r#"<?php
class BaseHook {
    public static function attach($hook, $priority = 10) {
        return static::class . ":" . $hook . ":" . $priority;
    }
}

class ChildHook extends BaseHook {}

$base = new ReflectionMethod(BaseHook::class, "attach");
echo $base->invoke(null, "init"), "\n";
echo $base->invokeArgs(new BaseHook(), array("save_post", 20)), "\n";

$child = new ReflectionMethod(ChildHook::class, "attach");
echo $child->invoke(null, "plugins_loaded"), "\n";
echo $child->invokeArgs(new ChildHook(), array("shutdown", 50));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "BaseHook:init:10\nBaseHook:save_post:20\nChildHook:plugins_loaded:10\nChildHook:shutdown:50"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_invokes_static_trait_methods() {
    let execution = run_source(
        r#"<?php
trait HookTools {
    public static function tag($hook, $priority = 10) {
        return "trait:" . $hook . ":" . $priority;
    }
}

class Plugin {
    use HookTools;
}

$method = new ReflectionMethod(HookTools::class, "tag");
echo $method->invoke(null, "init"), "\n";
echo $method->invokeArgs(new Plugin(), array("save_post", 20));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "trait:init:10\ntrait:save_post:20");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_rejects_abstract_method_invocation() {
    let interface_error = runtime_error(
        r#"<?php
interface HookContract {
    public function register($hook);
}

class HookPlugin implements HookContract {
    public function register($hook) {
        return "plugin:" . $hook;
    }
}

$method = new ReflectionMethod(HookContract::class, "register");
echo $method->invoke(new HookPlugin(), "init");
"#,
    );
    assert_eq!(
        interface_error.message,
        "unsupported call ReflectionMethod::invoke: trying to invoke abstract method HookContract::register(); PHP raises ReflectionException, exact ReflectionException objects are not implemented"
    );

    let abstract_class_error = runtime_error(
        r#"<?php
abstract class BaseHook {
    abstract public function register($hook);
}

class HookPlugin extends BaseHook {
    public function register($hook) {
        return "plugin:" . $hook;
    }
}

$method = new ReflectionMethod(BaseHook::class, "register");
echo $method->invokeArgs(new HookPlugin(), array("save_post"));
"#,
    );
    assert_eq!(
        abstract_class_error.message,
        "unsupported call ReflectionMethod::invoke: trying to invoke abstract method BaseHook::register(); PHP raises ReflectionException, exact ReflectionException objects are not implemented"
    );
}

#[test]
fn reflection_static_trait_methods_bind_trait_context() {
    let execution = run_source(
        r#"<?php
trait HookTools {
    public static function context($hook) {
        return __CLASS__ . "|" . __METHOD__ . "|" . self::class . "|" . static::class . "|" . get_called_class() . "|" . $hook;
    }
}

class Plugin {
    use HookTools;
}

$method = new ReflectionMethod(HookTools::class, "context");
echo $method->invoke(null, "init"), "\n";
echo $method->invokeArgs(new Plugin(), array("save_post"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "HookTools|HookTools::context|HookTools|HookTools|HookTools|init\nHookTools|HookTools::context|HookTools|HookTools|HookTools|save_post"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_static_trait_methods_call_self_and_static_trait_methods() {
    let execution = run_source(
        r#"<?php
trait HookTools {
    public static function label($hook) {
        return static::class . ":" . __METHOD__ . ":" . $hook;
    }

    public static function relay($hook) {
        return self::label($hook) . "|" . static::label($hook);
    }
}

class Plugin {
    use HookTools;
}

$method = new ReflectionMethod(HookTools::class, "relay");
echo $method->invoke(null, "init"), "\n";
echo $method->invokeArgs(new Plugin(), array("save_post"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "HookTools:HookTools::label:init|HookTools:HookTools::label:init\nHookTools:HookTools::label:save_post|HookTools:HookTools::label:save_post"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_method_invokes_non_public_user_class_methods() {
    let execution = run_source(
        r#"<?php
class BaseHook {
    private function privateTag($hook) {
        $this->log[] = "private:" . $hook . ":" . static::class;
        return count($this->log);
    }

    protected function protectedTag($hook, $priority = 10) {
        $this->log[] = "protected:" . $hook . ":" . $priority . ":" . static::class;
        return count($this->log);
    }

    protected static function staticTag($hook) {
        return "static:" . $hook . ":" . static::class;
    }
}

class ChildHook extends BaseHook {
    public $log = array();
}

$child = new ChildHook();

$private = new ReflectionMethod(BaseHook::class, "privateTag");
echo $private->invoke($child, "init"), "|", implode(",", $child->log), "\n";

$protected = new ReflectionMethod(BaseHook::class, "protectedTag");
echo $protected->invokeArgs($child, array("save_post", 20)), "|", implode(",", $child->log), "\n";

$static = new ReflectionMethod(ChildHook::class, "staticTag");
echo $static->invoke(null, "shutdown");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|private:init:ChildHook\n2|private:init:ChildHook,protected:save_post:20:ChildHook\nstatic:shutdown:ChildHook"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_reports_bounded_source_metadata() {
    let execution = run_source_with_source_file(
        r#"<?php
function helper_before() {}

/**
 * WordPress-style callback metadata.
 */
function reflected_callback($hook) {
    return $hook;
}

function no_doc_comment() {}

function yn($value) {
    return $value ? "1" : "0";
}

function doc_line($label, $function) {
    $doc = $function->getDocComment();
    echo $label, "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

$function = new ReflectionFunction("reflected_callback");
$suffix = "tests/fixtures/milestone1491/function_reflection_source_metadata.php";
echo "source|", substr($function->getFileName(), -strlen($suffix)), "|", $function->getStartLine(), "|", $function->getEndLine(), "\n";
doc_line("doc", $function);
$plain = new ReflectionFunction("no_doc_comment");
echo "plain|", $plain->getStartLine(), "|", $plain->getEndLine(), "|", yn($plain->getDocComment() === false);
"#,
        "tests/fixtures/milestone1491/function_reflection_source_metadata.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "source|tests/fixtures/milestone1491/function_reflection_source_metadata.php|7|9\ndoc|1|/**\\n * WordPress-style callback metadata.\\n */\nplain|11|11|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_property_reports_bounded_class_property_metadata() {
    let execution = run_source(
        r#"<?php
class Base {
    public $id = "base";
    protected static $cache = "warm";
    private $secret = "hidden";
}

class Plugin extends Base {
    public $name = "hook";
    protected $items = array("a" => 1);
    private static $flag = true;
}

function yn($value) {
    return $value ? "1" : "0";
}

function default_label($value) {
    if (is_array($value)) {
        return "array:" . count($value);
    }
    if (is_bool($value)) {
        return yn($value);
    }
    if ($value === null) {
        return "null";
    }
    return $value;
}

function line($label, $property, $ending = "\n") {
    echo $label, "|", get_class($property), "|", $property->getName(), "|", $property->getDeclaringClass()->getName(), "|", $property->getModifiers(), "|", yn($property->isPublic()), yn($property->isProtected()), yn($property->isPrivate()), yn($property->isStatic()), "|", yn($property->hasDefaultValue()), "|", default_label($property->getDefaultValue()), "|", yn($property->hasType()), yn($property->getType() === null), $ending;
}

$rc = new ReflectionClass(Plugin::class);
echo "constants|", ReflectionProperty::IS_PUBLIC, "|", ReflectionProperty::IS_PROTECTED, "|", ReflectionProperty::IS_PRIVATE, "|", ReflectionProperty::IS_STATIC, "\n";
echo "has|", yn($rc->hasProperty("items")), yn($rc->hasProperty("secret")), "\n";
line("direct", new ReflectionProperty(Plugin::class, "name"));
line("object", new ReflectionProperty(new Plugin(), "cache"));
line("get", $rc->getProperty("flag"));
foreach ($rc->getProperties() as $property) {
    line("list", $property);
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "constants|1|2|4|16\nhas|10\ndirect|ReflectionProperty|name|Plugin|1|1000|1|hook|01\nobject|ReflectionProperty|cache|Base|18|0101|1|warm|01\nget|ReflectionProperty|flag|Plugin|20|0011|1|1|01\nlist|ReflectionProperty|name|Plugin|1|1000|1|hook|01\nlist|ReflectionProperty|items|Plugin|2|0100|1|array:1|01\nlist|ReflectionProperty|flag|Plugin|20|0011|1|1|01\nlist|ReflectionProperty|id|Base|1|1000|1|base|01\nlist|ReflectionProperty|cache|Base|18|0101|1|warm|01\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_property_reports_mangled_name_default_state_and_to_string() {
    let execution = run_source(
        r#"<?php
class Packet {
    public $id;
    protected $payload = 4;
    private static $secret = "seal";
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($property) {
    echo $property->getName(), "|", $property->class, "|", $property->getMangledName(), "|", yn($property->isDefault()), yn($property->isDynamic()), "|", $property->__toString();
}

line(new ReflectionProperty(Packet::class, "id"));
line(new ReflectionProperty(Packet::class, "payload"));
line(new ReflectionProperty(Packet::class, "secret"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "id|Packet|id|10|Property [ public $id = NULL ]\npayload|Packet|\0*\0payload|10|Property [ protected $payload = 4 ]\nsecret|Packet|\0Packet\0secret|10|Property [ private static $secret = 'seal' ]\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_constant_exposes_name_class_and_deprecated_flag() {
    let execution = run_source(
        r#"<?php
class Packet {
    #[Deprecated]
    public const LEGACY = "old";
    protected const CURRENT = "new";
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($name) {
    $constant = new ReflectionClassConstant(Packet::class, $name);
    echo $constant->name, "|", $constant->class, "|", $constant->getName(), "|", yn($constant->isDeprecated()), "\n";
}

line("LEGACY");
line("CURRENT");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "LEGACY|Packet|LEGACY|1\nCURRENT|Packet|CURRENT|0\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_property_reports_bounded_typed_property_metadata() {
    let execution = run_source(
        r#"<?php
class Base {
    public string $id = "base";
    protected static ?string $cache = null;
}

class Plugin extends Base {
    public ?string $name = null;
    protected array $items = array("a" => 1);
    private static bool $flag = true;
    public ?Plugin $peer = null;
}

function yn($value) {
    return $value ? "1" : "0";
}

function default_label($value) {
    if (is_array($value)) {
        return "array:" . count($value);
    }
    if (is_bool($value)) {
        return yn($value);
    }
    if ($value === null) {
        return "null";
    }
    return $value;
}

function type_label($property) {
    $type = $property->getType();
    if ($type === null) {
        return "none";
    }
    return get_class($type) . ":" . $type->getName() . ":" . yn($type->allowsNull()) . yn($type->isBuiltin()) . yn($type instanceof ReflectionType);
}

function line($label, $property, $ending = "\n") {
    echo $label, "|", $property->getName(), "|", yn($property->hasType()), "|", type_label($property), "|", yn($property->hasDefaultValue()), "|", default_label($property->getDefaultValue()), $ending;
}

$rc = new ReflectionClass(Plugin::class);
line("direct", new ReflectionProperty(Plugin::class, "name"));
line("object", new ReflectionProperty(new Plugin(), "cache"));
line("get", $rc->getProperty("flag"));
$properties = $rc->getProperties();
$count = count($properties);
foreach ($properties as $index => $property) {
    line("list", $property, $index + 1 === $count ? "" : "\n");
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct|name|1|ReflectionNamedType:string:111|1|null\nobject|cache|1|ReflectionNamedType:string:111|1|null\nget|flag|1|ReflectionNamedType:bool:011|1|1\nlist|name|1|ReflectionNamedType:string:111|1|null\nlist|items|1|ReflectionNamedType:array:011|1|array:1\nlist|flag|1|ReflectionNamedType:bool:011|1|1\nlist|peer|1|ReflectionNamedType:Plugin:101|1|null\nlist|id|1|ReflectionNamedType:string:011|1|base\nlist|cache|1|ReflectionNamedType:string:111|1|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_property_reports_bounded_doc_comments() {
    let execution = run_source_with_source_file(
        r#"<?php
class Base {
    /**
     * Shared cache metadata.
     */
    protected static $cache = "warm";
}

class Plugin extends Base {
    /**
     * Public hook name metadata.
     */
    public $name = "hook";

    public $plain = "none";
}

function yn($value) {
    return $value ? "1" : "0";
}

function doc_line($label, $property) {
    $doc = $property->getDocComment();
    echo $label, "|", $property->getName(), "|", $property->getDeclaringClass()->getName(), "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

doc_line("direct", new ReflectionProperty(Plugin::class, "name"));
doc_line("inherited", new ReflectionProperty(Plugin::class, "cache"));
$plain = new ReflectionProperty(Plugin::class, "plain");
echo "plain|", $plain->getName(), "|", yn($plain->getDocComment() === false);
"#,
        "tests/fixtures/milestone1506/property_reflection_doc_comments.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct|name|Plugin|1|/**\\n     * Public hook name metadata.\\n     */\ninherited|cache|Base|1|/**\\n     * Shared cache metadata.\\n     */\nplain|plain|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_property_get_value_and_set_value_mutate_public_declared_properties() {
    let execution = run_source(
        r#"<?php
class Base {
    public $base = "base";
    public static $counter = 1;
}

class Plugin extends Base {
    public string $name = "hook";
    public array $log = array("start");
}

function label($value) {
    if (is_array($value)) {
        return "array:" . count($value);
    }
    if ($value === null) {
        return "null";
    }
    return $value;
}

$plugin = new Plugin();
$name = new ReflectionProperty(Plugin::class, "name");
echo "name|get|", $name->getValue($plugin), "|", $plugin->name, "\n";
$name->setValue($plugin, "save");
echo "name|set|", $name->getValue($plugin), "|", $plugin->name, "\n";
$name->setValue($plugin, 123);
echo "name|coerce|", gettype($plugin->name), ":", $plugin->name, "\n";

$base = new ReflectionProperty(Plugin::class, "base");
$base->setValue($plugin, "inherited");
echo "base|", $base->getDeclaringClass()->getName(), "|", $base->getValue($plugin), "|", $plugin->base, "\n";

$log = new ReflectionProperty(Plugin::class, "log");
$log->setValue($plugin, array("first", "second"));
echo "log|", label($log->getValue($plugin)), "|", count($plugin->log), "\n";

$static = new ReflectionProperty(Base::class, "counter");
echo "static|get|", $static->getValue(), "|", Base::$counter, "\n";
$static->setValue(41);
echo "static|set1|", $static->getValue(null), "|", Base::$counter, "\n";
$static->setValue($plugin, 42);
echo "static|set2|", $static->getValue($plugin), "|", Base::$counter;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "name|get|hook|hook\nname|set|save|save\nname|coerce|string:123\nbase|Base|inherited|inherited\nlog|array:2|2\nstatic|get|1|1\nstatic|set1|41|41\nstatic|set2|42|42"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn typed_properties_track_uninitialized_slots_and_enforce_simple_writes() {
    let execution = run_source(
        r#"<?php
class Peer {}
class ChildPeer extends Peer {}
class Box {
    public int $id;
    public ?string $name;
    public float $ratio;
    public Peer $peer;
    public static bool $ready;
    public static ?string $label;
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $property) {
    echo $label, "|", yn($property->hasDefaultValue()), "|", yn($property->hasType()), "|", ($property->getDefaultValue() === null ? "null" : "value"), "\n";
}

$box = new Box();
echo "isset-empty|", yn(isset($box->id)), yn(empty($box->id)), yn(isset(Box::$ready)), yn(empty(Box::$ready)), "\n";
line("prop", new ReflectionProperty(Box::class, "id"));
line("static", new ReflectionProperty(Box::class, "ready"));

$box->id = 42;
$box->name = null;
$box->ratio = 2;
$box->peer = new Peer();
Box::$ready = true;
Box::$label = null;
echo "values|", $box->id, "|", ($box->name === null ? "null" : $box->name), "|", $box->ratio, "|", get_class($box->peer), "|", yn(Box::$ready), "|", (Box::$label === null ? "null" : Box::$label), "\n";

$box->id = "42";
$box->ratio = "4.5";
$box->name = 123;
Box::$ready = "0";
Box::$label = false;
echo "coerced|", gettype($box->id), ":", $box->id, "|", gettype($box->ratio), ":", $box->ratio, "|", gettype($box->name), ":", $box->name, "|", gettype(Box::$ready), ":", yn(Box::$ready), "|", gettype(Box::$label), ":", (Box::$label === "" ? "empty" : Box::$label), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "isset-empty|0101\nprop|0|1|null\nstatic|0|1|null\nvalues|42|null|2|Peer|1|null\ncoerced|integer:42|double:4.5|string:123|boolean:0|string:empty\n"
    );
    assert_eq!(execution.exit_code, 0);

    let read_error = runtime_error(
        r#"<?php
class Box { public int $id; }
$box = new Box();
echo $box->id;
"#,
    );
    assert_eq!(read_error.line, 4);
    assert_eq!(read_error.column, 1);
    assert_eq!(
        read_error.message,
        "Typed property Box::$id must not be accessed before initialization"
    );

    let static_read_error = runtime_error(
        r#"<?php
class Box { public static int $id; }
echo Box::$id;
"#,
    );
    assert_eq!(static_read_error.line, 3);
    assert_eq!(static_read_error.column, 1);
    assert_eq!(
        static_read_error.message,
        "Typed property Box::$id must not be accessed before initialization"
    );

    let write_error = runtime_error(
        r#"<?php
class Box { public int $id; }
$box = new Box();
$box->id = "nope";
"#,
    );
    assert_eq!(write_error.line, 4);
    assert_eq!(write_error.column, 1);
    assert_eq!(
        write_error.message,
        "Cannot assign string to property Box::$id of type int"
    );

    let static_write_error = runtime_error(
        r#"<?php
class Box { public static bool $ready; }
Box::$ready = array();
"#,
    );
    assert_eq!(static_write_error.line, 3);
    assert_eq!(static_write_error.column, 1);
    assert_eq!(
        static_write_error.message,
        "Cannot assign array to property Box::$ready of type bool"
    );
}

#[test]
fn typed_property_reference_alias_writes_keep_property_enforcement() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id;
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;
$alias = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($alias), ":", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id;
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;
$alias = "bad";
"#,
    );
    assert_eq!(error.line, 9);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign string to reference held by property Box::$id of type int"
    );
}

#[test]
fn typed_property_reference_array_slot_writes_keep_property_enforcement() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id;
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;
$target = array();
$target["copy"] =& $alias;
$target["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($target["copy"]), ":", $target["copy"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id;
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;
$target = array();
$target["copy"] =& $alias;
$target["copy"] = "bad";
"#,
    );
    assert_eq!(error.line, 11);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign string to reference held by property Box::$id of type int"
    );
}

#[test]
fn typed_property_reference_alias_slot_variable_writes_keep_property_enforcement() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id;
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;
$target = array();
$target["copy"] =& $alias;
$slot =& $target["copy"];
$slot = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($slot), ":", $slot;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id;
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;
$target = array();
$target["copy"] =& $alias;
$slot =& $target["copy"];
$slot = "bad";
"#,
    );
    assert_eq!(error.line, 12);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign string to reference held by property Box::$id of type int"
    );
}

#[test]
fn typed_property_reference_arrayaccess_backing_writes_keep_property_enforcement() {
    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $bag;
}

$box = new Box();
$alias =& $box->id;
$bag = new Bag();
$bag->items["outer"] = array();
$bag->items["outer"]["copy"] =& $alias;
$holder = new Holder();
$holder->bag = $bag;
$holder->bag["outer"]["copy"] = array("bad");
"#,
    );
    assert_eq!(error.line, 30);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn scalar_nested_arrayaccess_writes_keep_typed_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$bag = new Bag();
$bag->items["outer"] = array();
$bag->items["outer"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$bag = new Bag();
$bag->items["outer"] = array();
$bag->items["outer"]["copy"] =& $alias;
$bag["outer"]["copy"] = array("bad");
"#,
    );
    assert_eq!(error.line, 24);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn scalar_arrayaccess_nested_appends_support_suffix_payloads() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $bag;
}

class MagicHolder {
    public $bag;
    public function __get($name) { return $this->bag; }
}

$bag = new Bag();
$bag->items["outer"] = array();
$bag["outer"][] = "direct";
$holder = new Holder();
$holder->bag = $bag;
$holder->bag["outer"][] = "property";
$holder->bag["outer"][]["leaf"] = "suffix";
$magic = new MagicHolder();
$magic->bag = $bag;
$magic->missing["outer"][] = "magic";
$magic->missing["outer"][]["leaf"] = "magic-suffix";
echo $bag->items["outer"][0],
    "|",
    $bag->items["outer"][1],
    "|",
    $bag->items["outer"][2]["leaf"],
    "|",
    $bag->items["outer"][3],
    "|",
    $bag->items["outer"][4]["leaf"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct|property|suffix|magic|magic-suffix"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn local_indexed_backing_alias_returns_preserve_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $bucket =& $this->items[$offset];
        return $bucket["leaf"];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class MagicBox {
    private $store = array();

    public function seed(&$value) {
        $this->store["missing"]["copy"] =& $value;
    }

    public function &__get($name) {
        return $this->store[$name];
    }

    public function read($name, $key) {
        return gettype($this->store[$name][$key]) . ":" . $this->store[$name][$key];
    }
}

$box = new Box();
$alias =& $box->id;

$bag = new Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["leaf"]["copy"]), ":", $bag->items["outer"]["leaf"]["copy"], "\n";

$magic = new MagicBox();
$magic->seed($alias);
$magic->missing["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", $magic->read("missing", "copy");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2\ninteger:3|integer:3");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $bucket =& $this->items[$offset];
        return $bucket["leaf"];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$bag = new Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = array("bad");
"#,
    );
    assert_eq!(error.line, 26);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );

    let magic_error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $bucket =& $this->items[$offset];
        return $bucket["leaf"];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class MagicBox {
    private $store = array();

    public function seed(&$value) {
        $this->store["missing"]["copy"] =& $value;
    }

    public function &__get($name) {
        return $this->store[$name];
    }
}

$box = new Box();
$alias =& $box->id;
$bag = new Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
$magic = new MagicBox();
$magic->seed($alias);
$magic->missing["copy"] = array("bad");
"#,
    );
    assert_eq!(magic_error.line, 41);
    assert_eq!(magic_error.column, 1);
    assert_eq!(
        magic_error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn local_literal_backing_keys_preserve_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $leaf = "leaf";
        $bucket =& $this->items[$offset];
        return $bucket[$leaf];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class MagicBox {
    private $store = array();

    public function seed(&$value) {
        $this->store["missing"]["leaf"]["copy"] =& $value;
    }

    public function &__get($name) {
        $leaf = "leaf";
        $bucket =& $this->store[$name];
        return $bucket[$leaf];
    }

    public function read($name, $key) {
        return gettype($this->store[$name]["leaf"][$key]) . ":" . $this->store[$name]["leaf"][$key];
    }
}

$box = new Box();
$alias =& $box->id;

$bag = new Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["leaf"]["copy"]), ":", $bag->items["outer"]["leaf"]["copy"], "\n";

$magic = new MagicBox();
$magic->seed($alias);
$magic->missing["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", $magic->read("missing", "copy");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2\ninteger:3|integer:3");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $leaf = "leaf";
        $bucket =& $this->items[$offset];
        return $bucket[$leaf];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$bag = new Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = array("bad");
"#,
    );
    assert_eq!(error.line, 27);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn offset_set_local_literal_keys_preserve_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id = 1;
}

class KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $leaf = "leaf";
        $this->items[$offset][$leaf] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class AppendBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $bucket = "bucket";
        $leaf = "leaf";
        if ($offset === null) {
            $this->items[$bucket][][$leaf] = $value;
            return;
        }
        $this->items[$bucket][$offset][$leaf] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;

$keyed = new KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($keyed->items["outer"]["leaf"]["copy"]), ":", $keyed->items["outer"]["leaf"]["copy"], "\n";

$append = new AppendBag();
$append[] = array("copy" => &$alias);
$append->items["bucket"][0]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($append->items["bucket"][0]["leaf"]["copy"]), ":", $append->items["bucket"][0]["leaf"]["copy"], "\n";

$append["named"] = array("copy" => &$alias);
$append->items["bucket"]["named"]["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($append->items["bucket"]["named"]["leaf"]["copy"]), ":", $append->items["bucket"]["named"]["leaf"]["copy"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "integer:2|integer:2\ninteger:3|integer:3\ninteger:4|integer:4"
    );
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $leaf = "leaf";
        $this->items[$offset][$leaf] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$keyed = new KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = array("bad");
"#,
    );
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn offset_set_local_offset_aliases_preserve_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id = 1;
}

class KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $slot = $offset;
        $leaf = "leaf";
        $this->items[$slot][$leaf] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class BranchBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $slot = $offset;
        $bucket = "bucket";
        $leaf = "leaf";
        if ($offset === null) {
            $this->items[$bucket][][$leaf] = $value;
            return;
        }
        $this->items[$bucket][$slot][$leaf] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;

$keyed = new KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($keyed->items["outer"]["leaf"]["copy"]), ":", $keyed->items["outer"]["leaf"]["copy"], "\n";

$branch = new BranchBag();
$branch["named"] = array("copy" => &$alias);
$branch->items["bucket"]["named"]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"]["named"]["leaf"]["copy"]), ":", $branch->items["bucket"]["named"]["leaf"]["copy"], "\n";

$branch[] = array("copy" => &$alias);
$branch->items["bucket"][0]["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"][0]["leaf"]["copy"]), ":", $branch->items["bucket"][0]["leaf"]["copy"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "integer:2|integer:2\ninteger:3|integer:3\ninteger:4|integer:4"
    );
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $slot = $offset;
        $leaf = "leaf";
        $this->items[$slot][$leaf] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$keyed = new KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = array("bad");
"#,
    );
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn offset_set_local_property_aliases_preserve_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id = 1;
}

class KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $items =& $this->items;
        $items[$offset]["leaf"] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class PrefixedBag implements ArrayAccess {
    public $items = array("bucket" => array());

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items["bucket"][$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items["bucket"][$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $bucket =& $this->items["bucket"];
        $bucket[$offset]["leaf"] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items["bucket"][$offset]); }
}

class BranchBag implements ArrayAccess {
    public $items = array("bucket" => array());

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items["bucket"][$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items["bucket"][$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $items =& $this->items;
        if ($offset === null) {
            $items["bucket"][]["leaf"] = $value;
            return;
        }
        $items["bucket"][$offset]["leaf"] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items["bucket"][$offset]); }
}

$box = new Box();
$alias =& $box->id;

$keyed = new KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($keyed->items["outer"]["leaf"]["copy"]), ":", $keyed->items["outer"]["leaf"]["copy"], "\n";

$prefixed = new PrefixedBag();
$prefixed["outer"] = array("copy" => &$alias);
$prefixed->items["bucket"]["outer"]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($prefixed->items["bucket"]["outer"]["leaf"]["copy"]), ":", $prefixed->items["bucket"]["outer"]["leaf"]["copy"], "\n";

$branch = new BranchBag();
$branch[] = array("copy" => &$alias);
$branch->items["bucket"][0]["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"][0]["leaf"]["copy"]), ":", $branch->items["bucket"][0]["leaf"]["copy"], "\n";

$branch["named"] = array("copy" => &$alias);
$branch->items["bucket"]["named"]["leaf"]["copy"] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"]["named"]["leaf"]["copy"]), ":", $branch->items["bucket"]["named"]["leaf"]["copy"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "integer:2|integer:2\ninteger:3|integer:3\ninteger:4|integer:4\ninteger:5|integer:5"
    );
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Box {
    public int $id = 1;
}

class KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $items =& $this->items;
        $items[$offset]["leaf"] = $value;
    }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$box = new Box();
$alias =& $box->id;
$keyed = new KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = array("bad");
"#,
    );
    assert_eq!(
        error.message,
        "Cannot assign array to reference held by property Box::$id of type int"
    );
}

#[test]
fn by_value_magic_get_false_nested_writes_are_noops() {
    let execution = run_source(
        r#"<?php
error_reporting(E_NOTICE);
function notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice", E_NOTICE);

class Box {
    public function __get($name) {
        return false;
    }
}

$box = new Box();
$box->missing["leaf"] = "keyed";
$box->missing["outer"]["leaf"] = "nested";
$box->missing[] = "append";
$box->missing["outer"][] = "nested-append";

echo property_exists($box, "missing") ? "mutated" : "no-op";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "notice:Indirect modification of overloaded property Box::$missing has no effect\n",
            "notice:Indirect modification of overloaded property Box::$missing has no effect\n",
            "notice:Indirect modification of overloaded property Box::$missing has no effect\n",
            "notice:Indirect modification of overloaded property Box::$missing has no effect\n",
            "no-op"
        )
    );
    assert_eq!(execution.exit_code, 0);

    let true_parent = runtime_error(
        r#"<?php
class Box {
    public function __get($name) {
        return true;
    }
}
$box = new Box();
$box->missing["leaf"] = "keyed";
"#,
    );
    assert_eq!(
        true_parent.message,
        "invalid array access: cannot write offset on bool"
    );

    let string_parent = runtime_error(
        r#"<?php
class Box {
    public function __get($name) {
        return "abc";
    }
}
$box = new Box();
$box->missing["outer"][] = "append";
"#,
    );
    assert_eq!(
        string_parent.message,
        "invalid array access: cannot write offset on string"
    );
}

#[test]
fn by_reference_magic_get_false_append_materializes_array() {
    let execution = run_source(
        r#"<?php
error_reporting(0);

class Box {
    public int $id = 1;
}

class StoreBox {
    public $store = false;

    public function &__get($name) {
        return $this->store;
    }
}

$box = new Box();
$alias =& $box->id;
$store = new StoreBox();
$store->missing[] = array("copy" => &$alias);
$store->store[0]["copy"] = "2";

echo gettype($box->id), ":", $box->id, "|", gettype($store->store[0]["copy"]), ":", $store->store[0]["copy"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "integer:2|integer:2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn root_false_nested_array_writes_materialize_and_keep_reference_slots() {
    let execution = run_source(
        r#"<?php
error_reporting(0);

class Box {
    public int $id = 1;
}

class Holder {
    public $items = false;
    public $appendItems = false;
}

$box = new Box();
$alias =& $box->id;

$direct = false;
$direct["leaf"] = array("copy" => &$alias);
$direct["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($direct["leaf"]["copy"]), ":", $direct["leaf"]["copy"], "\n";

$globalRoot = false;
$GLOBALS["globalRoot"]["leaf"] = array("copy" => &$alias);
$GLOBALS["globalRoot"]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($globalRoot["leaf"]["copy"]), ":", $globalRoot["leaf"]["copy"], "\n";

$holder = new Holder();
$holder->items["leaf"] = array("copy" => &$alias);
$holder->items["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["leaf"]["copy"]), ":", $holder->items["leaf"]["copy"], "\n";

$append = false;
$append["bucket"][] = array("copy" => &$alias);
$append["bucket"][0]["copy"] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($append["bucket"][0]["copy"]), ":", $append["bucket"][0]["copy"], "\n";

$holder->appendItems["bucket"][] = array("copy" => &$alias);
$holder->appendItems["bucket"][0]["copy"] = "6";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->appendItems["bucket"][0]["copy"]), ":", $holder->appendItems["bucket"][0]["copy"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "integer:2|integer:2\ninteger:3|integer:3\ninteger:4|integer:4\ninteger:5|integer:5\ninteger:6|integer:6"
    );
    assert_eq!(execution.exit_code, 0);

    let root_string = runtime_error(
        r#"<?php
$items = "abc";
$items["leaf"] = "x";
"#,
    );
    assert_eq!(
        root_string.message,
        "invalid array access: cannot write offset on string"
    );

    let property_true = runtime_error(
        r#"<?php
class Holder {
    public $items = true;
}
$holder = new Holder();
$holder->items["leaf"] = "x";
"#,
    );
    assert_eq!(
        property_true.message,
        "invalid array access: cannot write offset on bool"
    );
}

#[test]
fn root_false_reference_targets_materialize_and_keep_reference_slots() {
    let execution = run_source(
        r#"<?php
error_reporting(0);

class Box {
    public int $id = 1;
}

class Holder {
    public $items = false;
}

$box = new Box();
$alias =& $box->id;

$direct = false;
$direct["leaf"] =& $alias;
$direct["leaf"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($direct["leaf"]), ":", $direct["leaf"], "\n";

$globalRoot = false;
$GLOBALS["globalRoot"]["leaf"] =& $alias;
$GLOBALS["globalRoot"]["leaf"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($globalRoot["leaf"]), ":", $globalRoot["leaf"], "\n";

$holder = new Holder();
$holder->items["leaf"] =& $alias;
$holder->items["leaf"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["leaf"]), ":", $holder->items["leaf"], "\n";

$append = false;
$append[] =& $alias;
$append[0] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($append[0]), ":", $append[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "integer:2|integer:2\ninteger:3|integer:3\ninteger:4|integer:4\ninteger:5|integer:5"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn root_false_array_writes_emit_deprecation_diagnostics() {
    let execution = run_source(
        r#"<?php
function deprecated_handler($errno, $message, $file, $line) {
    echo "deprecated:", $message, "\n";
    return true;
}
set_error_handler("deprecated_handler", E_DEPRECATED);

class Holder {
    public $items = false;
    public $referenceItems = false;
    public $appendReferenceItems = false;
}

$direct = false;
$direct["leaf"] = "direct";

$globalRoot = false;
$GLOBALS["globalRoot"]["leaf"] = "global";

$holder = new Holder();
$holder->items["leaf"] = "object";

$append = false;
$append[] = "append";

$reference = false;
$source = "source";
$reference["leaf"] =& $source;

$holder->referenceItems["leaf"] =& $source;

$holder->appendReferenceItems[] =& $source;

echo $direct["leaf"], "|", $globalRoot["leaf"], "|", $holder->items["leaf"], "|", $append[0], "|", $reference["leaf"], "|", $holder->referenceItems["leaf"], "|", $holder->appendReferenceItems[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "deprecated:Automatic conversion of false to array is deprecated\n",
            "direct|global|object|append|source",
            "|source|source"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn arrayaccess_append_suffix_syntax_routes_to_backing_buckets() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $bag;
    public $dynamicBag;
}

$bag = new Bag();
$bag->items["outer"] = array();
$bag["outer"][]["leaf"] = "direct-root";
$holder = new Holder();
$holder->bag = $bag;
$holder->dynamicBag = $bag;
$property = "dynamicBag";
$holder->{$property}["outer"][]["leaf"] = "direct-dynamic";
$holders = array("box" => $holder);
$holders["box"]->bag["outer"][]["leaf"] = "non-direct";
$holders["box"]->{$property}["outer"][]["leaf"] = "non-direct-dynamic";
echo $bag->items["outer"][0]["leaf"],
    "|",
    $bag->items["outer"][1]["leaf"],
    "|",
    $bag->items["outer"][2]["leaf"],
    "|",
    $bag->items["outer"][3]["leaf"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct-root|direct-dynamic|non-direct|non-direct-dynamic"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn typed_property_reference_writes_use_live_class_alias_metadata() {
    let execution = run_source(
        r#"<?php
class Hook {}
class Registry { public HookLateAlias $instance; }

$hook = new Hook();
class_alias("Hook", "HookLateAlias");
$registry = new Registry();
$registry->instance = $hook;
$alias =& $registry->instance;
$alias = $hook;
echo get_class($registry->instance), "|", get_class($alias);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Hook|Hook");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Hook {}
class OtherHook {}
class Registry { public HookLateAlias $instance; }

$hook = new Hook();
$other = new OtherHook();
class_alias("Hook", "HookLateAlias");
$registry = new Registry();
$registry->instance = $hook;
$alias =& $registry->instance;
$alias = $other;
"#,
    );
    assert_eq!(error.line, 12);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "Cannot assign object to reference held by property Registry::$instance of type HookLateAlias"
    );
}

#[test]
fn unset_property_reference_detaches_alias_from_property_slot() {
    let execution = run_source(
        r#"<?php
class Box {
    public $item = "seed";
    public int $typed = 7;
}

$box = new Box();
$alias =& $box->item;
unset($box->item);
$alias = "alias";
echo isset($box->item) ? "item-set" : "item-unset";
echo "|", $alias, "\n";

$typed =& $box->typed;
unset($box->typed);
$typed = "bad";
echo isset($box->typed) ? "typed-set" : "typed-unset";
echo "|", $typed;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "item-unset|alias\ntyped-unset|bad");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn typed_properties_accept_inherited_class_name_assignments() {
    let execution = run_source(
        r#"<?php
class Hook {}
class ActionHook extends Hook {}
class FilterHook extends ActionHook {}
class OtherHook {}

class Registry {
    public Hook $instance;
    public static Hook $shared;
}

$registry = new Registry();
$registry->instance = new ActionHook();
Registry::$shared = new FilterHook();
echo get_class($registry->instance), "|", get_class(Registry::$shared);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ActionHook|FilterHook");
    assert_eq!(execution.exit_code, 0);

    let write_error = runtime_error(
        r#"<?php
class Hook {}
class OtherHook {}
class Registry { public Hook $instance; }
$registry = new Registry();
$registry->instance = new OtherHook();
"#,
    );
    assert_eq!(write_error.line, 6);
    assert_eq!(write_error.column, 1);
    assert_eq!(
        write_error.message,
        "Cannot assign object to property Registry::$instance of type Hook"
    );

    let static_write_error = runtime_error(
        r#"<?php
class Hook {}
class OtherHook {}
class Registry { public static Hook $shared; }
Registry::$shared = new OtherHook();
"#,
    );
    assert_eq!(static_write_error.line, 5);
    assert_eq!(static_write_error.column, 1);
    assert_eq!(
        static_write_error.message,
        "Cannot assign object to property Registry::$shared of type Hook"
    );
}

#[test]
fn typed_properties_accept_interface_name_assignments() {
    let execution = run_source(
        r#"<?php
interface HookContract {}
interface ChildHookContract extends HookContract {}
class ActionHook implements ChildHookContract {}
class FilterHook extends ActionHook {}
class OtherHook {}

class Registry {
    public HookContract $instance;
    public static ChildHookContract $shared;
}

$registry = new Registry();
$registry->instance = new ActionHook();
Registry::$shared = new FilterHook();
echo get_class($registry->instance), "|", get_class(Registry::$shared);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ActionHook|FilterHook");
    assert_eq!(execution.exit_code, 0);

    let write_error = runtime_error(
        r#"<?php
interface HookContract {}
class OtherHook {}
class Registry { public HookContract $instance; }
$registry = new Registry();
$registry->instance = new OtherHook();
"#,
    );
    assert_eq!(write_error.line, 6);
    assert_eq!(write_error.column, 1);
    assert_eq!(
        write_error.message,
        "Cannot assign object to property Registry::$instance of type HookContract"
    );

    let static_write_error = runtime_error(
        r#"<?php
interface HookContract {}
class OtherHook {}
class Registry { public static HookContract $shared; }
Registry::$shared = new OtherHook();
"#,
    );
    assert_eq!(static_write_error.line, 5);
    assert_eq!(static_write_error.column, 1);
    assert_eq!(
        static_write_error.message,
        "Cannot assign object to property Registry::$shared of type HookContract"
    );
}

#[test]
fn typed_properties_accept_class_alias_name_assignments() {
    let execution = run_source(
        r#"<?php
class Hook {}
class ActionHook extends Hook {}
interface HookContract {}
class ContractHook implements HookContract {}
class OtherHook {}

class_alias("Hook", "HookAlias");
class_alias("HookContract", "HookContractAlias");

class Registry {
    public HookAlias $instance;
    public static HookAlias $shared;
    public HookContractAlias $contract;
    public static HookContractAlias $staticContract;
}

$registry = new Registry();
$registry->instance = new Hook();
Registry::$shared = new ActionHook();
$registry->contract = new ContractHook();
Registry::$staticContract = new ContractHook();
echo get_class($registry->instance), "|", get_class(Registry::$shared), "|", get_class($registry->contract), "|", get_class(Registry::$staticContract);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hook|ActionHook|ContractHook|ContractHook"
    );
    assert_eq!(execution.exit_code, 0);

    let write_error = runtime_error(
        r#"<?php
class Hook {}
class OtherHook {}
class_alias("Hook", "HookAlias");
class Registry { public HookAlias $instance; }
$registry = new Registry();
$registry->instance = new OtherHook();
"#,
    );
    assert_eq!(write_error.line, 7);
    assert_eq!(write_error.column, 1);
    assert_eq!(
        write_error.message,
        "Cannot assign object to property Registry::$instance of type HookAlias"
    );

    let static_write_error = runtime_error(
        r#"<?php
interface HookContract {}
class OtherHook {}
class_alias("HookContract", "HookContractAlias");
class Registry { public static HookContractAlias $shared; }
Registry::$shared = new OtherHook();
"#,
    );
    assert_eq!(static_write_error.line, 6);
    assert_eq!(static_write_error.column, 1);
    assert_eq!(
        static_write_error.message,
        "Cannot assign object to property Registry::$shared of type HookContractAlias"
    );
}

#[test]
fn typed_properties_accept_class_aliases_registered_after_instantiation() {
    let execution = run_source(
        r#"<?php
class Hook {}
class ActionHook extends Hook {}
interface HookContract {}
class ContractHook implements HookContract {}

class Registry {
    public HookLateAlias $instance;
    public static HookLateAlias $shared;
    public HookContractLateAlias $contract;
    public static HookContractLateAlias $staticContract;
}

$hook = new Hook();
$action = new ActionHook();
$contract = new ContractHook();
class_alias("Hook", "HookLateAlias");
class_alias("HookContract", "HookContractLateAlias");

$registry = new Registry();
$registry->instance = $hook;
Registry::$shared = $action;
$registry->contract = $contract;
Registry::$staticContract = $contract;
echo get_class($registry->instance), "|", get_class(Registry::$shared), "|", get_class($registry->contract), "|", get_class(Registry::$staticContract);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hook|ActionHook|ContractHook|ContractHook"
    );
    assert_eq!(execution.exit_code, 0);

    let write_error = runtime_error(
        r#"<?php
class Hook {}
class OtherHook {}
class Registry { public HookLateAlias $instance; }
$other = new OtherHook();
class_alias("Hook", "HookLateAlias");
$registry = new Registry();
$registry->instance = $other;
"#,
    );
    assert_eq!(write_error.line, 8);
    assert_eq!(write_error.column, 1);
    assert_eq!(
        write_error.message,
        "Cannot assign object to property Registry::$instance of type HookLateAlias"
    );
}

#[test]
fn typed_properties_accept_bounded_union_and_intersection_property_types() {
    let execution = run_source(
        r#"<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract {}
class TaggedHook extends Hook implements TaggedContract {}
class OtherHook {}

class Registry {
    public HookContract|OtherHook|null $union = null;
    public static HookContract|OtherHook $staticUnion;
    public HookContract&TaggedContract $intersection;
    public static HookContract&TaggedContract $staticIntersection;
}

function yn($value) {
    return $value ? "1" : "0";
}

function type_names($type) {
    $names = array();
    foreach ($type->getTypes() as $inner) {
        $names[] = $inner->getName() . ":" . yn($inner->isBuiltin()) . ":" . yn($inner->allowsNull());
    }
    return implode(",", $names);
}

$registry = new Registry();
$registry->union = new Hook();
Registry::$staticUnion = new OtherHook();
$registry->intersection = new TaggedHook();
Registry::$staticIntersection = new TaggedHook();

$union = (new ReflectionProperty(Registry::class, "union"))->getType();
$intersection = (new ReflectionProperty(Registry::class, "intersection"))->getType();
echo get_class($registry->union), "|", get_class(Registry::$staticUnion), "|", get_class($registry->intersection), "|", get_class(Registry::$staticIntersection), "\n";
echo get_class($union), "|", yn($union instanceof ReflectionType), "|", yn($union->allowsNull()), "|", type_names($union), "\n";
echo get_class($intersection), "|", yn($intersection instanceof ReflectionType), "|", yn($intersection->allowsNull()), "|", type_names($intersection);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hook|OtherHook|TaggedHook|TaggedHook\nReflectionUnionType|1|1|HookContract:0:0,OtherHook:0:0,null:1:1\nReflectionIntersectionType|1|0|HookContract:0:0,TaggedContract:0:0"
    );
    assert_eq!(execution.exit_code, 0);

    let union_error = runtime_error(
        r#"<?php
interface HookContract {}
class OtherHook {}
class Bad {}
class Registry { public HookContract|OtherHook $union; }
$registry = new Registry();
$registry->union = new Bad();
"#,
    );
    assert_eq!(union_error.line, 7);
    assert_eq!(union_error.column, 1);
    assert_eq!(
        union_error.message,
        "Cannot assign object to property Registry::$union of type HookContract|OtherHook"
    );

    let intersection_error = runtime_error(
        r#"<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract {}
class Registry { public HookContract&TaggedContract $intersection; }
$registry = new Registry();
$registry->intersection = new Hook();
"#,
    );
    assert_eq!(intersection_error.line, 7);
    assert_eq!(intersection_error.column, 1);
    assert_eq!(
        intersection_error.message,
        "Cannot assign object to property Registry::$intersection of type HookContract&TaggedContract"
    );
}

#[test]
fn typed_property_unset_restores_uninitialized_instance_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public int $id;
    public ?string $label;
    public $legacy = "warm";

    public function __unset($property) {
        echo "magic=", $property;
    }
}

function yn($value) {
    return $value ? "1" : "0";
}

$box = new Box();
$box->id = 42;
$box->label = "plugin";
echo "before|", yn(isset($box->id)), yn(empty($box->id)), "|", count(get_object_vars($box)), "\n";
unset($box->id);
unset($box->label);
echo "after|", yn(isset($box->id)), yn(empty($box->id)), yn(isset($box->label)), yn(empty($box->label)), "|", count(get_object_vars($box)), "\n";
$box->id = 7;
$box->label = null;
echo "reassign|", $box->id, "|", ($box->label === null ? "null" : $box->label), "|", count(get_object_vars($box)), "\n";
unset($box->missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "before|10|3\nafter|0101|1\nreassign|7|null|3\nmagic=missing"
    );
    assert_eq!(execution.exit_code, 0);

    let read_error = runtime_error(
        r#"<?php
class Box { public int $id; }
$box = new Box();
$box->id = 42;
unset($box->id);
echo $box->id;
"#,
    );
    assert_eq!(read_error.line, 6);
    assert_eq!(read_error.column, 1);
    assert_eq!(
        read_error.message,
        "Typed property Box::$id must not be accessed before initialization"
    );
}

#[test]
fn get_declared_traits_reports_declared_trait_metadata() {
    let source = r#"<?php
namespace App;

class Box {}
trait Logger {}
trait Hookable {}

$declared = get_declared_traits();
print_r($declared);
echo count($declared), "\n";

$call = "get_declared_traits";
$dynamic = $call();
echo count($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => App\\Logger\n    [1] => App\\Hookable\n)\n2\n2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_declared_traits_requires_no_arguments() {
    let error = runtime_error("<?php\nvar_dump(get_declared_traits(42));\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "arity mismatch for get_declared_traits(): expected 0 argument(s), got 1"
    );
}

#[test]
fn get_called_class_requires_method_or_static_class_context() {
    let error = runtime_error("<?php\nvar_dump(get_called_class());\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported call get_called_class(): method or static class context is required"
    );

    let dynamic_error = runtime_error("<?php\n$call = \"get_called_class\";\nvar_dump($call());\n");

    assert_eq!(dynamic_error.line, 3);
    assert_eq!(dynamic_error.column, 10);
    assert_eq!(
        dynamic_error.message,
        "unsupported call get_called_class(): method or static class context is required"
    );
}

#[test]
fn called_class_context_supports_get_called_class_and_static_class() {
    let execution = run_source(
        r#"<?php
class Base {
    public function instanceName() {
        return get_called_class() . ":" . static::class;
    }

    public static function named() {
        return get_called_class() . ":" . static::class;
    }

    public static function forwardSelf() {
        return self::named();
    }
}

class Child extends Base {
    public static function forwardParent() {
        return parent::named();
    }
}

$child = new Child();
echo $child->instanceName(), "\n";
echo Base::named(), "\n";
echo Child::named(), "\n";
echo Child::forwardSelf(), "\n";
echo Child::forwardParent();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Child:Child\nBase:Base\nChild:Child\nChild:Child\nChild:Child"
    );
    assert_eq!(execution.exit_code, 0);

    let static_class_error = runtime_error("<?php\nstatic::class;\n");
    assert_eq!(
        static_class_error.message,
        "unsupported call static::class: static::class requires method or static class context"
    );
}

#[test]
fn late_static_method_calls_execute_visible_static_methods() {
    let execution = run_source(
        r#"<?php
class Base {
    protected static function hidden() {
        return "hidden:" . static::class;
    }

    public static function name() {
        return "base:" . static::class;
    }

    public static function label() {
        return static::name() . ":" . static::hidden();
    }
}

class Child extends Base {
    public static function name() {
        return "child:" . static::class;
    }

    public static function parentLabel() {
        return parent::label();
    }
}

echo Base::label(), "\n";
echo Child::label(), "\n";
echo Child::parentLabel();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:Base:hidden:Base\nchild:Child:hidden:Child\nchild:Child:hidden:Child"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn late_static_method_calls_report_current_boundaries() {
    let top_level = runtime_error("<?php\nstatic::make();\n");
    assert_eq!(
        top_level.message,
        "unsupported call static::make(): static method calls require method or static class context"
    );

    let non_static_method = runtime_error(
        r#"<?php
class Box {
    public function make() {}

    public static function call() {
        static::make();
    }
}
Box::call();
"#,
    );
    assert_eq!(
        non_static_method.message,
        "unsupported call Box::make(): non-static method dispatch through static:: is not implemented"
    );
}

#[test]
fn object_static_method_calls_execute_visible_static_methods() {
    let execution = run_source(
        r#"<?php
class Base {
    protected static function hidden() {
        return "hidden:" . static::class;
    }

    public static function name() {
        return "base:" . static::class;
    }

    public static function label() {
        return static::name() . ":" . static::hidden();
    }

    public function fromThis() {
        return $this::label();
    }
}

class Child extends Base {
    public static function name() {
        return "child:" . static::class;
    }

    public static function callOn($object) {
        return $object::label();
    }
}

class Vault {
    private static function key() {
        return "key:" . static::class;
    }

    public function reveal($other) {
        return $other::key();
    }
}

$base = new Base();
$child = new Child();
$vault = new Vault();
echo $base::label(), "\n";
echo $child::label(), "\n";
echo $child->fromThis(), "\n";
echo Child::callOn($child), "\n";
echo $vault->reveal(new Vault());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:Base:hidden:Base\nchild:Child:hidden:Child\nchild:Child:hidden:Child\nchild:Child:hidden:Child\nkey:Vault"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_static_method_receivers_execute_class_strings() {
    let execution = run_source(
        r#"<?php
class Base {
    public static function name() {
        return "base:" . static::class . ":" . get_called_class();
    }
}

class Child extends Base {
    public static function name() {
        return "child:" . static::class . ":" . get_called_class();
    }

    public static function parentName() {
        $class = Base::class;
        return $class::name();
    }
}

$base = "Base";
$child = "Child";
echo $base::name(), "\n";
echo $child::name(), "\n";
echo Child::parentName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:Base:Base\nchild:Child:Child\nbase:Base:Base"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_static_method_calls_report_current_boundaries() {
    let non_object = runtime_error(
        r#"<?php
$value = 12;
$value::make();
"#,
    );
    assert_eq!(
        non_object.message,
        "unsupported call make(): dynamic static method receiver must be object or class string, got int"
    );

    let undefined_class = runtime_error(
        r#"<?php
$class = "Missing";
$class::make();
"#,
    );
    assert_eq!(undefined_class.message, "Class \"Missing\" not found");

    let missing_method = runtime_error(
        r#"<?php
class Box {}
$class = "Box";
$class::make();
"#,
    );
    assert_eq!(
        missing_method.message,
        "Call to undefined method Box::make()"
    );

    let non_static_method = runtime_error(
        r#"<?php
class Box {
    public function make() {}
}
$box = new Box();
$box::make();
"#,
    );
    assert_eq!(
        non_static_method.message,
        "Non-static method Box::make() cannot be called statically"
    );

    let private_method = runtime_error(
        r#"<?php
class Box {
    private static function make() {}
}
$box = new Box();
$box::make();
"#,
    );
    assert_eq!(
        private_method.message,
        "unsupported call Box::make(): private method dispatch requires same-class method context"
    );
}

#[test]
fn late_static_properties_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Base {
    public static $shared = "base-default";
    public static $maybe;
    public static $count;
    protected static $secret;

    public static function seed($value) {
        static::$shared = $value;
        static::$secret = static::class . ":secret";
        echo static::$shared, ":", static::$secret, "\n";
        static::$shared .= ":x";
        echo static::$shared, "\n";
        static::$count ??= 0;
        static::$count += 2;
        echo static::$count++, ":", static::$count, "\n";
        echo isset(static::$shared) ? "shared:set\n" : "shared:unset\n";
        echo empty(static::$missing) ? "missing:empty\n" : "missing:not-empty\n";
        echo static::$missing ?? "fallback", "\n";
        static::$maybe ??= "maybe";
        echo static::$maybe, "\n";
    }
}

class Child extends Base {
    public static $shared = "child-default";
    public static $maybe;
    public static $count;

    public static function callParentSeed() {
        parent::seed("parent-child");
    }
}

Base::seed("base");
Child::seed("child");
Child::callParentSeed();
echo Base::$shared, "\n";
echo Child::$shared;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:Base:secret\nbase:x\n2:3\nshared:set\nmissing:empty\nfallback\nmaybe\nchild:Child:secret\nchild:x\n2:3\nshared:set\nmissing:empty\nfallback\nmaybe\nparent-child:Child:secret\nparent-child:x\n5:6\nshared:set\nmissing:empty\nfallback\nmaybe\nbase:x\nparent-child:x"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn late_static_properties_report_current_boundaries() {
    let top_level = runtime_error("<?php\nstatic::$value;\n");
    assert_eq!(
        top_level.message,
        "unsupported call static::$value: static property access requires method or static class context"
    );

    let unset_error = runtime_error(
        r#"<?php
class Box {
    public static $value;

    public static function clear() {
        unset(static::$value);
    }
}
Box::clear();
"#,
    );
    assert_eq!(
        unset_error.message,
        "unsupported call Box::$value: static property unset is not supported; assign null to the static property in the current subset"
    );
}

#[test]
fn late_static_class_constants_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Base {
    public const NAME = "base";
    protected const SECRET = "secret";

    public static function describe() {
        return static::NAME . ":" . static::class . ":" . static::SECRET;
    }

    public function instanceDescribe() {
        return static::NAME . ":" . static::class;
    }
}

class Child extends Base {
    public const NAME = "child";

    public static function parentDescribe() {
        return parent::describe();
    }
}

echo Base::describe(), "\n";
echo Child::describe(), "\n";
echo Child::parentDescribe(), "\n";
$child = new Child();
echo $child->instanceDescribe();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:Base:secret\nchild:Child:secret\nchild:Child:secret\nchild:Child"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn late_static_class_constants_report_current_boundaries() {
    let top_level = runtime_error("<?php\nstatic::VERSION;\n");
    assert_eq!(
        top_level.message,
        "unsupported call static::VERSION: static class constant access requires method or static class context"
    );

    let missing = runtime_error(
        r#"<?php
class Base {
    public static function missing() {
        return static::MISSING;
    }
}
class Child extends Base {}
Child::missing();
"#,
    );
    assert_eq!(missing.message, "undefined constant Child::MISSING");

    let private_visibility = runtime_error(
        r#"<?php
class Base {
    private const SECRET = "base";
    public static function reveal() {
        return static::SECRET;
    }
}
class Child extends Base {
    private const SECRET = "child";
}
Child::reveal();
"#,
    );
    assert_eq!(
        private_visibility.message,
        "unsupported call Child::SECRET: private class constant is not visible from the current class context"
    );
}

#[test]
fn get_called_class_requires_no_arguments() {
    let error = runtime_error("<?php\nvar_dump(get_called_class(42));\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "arity mismatch for get_called_class(): expected 0 argument(s), got 1"
    );
}

#[test]
fn spl_object_id_reports_current_object_handle_identity() {
    let source = r#"<?php
class Box {}
$left = new Box();
$alias = $left;
$right = new Box();
$call = "spl_object_id";

var_dump(spl_object_id($left) === spl_object_id($alias));
var_dump(spl_object_id($left) === spl_object_id($right));
var_dump($call($left) === spl_object_id($left));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "bool(true)\nbool(false)\nbool(true)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_object_id_requires_one_object_argument() {
    let arity_error = runtime_error("<?php\nvar_dump(spl_object_id());\n");

    assert_eq!(arity_error.line, 2);
    assert_eq!(arity_error.column, 1);
    assert_eq!(
        arity_error.message,
        "Too few arguments to function spl_object_id(), 0 passed in Command line code on line 2 and exactly 1 expected"
    );

    let type_error = runtime_error("<?php\nvar_dump(spl_object_id(42));\n");

    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 10);
    assert_eq!(
        type_error.message,
        "unsupported call spl_object_id(): argument must be object, got int"
    );
}

#[test]
fn spl_object_hash_reports_stable_current_object_handle_hash() {
    let source = r#"<?php
class Box {}
$left = new Box();
$alias = $left;
$right = new Box();
$call = "spl_object_hash";

var_dump(spl_object_hash($left) === spl_object_hash($alias));
var_dump(spl_object_hash($left) === spl_object_hash($right));
var_dump($call($left) === spl_object_hash($left));
var_dump(strlen(spl_object_hash($left)));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(true)\nint(32)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clone_expression_creates_new_handle_with_copied_properties() {
    let source = r#"<?php
class Box {
    public $name;
    public $child;
}

$child = new Box();
$child->name = "child";
$box = new Box();
$box->name = "original";
$box->child = $child;

$copy = clone $box;
$copy->name = "copy";
$box_child = $box->child;
$copy_child = $copy->child;

var_dump($box === $copy);
var_dump(spl_object_id($box) === spl_object_id($copy));
echo $box->name, "\n";
echo $copy->name, "\n";
var_dump($box_child === $copy_child);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\noriginal\ncopy\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clone_expression_mirrors_public_property_reference_slots() {
    let source = r#"<?php
class Box {
    public $items = [];
}

$box = new Box();
$box->items["slot"] = "original";
$slot =& $box->items["slot"];

$copy = clone $box;
$copy->items["slot"] = "copy-slot";
echo $slot, "|", $box->items["slot"], "|", $copy->items["slot"], "\n";

$slot = "alias-slot";
echo $slot, "|", $box->items["slot"], "|", $copy->items["slot"], "\n";

$copy->items = ["slot" => "detached"];
echo $slot, "|", $box->items["slot"], "|", $copy->items["slot"], "\n";

$items =& $box->items;
$copy2 = clone $box;
$copy2->items["slot"] = "whole-property";
echo $items["slot"], "|", $box->items["slot"], "|", $copy2->items["slot"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "copy-slot|copy-slot|copy-slot\n\
alias-slot|alias-slot|alias-slot\n\
alias-slot|alias-slot|detached\n\
whole-property|whole-property|whole-property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clone_expression_mirrors_context_property_reference_slots() {
    let source = r#"<?php
class Base {
    protected $shared = "base";

    public function readShared() {
        return $this->shared;
    }
}

class Box extends Base {
    private $secret = "initial";

    public function exercisePrivate() {
        $alias =& $this->secret;
        $copy = clone $this;
        $copy->secret = "copy-secret";
        echo $alias, "|", $this->secret, "|", $copy->secret, "\n";
        $alias = "alias-secret";
        echo $alias, "|", $this->secret, "|", $copy->secret, "\n";
    }

    public function exerciseProtected($other) {
        $alias =& $other->shared;
        $copy = clone $other;
        $copy->shared = "copy-shared";
        echo $alias, "|", $other->readShared(), "|", $copy->readShared(), "\n";
        $alias = "alias-shared";
        echo $alias, "|", $other->readShared(), "|", $copy->readShared();
    }
}

$box = new Box();
$box->exercisePrivate();
$box->exerciseProtected(new Box());
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "copy-secret|copy-secret|copy-secret\n\
alias-secret|alias-secret|alias-secret\n\
copy-shared|copy-shared|copy-shared\n\
alias-shared|alias-shared|alias-shared"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clone_expression_mirrors_context_property_array_offset_reference_slots() {
    let source = r#"<?php
class Base {
    protected $items = ["slot" => "base"];

    public function readItem($key) {
        return $this->items[$key];
    }
}

class Box extends Base {
    private $privateItems = ["slot" => "private"];
    private $privateAppends = [];

    public function exercisePrivate($key) {
        $alias =& $this->privateItems[$key];
        $copy = clone $this;
        $copy->privateItems[$key] = "copy-private";
        echo $alias, "|", $this->privateItems[$key], "|", $copy->privateItems[$key], "\n";
        $alias = "alias-private";
        echo $alias, "|", $this->privateItems[$key], "|", $copy->privateItems[$key], "\n";
    }

    public function exercisePrivateAppend() {
        $alias =& $this->privateAppends[];
        $copy = clone $this;
        $copy->privateAppends[0] = "copy-append";
        echo $alias, "|", $this->privateAppends[0], "|", $copy->privateAppends[0], "\n";
    }

    public function exerciseProtectedPeer($other, $key) {
        $alias =& $other->items[$key];
        $copy = clone $other;
        $copy->items[$key] = "copy-protected";
        echo $alias, "|", $other->readItem($key), "|", $copy->readItem($key), "\n";
        $alias = "alias-protected";
        echo $alias, "|", $other->readItem($key), "|", $copy->readItem($key);
    }
}

$box = new Box();
$box->exercisePrivate("slot");
$box->exercisePrivateAppend();
$box->exerciseProtectedPeer(new Box(), "slot");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "copy-private|copy-private|copy-private\n\
alias-private|alias-private|alias-private\n\
copy-append|copy-append|copy-append\n\
copy-protected|copy-protected|copy-protected\n\
alias-protected|alias-protected|alias-protected"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clone_expression_rejects_non_objects_and_dispatches_declared_clone_methods() {
    let type_error = runtime_error("<?php\n$copy = clone 42;\n");

    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "clone operand must be object in the current subset, got int"
    );

    let clone_execution = run_source(
        r#"<?php
class Box {
    public $label;

    public function __construct($label) {
        $this->label = $label;
    }

    public function __clone() {
        $this->label = $this->label . "-cloned";
    }
}
$box = new Box("seed");
$copy = clone $box;
echo $box->label, "|", $copy->label;
"#,
    )
    .unwrap();
    assert_eq!(clone_execution.stdout, "seed|seed-cloned");
    assert_eq!(clone_execution.exit_code, 0);
}

#[test]
fn spl_object_hash_requires_one_object_argument() {
    let arity_error = runtime_error("<?php\nvar_dump(spl_object_hash());\n");

    assert_eq!(arity_error.line, 2);
    assert_eq!(arity_error.column, 1);
    assert_eq!(
        arity_error.message,
        "Too few arguments to function spl_object_hash(), 0 passed in Command line code on line 2 and exactly 1 expected"
    );

    let type_error = runtime_error("<?php\nvar_dump(spl_object_hash(42));\n");

    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 10);
    assert_eq!(
        type_error.message,
        "unsupported call spl_object_hash(): argument must be object, got int"
    );
}

#[test]
fn emit_ir_rejects_get_debug_type_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Box {}\necho get_debug_type(new Box());\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_folds_scalar_is_object_and_get_debug_type_calls() {
    let ir = php_compiler::emit_ir_source(
        r#"<?php
echo is_object(null) ? "1" : "0";
echo is_object(false) ? "1" : "0";
echo is_object(7) ? "1" : "0";
echo is_object(3.5) ? "1" : "0";
echo is_object("x") ? "1" : "0";
echo "\n";
echo get_debug_type(null), "\n";
echo get_debug_type(false), "\n";
echo get_debug_type(7), "\n";
echo get_debug_type(3.5), "\n";
echo get_debug_type("x");
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"0\\00\"").count(), 5, "{ir}");
    for expected in [
        "c\"null\\00\"",
        "c\"bool\\00\"",
        "c\"int\\00\"",
        "c\"float\\00\"",
        "c\"string\\00\"",
    ] {
        assert!(ir.contains(expected), "{ir}");
    }
    assert!(!ir.contains("is_object"), "{ir}");
    assert!(!ir.contains("get_debug_type"), "{ir}");
}

#[test]
fn emit_ir_rejects_array_is_object_and_get_debug_type_until_native_array_lowering_exists() {
    for source in [
        "<?php\necho is_object([]) ? 1 : 0;\n",
        "<?php\necho get_debug_type([]);\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("array lowering rejects"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_rejects_is_object_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nclass Box {}\necho is_object(new Box());\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_class_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nclass Box {}\necho get_class(new Box());\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_routes_class_exists_through_native_metadata_abi_and_folds_other_absent_metadata_calls() {
    let ir = php_compiler::emit_ir_source(
        r#"<?php
$name = "Box";
$autoload = false;
echo class_exists("Box") ? "1" : "0";
echo class_exists($name, $autoload) ? "1" : "0";
echo interface_exists("I") ? "1" : "0";
echo trait_exists("T", true) ? "1" : "0";
echo enum_exists("E") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_value_class_metadata_exists_with_diagnostic")
            .count(),
        2,
        "{ir}"
    );
    assert!(
        ir.contains("declare i1 @phpc_native_value_class_metadata_exists_with_diagnostic"),
        "{ir}"
    );
    assert!(ir.contains("i8 0"), "{ir}");
    for name in [
        "class_exists",
        "interface_exists",
        "trait_exists",
        "enum_exists",
    ] {
        assert!(!ir.contains(name), "{ir}");
    }
}

#[test]
fn emit_ir_rejects_metadata_exists_arguments_outside_native_static_subset() {
    for source in [
        "<?php\necho class_exists(42);\n",
        "<?php\necho class_exists(\"Box\", 1);\n",
        "<?php\necho class_exists();\n",
        "<?php\necho class_exists(\"Box\", false, false);\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("function calls"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_rejects_array_metadata_exists_names_until_native_array_lowering_exists() {
    for source in [
        "<?php\necho class_exists([]);\n",
        "<?php\necho interface_exists([]);\n",
        "<?php\necho trait_exists([]);\n",
        "<?php\necho enum_exists([]);\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("array lowering rejects"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_routes_absent_native_property_and_method_exists_calls_through_metadata_abi() {
    let ir = php_compiler::emit_ir_source(
        r#"<?php
$class = "Box";
$property = "name";
$method = "open";
echo property_exists("Box", "name") ? "1" : "0";
echo property_exists($class, $property) ? "1" : "0";
echo method_exists("Box", "open") ? "1" : "0";
echo method_exists($class, $method) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(
        ir.matches("call i1 @phpc_native_value_class_metadata_exists_with_diagnostic")
            .count(),
        4,
        "{ir}"
    );
    for operation in ["i8 1", "i8 2"] {
        assert!(
            ir.contains(operation),
            "missing metadata operation {operation}\n{ir}"
        );
    }
    assert!(!ir.contains("property_exists"), "{ir}");
    assert!(!ir.contains("method_exists"), "{ir}");
}

#[test]
fn emit_ir_rejects_member_metadata_exists_arguments_outside_native_static_subset() {
    for source in [
        "<?php\necho property_exists(42, \"name\");\n",
        "<?php\necho property_exists(\"Box\", 42);\n",
        "<?php\necho property_exists(\"Box\");\n",
        "<?php\necho method_exists(42, \"open\");\n",
        "<?php\necho method_exists(\"Box\", 42);\n",
        "<?php\necho method_exists(\"Box\");\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("function calls"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_rejects_array_member_metadata_exists_targets_until_native_array_lowering_exists() {
    for source in [
        "<?php\necho property_exists([], \"name\");\n",
        "<?php\necho method_exists([], \"open\");\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("array lowering rejects"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_rejects_get_class_methods_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho get_class_methods(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_class_vars_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_class_vars(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_object_vars_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Box {}\necho get_object_vars(new Box());\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_mangled_object_vars_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Box {}\necho get_mangled_object_vars(new Box());\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_property_empty_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Box { public $name; }\n$box = new Box();\necho empty($box->name);\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("object property access"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_folds_absent_native_relationship_metadata_calls() {
    let ir = php_compiler::emit_ir_source(
        r#"<?php
$class = "Box";
$target = "Box";
$allow = true;
echo is_a("Box", "Box") ? "1" : "0";
echo is_a("Box", "Box", true) ? "1" : "0";
echo is_a($class, $target, $allow) ? "1" : "0";
echo is_subclass_of("Box", "Box") ? "1" : "0";
echo is_subclass_of($class, $target, $allow) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"0\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("is_a"), "{ir}");
    assert!(!ir.contains("is_subclass_of"), "{ir}");
}

#[test]
fn emit_ir_rejects_relationship_metadata_arguments_outside_native_static_subset() {
    for source in [
        "<?php\necho is_a(42, \"Box\");\n",
        "<?php\necho is_a(\"Box\", 42);\n",
        "<?php\necho is_a(\"Box\", \"Box\", 1);\n",
        "<?php\necho is_a(\"Box\");\n",
        "<?php\necho is_subclass_of(42, \"Box\");\n",
        "<?php\necho is_subclass_of(\"Box\", 42);\n",
        "<?php\necho is_subclass_of(\"Box\", \"Box\", 1);\n",
        "<?php\necho is_subclass_of(\"Box\");\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("function calls"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_rejects_array_relationship_metadata_targets_until_native_array_lowering_exists() {
    for source in [
        "<?php\necho is_a([], \"Box\");\n",
        "<?php\necho is_subclass_of([], \"Box\");\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("array lowering rejects"),
            "{}",
            error.message
        );
    }
}

#[test]
fn emit_ir_rejects_get_parent_class_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho get_parent_class(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_classes_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_declared_classes();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_interfaces_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho get_declared_interfaces();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_class_implements_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho class_implements(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_class_uses_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho class_uses(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_class_parents_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho class_parents(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_traits_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_declared_traits();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_called_class_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_called_class();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-metadata lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_spl_object_id_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Box {}\necho spl_object_id(new Box());\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_spl_object_hash_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Box {}\necho spl_object_hash(new Box());\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object-metadata lowering rejects")
            || error.message.contains("function-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_clone_expression_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho clone $object;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("clone lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_parent_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nparent::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_self_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nself::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_named_static_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nBox::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_late_static_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nstatic::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_static_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\n$box::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_static_properties_until_native_object_lowering_exists() {
    let read_error = php_compiler::emit_ir_source("<?php\n$box::$value;\n").unwrap_err();
    assert_eq!(read_error.phase, Phase::Codegen);
    assert_eq!(read_error.message, LLVM_STATIC_MEMBER_REJECTION);

    let write_error = php_compiler::emit_ir_source("<?php\n$box::$value = 1;\n").unwrap_err();
    assert_eq!(write_error.phase, Phase::Codegen);
    assert_eq!(
        write_error.message,
        LLVM_NATIVE_ARRAY_NON_LOCAL_ASSIGNMENT_REJECTION
    );
}

#[test]
fn class_name_constants_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Box {}
class Root {}
class Base extends Root {
    public function baseNames() {
        return self::class . ":" . parent::class;
    }
}
class Child extends Base {
    public function childNames() {
        return self::class . ":" . parent::class;
    }

    public function inheritedNames() {
        return $this->baseNames();
    }
}

echo Box::class, "\n";
echo Box::CLASS, "\n";
echo Missing::class, "\n";
$child = new Child();
echo $child->childNames(), "\n";
echo $child->inheritedNames();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Box\nBox\nMissing\nChild:Base\nBase:Root");

    let self_error = runtime_error("<?php\nself::class;\n");
    assert_eq!(self_error.line, 2);
    assert_eq!(self_error.column, 5);
    assert_eq!(
        self_error.message,
        "unsupported call self::class: self::class requires instance method context"
    );

    let parent_error = runtime_error("<?php\nparent::class;\n");
    assert_eq!(parent_error.line, 2);
    assert_eq!(parent_error.column, 7);
    assert_eq!(
        parent_error.message,
        "unsupported call parent::class: parent::class requires instance method context"
    );

    let parent_error = runtime_error(
        r#"<?php
class Root {
    public function name() {
        return parent::class;
    }
}
$root = new Root();
echo $root->name();
"#,
    );
    assert_eq!(
        parent_error.message,
        "unsupported call parent::class: parent::class requires a parent class"
    );
}

#[test]
fn magic_class_names_in_new_expressions_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Base {
    public static function makeSelf() {
        return new self();
    }

    public static function makeStatic() {
        return new static();
    }
}

class Child extends Base {
    public function makeParent() {
        return new parent;
    }
}

echo get_class(Base::makeSelf()), "\n";
echo get_class(Child::makeSelf()), "\n";
echo get_class(Child::makeStatic()), "\n";
$child = new Child();
echo get_class($child->makeParent());
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Base\nBase\nChild\nBase");

    let self_error = runtime_error("<?php\nnew self();\n");
    assert_eq!(self_error.line, 2);
    assert_eq!(self_error.column, 1);
    assert_eq!(
        self_error.message,
        "unsupported object instantiation for self: self requires active class context"
    );

    let parent_error = runtime_error(
        r#"<?php
class Root {
    public function make() {
        return new parent();
    }
}
$root = new Root();
$root->make();
"#,
    );
    assert_eq!(
        parent_error.message,
        "unsupported object instantiation for parent: parent requires a parent class"
    );
}

#[test]
fn emit_ir_rejects_class_name_constants_until_native_object_lowering_exists() {
    for source in [
        "<?php\necho Box::class;\n",
        "<?php\nself::class;\n",
        "<?php\nparent::class;\n",
        "<?php\nstatic::class;\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CLASS_NAME_CONSTANT_REJECTION);
    }
}

#[test]
fn class_constants_execute_current_subset() {
    let execution = run_source(
        r#"<?php
const GLOBAL_SUFFIX = "global";
class Box {}
class Root {
    public const ROOT = "root";
    protected const PROTECTED_NAME = "protected";
    private const SECRET = "secret";
    public const LABEL = Box::class;
    public const SUM = 7 + 5;
    public const FROM_GLOBAL = GLOBAL_SUFFIX;

    public function rootNames() {
        return self::ROOT . ":" . self::SECRET;
    }
}
class Base extends Root {
    public const BASE = "base";

    public function baseNames() {
        return self::BASE . ":" . parent::ROOT . ":" . parent::PROTECTED_NAME;
    }
}
class Child extends Base {
    public function childNames() {
        return self::BASE . ":" . parent::BASE . ":" . Root::ROOT . ":" . Root::LABEL . ":" . Root::SUM . ":" . Root::FROM_GLOBAL;
    }
}

$child = new Child();
echo Root::ROOT, "\n";
echo $child->rootNames(), "\n";
echo $child->baseNames(), "\n";
echo $child->childNames();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "root\nroot:secret\nbase:root:protected\nbase:base:root:Box:12:global"
    );

    let self_error = runtime_error("<?php\nself::VERSION;\n");
    assert_eq!(
        self_error.message,
        "unsupported call self::VERSION: self class constant access requires instance method context"
    );

    let parent_error = runtime_error("<?php\nparent::VERSION;\n");
    assert_eq!(
        parent_error.message,
        "unsupported call parent::VERSION: parent class constant access requires instance method context"
    );

    let missing_parent_error = runtime_error(
        r#"<?php
class Root {
    public function name() {
        return parent::VERSION;
    }
}
$root = new Root();
echo $root->name();
"#,
    );
    assert_eq!(
        missing_parent_error.message,
        "unsupported call parent::VERSION: parent class constant access requires a parent class"
    );

    let visibility_error = runtime_error(
        r#"<?php
class Root {
    protected const NAME = "root";
}
echo Root::NAME;
"#,
    );
    assert_eq!(
        visibility_error.message,
        "unsupported call Root::NAME: protected class constant is not visible from the current class context"
    );

    let private_error = runtime_error(
        r#"<?php
class Root {
    private const NAME = "root";
}
echo Root::NAME;
"#,
    );
    assert_eq!(
        private_error.message,
        "unsupported call Root::NAME: private class constant is not visible from the current class context"
    );

    let undefined_class = runtime_error("<?php\necho Missing::VALUE;\n");
    assert_eq!(undefined_class.message, "Class \"Missing\" not found");

    let undefined_constant = runtime_error(
        r#"<?php
class Root {}
echo Root::MISSING;
"#,
    );
    assert_eq!(
        undefined_constant.message,
        "undefined constant Root::MISSING"
    );
}

#[test]
fn reflection_class_reports_bounded_constant_metadata() {
    let execution = run_source(
        r#"<?php
interface Contract {
    const FLAG = "iface";
}

class Base {
    public const BASE = "base";
    protected const PROTECTED_NAME = "protected";
    private const SECRET = "base-secret";
}

class Plugin extends Base implements Contract {
    public const NAME = "plugin";
    public const COMBO = "combo";
    private const SECRET = "child-secret";
}

function yn($value) {
    return $value ? "1" : "0";
}

$class = new ReflectionClass(Plugin::class);
echo "has|", yn($class->hasConstant("NAME")), yn($class->hasConstant("MISSING")), "\n";
echo "values|", $class->getConstant("COMBO"), "|", $class->getConstant("PROTECTED_NAME"), "|", $class->getConstant("SECRET"), "|", $class->getConstant("FLAG"), "\n";
foreach ($class->getConstants() as $name => $value) {
    echo "all|", $name, "=", $value, "\n";
}
foreach ($class->getConstants(1) as $name => $value) {
    echo "public|", $name, "=", $value, "\n";
}
foreach ($class->getConstants(4) as $name => $value) {
    echo "private|", $name, "=", $value, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "has|10\nvalues|combo|protected|child-secret|iface\nall|NAME=plugin\nall|COMBO=combo\nall|SECRET=child-secret\nall|BASE=base\nall|PROTECTED_NAME=protected\nall|FLAG=iface\npublic|NAME=plugin\npublic|COMBO=combo\npublic|BASE=base\npublic|FLAG=iface\nprivate|SECRET=child-secret\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_constant_lookup_accepts_scalar_names() {
    let execution = run_source(
        r#"<?php
class Packet {
    public const VALUE = 1;
}

function yn($value) {
    return $value ? "1" : "0";
}

$class = new ReflectionClass(Packet::class);
echo "has|", yn($class->hasConstant(1)), yn($class->hasConstant(1.5)), yn($class->hasConstant(true)), "\n";
var_dump($class->getConstant(1));
"#,
    )
    .unwrap();

    assert!(execution.stdout.starts_with("has|000\n"));
    assert!(execution.stdout.contains(
        "Deprecated: ReflectionClass::getConstant() for a non-existent constant is deprecated"
    ));
    assert!(execution.stdout.ends_with("bool(false)\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_bounded_shape_relationship_metadata() {
    let execution = run_source(
        r#"<?php
namespace App\Meta;

abstract class Base {}
final class Plugin extends Base implements \IteratorAggregate {
    public function getIterator() {
        return array();
    }
}
class Plain {}

function yn($value) {
    return $value ? "1" : "0";
}

$plugin = new \ReflectionClass(Plugin::class);
$base = new \ReflectionClass(Base::class);
$plain = new \ReflectionClass(Plain::class);
$core = new \ReflectionClass("stdClass");

echo "names|", yn($plugin->inNamespace()), "|", $plugin->getNamespaceName(), "|", $plugin->getShortName(), "\n";
echo "mods|", $base->getModifiers(), "|", yn($base->isAbstract()), yn($base->isFinal()), "|", $plugin->getModifiers(), "|", yn($plugin->isAbstract()), yn($plugin->isFinal()), "\n";
echo "origin|", yn($core->isInternal()), yn($core->isUserDefined()), "|", yn($plain->isInternal()), yn($plain->isUserDefined()), "\n";
echo "subclass|", yn($plugin->isSubclassOf($base)), yn($plugin->isSubclassOf(Base::class)), yn($base->isSubclassOf($plugin)), "\n";
echo "instance|", yn($base->isInstance(new Plugin())), yn($plugin->isInstance(new Plain())), yn($plain->isInstance(new Plugin())), "\n";
echo "iterable|", yn($plugin->isIterable()), yn($plugin->isIterateable()), yn($plain->isIterable());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "names|1|App\\Meta|Plugin\nmods|64|10|32|01\norigin|10|01\nsubclass|110\ninstance|100\niterable|110"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_class_reports_static_and_default_property_metadata() {
    let execution = run_source(
        r#"<?php
class Base {
    private $hidden = "secret";
    private static $secret = "hidden";
    public static $shared = "base";
    protected static $cache = "warm";
    protected $base = 2;
    public $nullSlot;
}

class Plugin extends Base {
    public static $active = true;
    public static $shared = "plugin";
    private $own = 1;
    public $name = "hook";
    protected $items = array("a" => 1);
}

class A {
    public static $x = "default";
    public $y = "iy";
}

function label($value) {
    if (is_array($value)) {
        return "array:" . count($value);
    }
    if (is_bool($value)) {
        return "bool:" . ($value ? "1" : "0");
    }
    if ($value === null) {
        return "null";
    }
    return (string) $value;
}

$class = new ReflectionClass(Plugin::class);
foreach ($class->getDefaultProperties() as $name => $value) {
    echo "default|", $name, "|", label($value), "\n";
}
foreach ($class->getStaticProperties() as $name => $value) {
    echo "static|", $name, "|", label($value), "\n";
}
echo "get|", label($class->getStaticPropertyValue("cache")), "|", label($class->getStaticPropertyValue("shared")), "|", label($class->getStaticPropertyValue("secret", "fallback")), "\n";
$class->setStaticPropertyValue("cache", "hot");
echo "set|", label($class->getStaticPropertyValue("cache")), "|", label(Base::$shared), "|", label(Plugin::$shared), "\n";
A::$x = "changed";
$aClass = new ReflectionClass(A::class);
echo "mutation|", label($aClass->getDefaultProperties()["x"]), "|", label($aClass->getStaticProperties()["x"]);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "default|active|bool:1\ndefault|shared|plugin\ndefault|cache|warm\ndefault|own|1\ndefault|name|hook\ndefault|items|array:1\ndefault|base|2\ndefault|nullSlot|null\nstatic|active|bool:1\nstatic|shared|plugin\nstatic|cache|warm\nget|warm|plugin|fallback\nset|hot|base|plugin\nmutation|default|changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_class_constants_until_native_object_lowering_exists() {
    for source in [
        "<?php\necho Box::VERSION;\n",
        "<?php\nself::VERSION;\n",
        "<?php\nparent::VERSION;\n",
        "<?php\nstatic::VERSION;\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_STATIC_MEMBER_REJECTION);
    }
}

#[test]
fn static_properties_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Counter {
    public static $count;
}
class Base {
    public static $shared;
    protected static $secret;

    public function readBase() {
        return self::$shared . ":" . self::$secret;
    }
}
class Child extends Base {
    public static $own;

    public function writeBoth() {
        parent::$shared = "base";
        parent::$secret = "protected";
        self::$own = "child";
        return parent::$shared . ":" . self::$own;
    }
}

Counter::$count = 1;
echo Counter::$count, "\n";
Counter::$count = Counter::$count + 4;
echo Counter::$count, "\n";
$child = new Child();
echo $child->writeBoth(), "\n";
echo $child->readBase(), "\n";
echo Base::$shared;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\n5\nbase:child\nbase:protected\nbase");

    let self_error = runtime_error("<?php\nself::$value;\n");
    assert_eq!(
        self_error.message,
        "unsupported call self::$value: self static property access requires instance method context"
    );

    let parent_error = runtime_error("<?php\nparent::$value;\n");
    assert_eq!(
        parent_error.message,
        "unsupported call parent::$value: parent static property access requires instance method context"
    );

    let missing_parent_error = runtime_error(
        r#"<?php
class Root {
    public function read() {
        return parent::$value;
    }
}
$root = new Root();
echo $root->read();
"#,
    );
    assert_eq!(
        missing_parent_error.message,
        "unsupported call parent::$value: parent static property access requires a parent class"
    );

    let visibility_error = runtime_error(
        r#"<?php
class Root {
    protected static $name;
}
echo Root::$name;
"#,
    );
    assert_eq!(
        visibility_error.message,
        "unsupported call Root::$name: protected static property is not visible from the current class context"
    );

    let private_error = runtime_error(
        r#"<?php
class Root {
    private static $name;
}
echo Root::$name;
"#,
    );
    assert_eq!(
        private_error.message,
        "unsupported call Root::$name: private static property is not visible from the current class context"
    );

    let undefined_class = runtime_error("<?php\necho Missing::$value;\n");
    assert_eq!(undefined_class.message, "Class \"Missing\" not found");

    let undefined_property = runtime_error(
        r#"<?php
class Root {}
echo Root::$missing;
"#,
    );
    assert_eq!(
        undefined_property.message,
        "undefined property Root::$missing"
    );
}

#[test]
fn object_static_properties_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Mailer {
    public static $validator;
    protected static $secret;

    public function seedSecret($value) {
        $this::$secret = $value;
    }

    public static function readSecret() {
        return static::$secret;
    }
}

class ChildMailer extends Mailer {}

$mailer = new Mailer();
$mailer::$validator = "object";
echo Mailer::$validator, "\n";

$class = "ChildMailer";
$class::$validator = "class-string";
echo ChildMailer::$validator, "\n";

$child = new ChildMailer();
$child->seedSecret("hidden");
echo ChildMailer::readSecret(), "\n";

function install_validator($phpmailer) {
    $phpmailer::$validator = static function ($email) {
        return true;
    };
}

echo "closure-assignment-parsed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "object\nclass-string\nhidden\nclosure-assignment-parsed"
    );
    assert_eq!(execution.exit_code, 0);

    let non_object = runtime_error(
        r#"<?php
$value = 12;
$value::$name;
"#,
    );
    assert_eq!(
        non_object.message,
        "unsupported call ::$name: dynamic static property receiver must be object or class string, got int"
    );

    let private_error = runtime_error(
        r#"<?php
class Mailer {
    private static $validator;
}
$mailer = new Mailer();
echo $mailer::$validator;
"#,
    );
    assert_eq!(
        private_error.message,
        "unsupported call Mailer::$validator: private static property is not visible from the current class context"
    );
}

#[test]
fn static_properties_support_current_default_value_subset() {
    let execution = run_source(
        r#"<?php
class Defaults {
    public static $name = "Ada";
    public static $count = 2 + 3;
    protected static $secret = "ok";

    public static function read() {
        return self::$name . ":" . self::$count . ":" . self::$secret;
    }
}

echo Defaults::$name, "\n";
echo Defaults::$count, "\n";
Defaults::$count += 4;
echo Defaults::read();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada\n5\nAda:9:ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn instance_properties_support_current_default_value_subset() {
    let execution = run_source(
        r#"<?php
class Defaults {
    public $count = 2 + 3;
    public $name = "Ada";
    public $flag = true;
    public $nothing = null;
    protected $secret = "ok";
    private $token = "sealed";

    public function __construct() {
        echo $this->count, ":", $this->secret, ":", $this->token, "\n";
    }

    public function read() {
        return $this->count . ":" . $this->name . ":" . $this->flag . ":" . ($this->nothing === null);
    }
}

$first = new Defaults();
$second = new Defaults();
$first->count = 9;
echo $first->read(), "\n";
echo $second->read();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5:ok:sealed\n5:ok:sealed\n9:Ada:1:1\n5:Ada:1:1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_instance_property_defaults_initialize_visible_slots() {
    let execution = run_source(
        r#"<?php
class Base {
    public $name = "base";
    protected $shared = "shared";
    private $token = "base-token";

    public function baseRead() {
        return $this->name . ":" . $this->shared . ":" . $this->token;
    }
}

class Child extends Base {
    public $name = "child";
    private $token = "child-token";

    public function childRead() {
        return $this->name . ":" . $this->shared . ":" . $this->token;
    }
}

$child = new Child();
echo $child->baseRead(), "\n";
echo $child->childRead();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "child:shared:base-token\nchild:shared:child-token"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn property_introspection_reports_current_instance_defaults() {
    let execution = run_source(
        r#"<?php
class Defaults {
    public $count = 2;
    public static $shared = "S";
    protected $secret = "ok";
    private $token = "sealed";
}

$object = new Defaults();
print_r(get_class_vars("Defaults"));
print_r(get_object_vars($object));
$mangled = get_mangled_object_vars($object);
$keys = array_keys($mangled);
echo count($mangled), "|", $mangled[$keys[0]], "|", $mangled[$keys[1]], "|", $mangled[$keys[2]];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array\n(\n    [count] => 2\n    [shared] => S\n)\nArray\n(\n    [count] => 2\n)\n3|2|ok|sealed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_properties_support_current_mutation_subset() {
    let execution = run_source(
        r#"<?php
class Counter {
    public static $count;
    public static $label;
    public static $maybe;
}
class Base {
    protected static $shared;
}
class Child extends Base {
    public static $own;

    public function run() {
        parent::$shared = 5;
        parent::$shared += 2;
        echo parent::$shared++, "\n";
        echo parent::$shared, "\n";
        echo ++parent::$shared, "\n";
        self::$own ??= "child";
        self::$own ??= "again";
        return parent::$shared . ":" . self::$own;
    }
}
class LoopCounter {
    public static $i;
}

Counter::$count = 1;
$updated = (Counter::$count += 4);
echo $updated, "\n";
echo Counter::$count++, "\n";
echo ++Counter::$count, "\n";
Counter::$label = "a";
Counter::$label .= "b";
echo Counter::$label, "\n";
$first = (Counter::$maybe ??= "first");
$second = (Counter::$maybe ??= "second");
echo $first, "\n";
echo $second, "\n";
$child = new Child();
echo $child->run(), "\n";
for (LoopCounter::$i = 0; LoopCounter::$i < 2; LoopCounter::$i++) {
    echo "loop", LoopCounter::$i, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5\n5\n7\nab\nfirst\nfirst\n7\n8\n9\n9:child\nloop0\nloop1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_properties_support_current_isset_empty_and_null_coalescing_subset() {
    let execution = run_source(
        r#"<?php
class Flags {
    public static $nullable;
    public static $flag;
    public static $zero;
    public static $blank;
    public static $text;
}
class Base {
    protected static $secret;
    public static $shared;
}
class Child extends Base {
    public function run() {
        parent::$secret = "hidden";
        self::$shared = "shared";
        echo isset(parent::$secret) ? "secret:set\n" : "secret:unset\n";
        echo empty(parent::$secret) ? "secret:empty\n" : "secret:not-empty\n";
        echo parent::$secret ?? "secret-fallback", "\n";
        echo self::$shared ?? "shared-fallback", "\n";
    }
}

Flags::$flag = false;
Flags::$zero = 0;
Flags::$blank = "";
Flags::$text = "ok";
echo isset(Flags::$nullable) ? "nullable:set\n" : "nullable:unset\n";
echo isset(Flags::$flag) ? "flag:set\n" : "flag:unset\n";
echo isset(Flags::$zero) ? "zero:set\n" : "zero:unset\n";
echo isset(Flags::$blank) ? "blank:set\n" : "blank:unset\n";
echo isset(Flags::$text) ? "text:set\n" : "text:unset\n";
echo empty(Flags::$nullable) ? "nullable:empty\n" : "nullable:not-empty\n";
echo empty(Flags::$flag) ? "flag:empty\n" : "flag:not-empty\n";
echo empty(Flags::$zero) ? "zero:empty\n" : "zero:not-empty\n";
echo empty(Flags::$blank) ? "blank:empty\n" : "blank:not-empty\n";
echo empty(Flags::$text) ? "text:empty\n" : "text:not-empty\n";
echo Flags::$nullable ?? "fallback", "\n";
echo Flags::$text ?? "fallback", "\n";
echo isset(Flags::$missing) ? "missing:set\n" : "missing:unset\n";
echo empty(Flags::$missing) ? "missing:empty\n" : "missing:not-empty\n";
echo Flags::$missing ?? "missing-fallback", "\n";
$child = new Child();
$child->run();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "nullable:unset\nflag:set\nzero:set\nblank:set\ntext:set\nnullable:empty\nflag:empty\nzero:empty\nblank:empty\ntext:not-empty\nfallback\nok\nmissing:unset\nmissing:empty\nmissing-fallback\nsecret:set\nsecret:not-empty\nhidden\nshared\n"
    );

    let visibility_error = runtime_error(
        r#"<?php
class Root {
    protected static $name;
}
isset(Root::$name);
"#,
    );
    assert_eq!(
        visibility_error.message,
        "unsupported call Root::$name: protected static property is not visible from the current class context"
    );
}

#[test]
fn static_property_unset_reports_current_php_forbidden_boundary() {
    let public_error = runtime_error(
        r#"<?php
class Root {
    public static $name;
}
unset(Root::$name);
"#,
    );
    assert_eq!(
        public_error.message,
        "unsupported call Root::$name: static property unset is not supported; assign null to the static property in the current subset"
    );

    let missing_property_error = runtime_error(
        r#"<?php
class Root {}
unset(Root::$missing);
"#,
    );
    assert_eq!(
        missing_property_error.message,
        "unsupported call Root::$missing: static property unset is not supported; assign null to the static property in the current subset"
    );

    let self_error = runtime_error(
        r#"<?php
class Root {
    public static $name;

    public function clear() {
        unset(self::$name);
    }
}
$root = new Root();
$root->clear();
"#,
    );
    assert_eq!(
        self_error.message,
        "unsupported call Root::$name: static property unset is not supported; assign null to the static property in the current subset"
    );

    let parent_error = runtime_error(
        r#"<?php
class Base {
    protected static $secret;
}
class Child extends Base {
    public function clear() {
        unset(parent::$secret);
    }
}
$child = new Child();
$child->clear();
"#,
    );
    assert_eq!(
        parent_error.message,
        "unsupported call Base::$secret: static property unset is not supported; assign null to the static property in the current subset"
    );

    let undefined_class = runtime_error("<?php\nunset(Missing::$value);\n");
    assert_eq!(undefined_class.message, "Class \"Missing\" not found");
}

#[test]
fn emit_ir_rejects_static_properties_until_native_object_lowering_exists() {
    for source in [
        "<?php\necho Box::$cache;\n",
        "<?php\nself::$cache;\n",
        "<?php\nparent::$cache;\n",
        "<?php\nstatic::$cache;\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_STATIC_MEMBER_REJECTION);
    }
}

#[test]
fn undefined_class_instantiation_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
$box = new Missing();
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "Class \"Missing\" not found");
}

#[test]
fn public_constructors_execute_with_this_binding() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
    public $count;

    public function __construct($name = "Ada") {
        $this->name = $name;
        $this->count = 1;
    }

    public function label() {
        return $this->name . ":" . $this->count;
    }
}

$box = new Box("Grace");
echo $box->label(), "\n";
$default = new Box();
echo $default->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace:1\nAda:1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn constructor_reference_parameters_share_caller_cell_during_execution() {
    let execution = run_source(
        r#"<?php
function observe() {
    global $value;
    echo "seen=", $value, "|";
}

class Box {
    public function __construct(&$param) {
        $param = 2;
        observe();
        $param = 3;
    }
}

$value = 1;
$box = new Box($value);
echo "final=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seen=2|final=3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_constructor_reference_parameter_detaches_local_name() {
    let execution = run_source(
        r#"<?php
class Box {
    public function __construct(&$param) {
        unset($param);
        $param = 9;
        echo "local=", $param, "|";
    }
}

$value = 1;
$box = new Box($value);
echo "caller=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "local=9|caller=1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_public_constructors_execute_with_child_this_binding() {
    let execution = run_source(
        r#"<?php
class Base {
    public $id;

    public function __construct($id = 7) {
        $this->id = $id;
    }

    public function label() {
        return "base:" . $this->id;
    }
}

class Child extends Base {
    public $name;

    public function rename($name) {
        $this->name = $name;
    }
}

$child = new Child(11);
$child->rename("Ada");
echo $child->label(), "\n";
echo $child->id, "|", $child->name, "\n";
$default = new Child();
echo $default->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "base:11\n11|Ada\nbase:7");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn public_destructors_run_at_shutdown_for_allocated_objects() {
    let execution = run_source(
        r#"<?php
class Base {
    public $name;

    public function __construct($name) {
        $this->name = $name;
        echo "construct:", $this->name, "\n";
    }

    public function __destruct() {
        echo "destruct:", $this->name;
        if ($this->name !== "first") {
            echo "\n";
        }
    }
}

class Child extends Base {}

$first = new Child("first");
$second = new Child("second");
$copy = clone $first;
$copy->name = "copy";
echo "body\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "construct:first\nconstruct:second\nbody\ndestruct:copy\ndestruct:second\ndestruct:first"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn destructor_global_hash_table_retention_loop_finishes() {
    let execution = run_source(
        r#"<?php
define('OBJECT_COUNT', 512);

$containers = array();

class ObjectOne {
    protected $guid = 0;

    public function __construct() {
        global $containers;
        $this->guid = 1;
        $containers[spl_object_hash($this)] = $this;
    }

    public function __destruct() {
        global $containers;
        $containers[spl_object_hash($this)] = NULL;
    }
}

for ($i = 0; $i < OBJECT_COUNT; ++$i) {
    new ObjectOne();
}

echo count($containers), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "512\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_unset_finalizes_unreachable_destructor_objects_immediately() {
    let execution = run_source(
        r#"<?php
class Watch {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function __destruct() {
        echo "|destruct:", $this->name;
    }
}

$single = new Watch("single");
echo "body";
unset($single);
echo "|after-single";

$kept = new Watch("kept");
$alias = $kept;
unset($kept);
echo "|after-kept";
unset($alias);
echo "|after-alias";

$left = new Watch("many");
$right = $left;
unset($left, $right);
echo "|after-many";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "body|destruct:single|after-single|after-kept|destruct:kept|after-alias|destruct:many|after-many"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_unset_destructor_output_stays_inside_active_output_buffer() {
    let execution = run_source(
        r#"<?php
class BufferedWatch {
    public function __destruct() {
        echo "|destruct";
    }
}

ob_start();
$watch = new BufferedWatch();
echo "body";
unset($watch);
echo "|after";
$captured = ob_get_clean();
echo "captured=[" . $captured . "]";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "captured=[body|destruct|after]");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_unset_keeps_caller_frame_object_roots_alive() {
    let execution = run_source(
        r#"<?php
class FrameWatch {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function __destruct() {
        echo "|destruct:", $this->name;
    }
}

function clear_parameter($param) {
    echo "body";
    unset($param);
    echo "|after-callee";
}

function clear_caller() {
    $root = new FrameWatch("caller");
    clear_parameter($root);
    echo "|after-caller";
    unset($root);
    echo "|after-caller-unset";
}

clear_caller();
echo "|end";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "body|after-callee|after-caller|destruct:caller|after-caller-unset|end"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_unset_finalizes_function_local_last_root_immediately() {
    let execution = run_source(
        r#"<?php
class LocalWatch {
    public function __destruct() {
        echo "|destruct";
    }
}

function clear_local() {
    $root = new LocalWatch();
    echo "body";
    unset($root);
    echo "|after-local";
}

clear_local();
echo "|end";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "body|destruct|after-local|end");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_unset_defers_objects_reachable_from_tracked_runtime_roots() {
    let execution = run_source(
        r#"<?php
class StaticRoot {
    public static $slot;
}

class RuntimeRootWatch {
    public function __destruct() {
        echo "|destruct";
    }
}

function shutdown_marker($value) {
    echo "|shutdown";
}

$root = new RuntimeRootWatch();
StaticRoot::$slot = $root;
register_shutdown_function("shutdown_marker", $root);
echo "body";
unset($root);
echo "|after-unset";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "body|after-unset|shutdown|destruct");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn destructor_declarations_validate_public_non_static_parameterless_shape() {
    let private_destructor = runtime_error(
        r#"<?php
class PrivateDestructor {
    private function __destruct() {}
}
echo "unreachable";
"#,
    );
    assert_eq!(private_destructor.line, 3);
    assert_eq!(private_destructor.column, 13);
    assert_eq!(
        private_destructor.message,
        "unsupported class inheritance for PrivateDestructor: destructor PrivateDestructor::__destruct() must be public in the current subset"
    );

    let protected_destructor = runtime_error(
        r#"<?php
class ProtectedDestructor {
    protected function __destruct() {}
}
"#,
    );
    assert_eq!(protected_destructor.line, 3);
    assert_eq!(protected_destructor.column, 15);
    assert_eq!(
        protected_destructor.message,
        "unsupported class inheritance for ProtectedDestructor: destructor ProtectedDestructor::__destruct() must be public in the current subset"
    );

    let static_destructor = runtime_error(
        r#"<?php
class StaticDestructor {
    public static function __destruct() {}
}
"#,
    );
    assert_eq!(static_destructor.line, 3);
    assert_eq!(static_destructor.column, 19);
    assert_eq!(
        static_destructor.message,
        "unsupported class inheritance for StaticDestructor: destructor StaticDestructor::__destruct() must be non-static in the current subset"
    );

    let parameter_destructor = runtime_error(
        r#"<?php
class ParameterDestructor {
    public function __destruct($value = "default") {}
}
"#,
    );
    assert_eq!(parameter_destructor.line, 3);
    assert_eq!(parameter_destructor.column, 12);
    assert_eq!(
        parameter_destructor.message,
        "unsupported class inheritance for ParameterDestructor: destructor ParameterDestructor::__destruct() cannot declare parameters in the current subset"
    );
}

#[test]
fn parent_method_calls_execute_with_current_this_binding() {
    let execution = run_source(
        r#"<?php
class Base {
    public $id;

    public function __construct($id = 7) {
        $this->id = $id;
    }

    public function label() {
        return "base:" . $this->id;
    }

    protected function bumpBase($amount) {
        $this->id = $this->id + $amount;
    }
}

class Child extends Base {
    public $name;

    public function __construct($id, $name) {
        parent::__construct($id);
        $this->name = $name;
    }

    public function label() {
        return parent::label() . ":" . $this->name;
    }

    public function bump($amount) {
        parent::bumpBase($amount);
    }
}

$child = new Child(4, "Ada");
echo $child->label(), "\n";
$child->bump(5);
echo $child->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "base:4:Ada\nbase:9:Ada");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parent_method_calls_report_current_unsupported_boundaries() {
    let top_level = runtime_error(
        r#"<?php
parent::make();
"#,
    );

    assert_eq!(top_level.line, 2);
    assert_eq!(top_level.column, 7);
    assert_eq!(
        top_level.message,
        "unsupported call parent::make(): parent method calls require instance method context"
    );

    let no_parent = runtime_error(
        r#"<?php
class Solo {
    public function call() {
        parent::make();
    }
}
$solo = new Solo();
$solo->call();
"#,
    );

    assert_eq!(no_parent.line, 4);
    assert_eq!(no_parent.column, 15);
    assert_eq!(
        no_parent.message,
        "unsupported call parent::make(): parent method calls require a parent class"
    );

    let private_parent_method = runtime_error(
        r#"<?php
class Base {
    private function hide() {}
}
class Child extends Base {
    public function call() {
        parent::hide();
    }
}
$child = new Child();
$child->call();
"#,
    );

    assert_eq!(private_parent_method.line, 7);
    assert_eq!(private_parent_method.column, 15);
    assert_eq!(
        private_parent_method.message,
        "unsupported call Base::hide(): private method dispatch requires same-class method context"
    );

    let non_static_parent_method_without_this = runtime_error(
        r#"<?php
class Base {
    public function make() {}
}
class Child extends Base {
    public static function call() {
        parent::make();
    }
}
Child::call();
"#,
    );

    assert_eq!(non_static_parent_method_without_this.line, 7);
    assert_eq!(non_static_parent_method_without_this.column, 15);
    assert_eq!(
        non_static_parent_method_without_this.message,
        "unsupported call Base::make(): non-static method dispatch through parent:: requires current $this object context"
    );
}

#[test]
fn self_method_calls_execute_with_current_this_binding() {
    let execution = run_source(
        r#"<?php
class Base {
    public $id;

    public function baseLabel() {
        return "base:" . $this->id;
    }

    protected function bumpBase($amount) {
        $this->id = $this->id + $amount;
    }
}

class Child extends Base {
    public function __construct($id) {
        $this->id = $id;
    }

    private function suffix() {
        return "child";
    }

    public function label() {
        return self::baseLabel() . ":" . self::suffix();
    }

    public function bump($amount) {
        self::bumpBase($amount);
    }
}

$child = new Child(3);
echo $child->label(), "\n";
$child->bump(5);
echo $child->label(), "\n";

class Ancestor {
    public function label() {
        return "ancestor";
    }

    public function callSelf() {
        return self::label();
    }
}

class Descendant extends Ancestor {
    public function label() {
        return "descendant";
    }
}

$descendant = new Descendant();
echo $descendant->callSelf();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "base:3:child\nbase:8:child\nancestor");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn self_method_calls_report_current_unsupported_boundaries() {
    let top_level = runtime_error(
        r#"<?php
self::make();
"#,
    );

    assert_eq!(top_level.line, 2);
    assert_eq!(top_level.column, 5);
    assert_eq!(
        top_level.message,
        "unsupported call self::make(): self method calls require instance method context"
    );

    let private_parent_method = runtime_error(
        r#"<?php
class Base {
    private function hide() {}
}
class Child extends Base {
    public function call() {
        self::hide();
    }
}
$child = new Child();
$child->call();
"#,
    );

    assert_eq!(private_parent_method.line, 7);
    assert_eq!(private_parent_method.column, 13);
    assert_eq!(
        private_parent_method.message,
        "unsupported call Base::hide(): private method dispatch requires same-class method context"
    );

    let non_static_self_method_without_this = runtime_error(
        r#"<?php
class Box {
    public function make() {}

    public static function call() {
        self::make();
    }
}
Box::call();
"#,
    );

    assert_eq!(non_static_self_method_without_this.line, 6);
    assert_eq!(non_static_self_method_without_this.column, 13);
    assert_eq!(
        non_static_self_method_without_this.message,
        "unsupported call Box::make(): non-static method dispatch through self:: requires current $this object context"
    );
}

#[test]
fn self_and_parent_static_methods_execute_from_class_context() {
    let execution = run_source(
        r#"<?php
class Base {
    public static $count;

    public static function bump($step = 1) {
        self::$count ??= 0;
        self::$count += $step;
        return self::$count;
    }

    protected static function prefix() {
        return self::class;
    }

    public static function label() {
        return self::prefix();
    }
}

class Child extends Base {
    public static function parentBump($step) {
        return parent::bump($step);
    }

    public static function parentPrefix() {
        return parent::prefix();
    }
}

echo Base::bump(), "\n";
echo Child::parentBump(4), "\n";
echo Base::label(), "\n";
echo Child::parentPrefix();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\n5\nBase\nBase");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn named_static_methods_execute_current_public_subset() {
    let execution = run_source(
        r#"<?php
class Counter {
    public static $count;

    public static function bump($step = 1) {
        self::$count ??= 0;
        self::$count += $step;
        return self::$count;
    }
}
class Base {
    public static function name() {
        return self::class;
    }
}
class Child extends Base {}
class Hidden {
    private static $secret;

    public static function setSecret($value) {
        self::$secret = $value;
        return self::$secret;
    }
}

echo Counter::bump(), "\n";
echo Counter::bump(4), "\n";
echo Child::name(), "\n";
echo Hidden::setSecret("ok");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\n5\nBase\nok");
}

#[test]
fn named_static_method_calls_report_current_unsupported_boundaries() {
    let non_static_method = runtime_error(
        r#"<?php
class Box {
    public function make() {}
}
Box::make();
"#,
    );
    assert_eq!(non_static_method.line, 5);
    assert_eq!(non_static_method.column, 1);
    assert_eq!(
        non_static_method.message,
        "Non-static method Box::make() cannot be called statically"
    );

    let private_method = runtime_error(
        r#"<?php
class Box {
    private static function make() {}
}
Box::make();
"#,
    );
    assert_eq!(
        private_method.message,
        "unsupported call Box::make(): private method dispatch requires same-class method context"
    );

    let protected_method = runtime_error(
        r#"<?php
class Box {
    protected static function make() {}
}
Box::make();
"#,
    );
    assert_eq!(
        protected_method.message,
        "unsupported call Box::make(): protected method dispatch requires same-class or child method context"
    );

    let missing_method = runtime_error(
        r#"<?php
class Box {}
Box::missing();
"#,
    );
    assert_eq!(
        missing_method.message,
        "Call to undefined method Box::missing()"
    );

    let missing_class = runtime_error("<?php\nMissing::make();\n");
    assert_eq!(missing_class.message, "Class \"Missing\" not found");
}

#[test]
fn constructor_dispatch_reports_current_unsupported_boundaries() {
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

    let private_constructor = runtime_error(
        r#"<?php
class Box {
    private function __construct() {}
}

$box = new Box();
"#,
    );

    assert_eq!(private_constructor.line, 6);
    assert_eq!(private_constructor.column, 8);
    assert_eq!(
        private_constructor.message,
        "unsupported object instantiation for Box: private constructor Box::__construct() requires same-class construction context"
    );

    let protected_inherited_constructor = runtime_error(
        r#"<?php
class Base {
    protected function __construct() {}
}
class Child extends Base {}

$child = new Child();
"#,
    );

    assert_eq!(protected_inherited_constructor.line, 7);
    assert_eq!(protected_inherited_constructor.column, 10);
    assert_eq!(
        protected_inherited_constructor.message,
        "unsupported object instantiation for Child: protected constructor Base::__construct() requires same-class or child-class construction context"
    );

    let static_constructor = runtime_error(
        r#"<?php
class Box {
    public static function __construct() {}
}

$box = new Box();
"#,
    );

    assert_eq!(static_constructor.line, 6);
    assert_eq!(static_constructor.column, 8);
    assert_eq!(
        static_constructor.message,
        "unsupported object instantiation for Box: static constructors are not implemented"
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
fn duplicate_interface_and_class_names_share_class_like_registry() {
    let duplicate_interface = run_source(
        r#"<?php
interface Hookable {}
interface hookable {}
"#,
    )
    .unwrap_err();

    assert_eq!(duplicate_interface.phase, Phase::Runtime);
    assert_eq!(duplicate_interface.line, 3);
    assert_eq!(duplicate_interface.column, 1);
    assert_eq!(
        duplicate_interface.message,
        "class hookable is already defined"
    );

    let class_then_interface = run_source(
        r#"<?php
class Existing {}
interface existing {}
"#,
    )
    .unwrap_err();

    assert_eq!(class_then_interface.phase, Phase::Runtime);
    assert_eq!(class_then_interface.line, 3);
    assert_eq!(class_then_interface.column, 1);
    assert_eq!(
        class_then_interface.message,
        "class existing is already defined"
    );

    let interface_then_class = run_source(
        r#"<?php
interface Named {}
class named {}
"#,
    )
    .unwrap_err();

    assert_eq!(interface_then_class.phase, Phase::Runtime);
    assert_eq!(interface_then_class.line, 3);
    assert_eq!(interface_then_class.column, 1);
    assert_eq!(
        interface_then_class.message,
        "class named is already defined"
    );

    let duplicate_trait = run_source(
        r#"<?php
trait Reusable {}
trait reusable {}
"#,
    )
    .unwrap_err();

    assert_eq!(duplicate_trait.phase, Phase::Runtime);
    assert_eq!(duplicate_trait.line, 3);
    assert_eq!(duplicate_trait.column, 1);
    assert_eq!(duplicate_trait.message, "class reusable is already defined");

    let class_then_trait = run_source(
        r#"<?php
class Worker {}
trait worker {}
"#,
    )
    .unwrap_err();

    assert_eq!(class_then_trait.phase, Phase::Runtime);
    assert_eq!(class_then_trait.line, 3);
    assert_eq!(class_then_trait.column, 1);
    assert_eq!(class_then_trait.message, "class worker is already defined");

    let interface_then_trait = run_source(
        r#"<?php
interface Contract {}
trait contract {}
"#,
    )
    .unwrap_err();

    assert_eq!(interface_then_trait.phase, Phase::Runtime);
    assert_eq!(interface_then_trait.line, 3);
    assert_eq!(interface_then_trait.column, 1);
    assert_eq!(
        interface_then_trait.message,
        "class contract is already defined"
    );

    let trait_then_interface = run_source(
        r#"<?php
trait NamedTrait {}
interface namedtrait {}
"#,
    )
    .unwrap_err();

    assert_eq!(trait_then_interface.phase, Phase::Runtime);
    assert_eq!(trait_then_interface.line, 3);
    assert_eq!(trait_then_interface.column, 1);
    assert_eq!(
        trait_then_interface.message,
        "class namedtrait is already defined"
    );

    let duplicate_enum = run_source(
        r#"<?php
enum Mode {}
enum mode {}
"#,
    )
    .unwrap_err();

    assert_eq!(duplicate_enum.phase, Phase::Runtime);
    assert_eq!(duplicate_enum.line, 3);
    assert_eq!(duplicate_enum.column, 1);
    assert_eq!(duplicate_enum.message, "class mode is already defined");

    let class_then_enum = run_source(
        r#"<?php
class State {}
enum state {}
"#,
    )
    .unwrap_err();

    assert_eq!(class_then_enum.phase, Phase::Runtime);
    assert_eq!(class_then_enum.line, 3);
    assert_eq!(class_then_enum.column, 1);
    assert_eq!(class_then_enum.message, "class state is already defined");

    let interface_then_enum = run_source(
        r#"<?php
interface Shape {}
enum shape {}
"#,
    )
    .unwrap_err();

    assert_eq!(interface_then_enum.phase, Phase::Runtime);
    assert_eq!(interface_then_enum.line, 3);
    assert_eq!(interface_then_enum.column, 1);
    assert_eq!(
        interface_then_enum.message,
        "class shape is already defined"
    );

    let trait_then_enum = run_source(
        r#"<?php
trait Shared {}
enum shared {}
"#,
    )
    .unwrap_err();

    assert_eq!(trait_then_enum.phase, Phase::Runtime);
    assert_eq!(trait_then_enum.line, 3);
    assert_eq!(trait_then_enum.column, 1);
    assert_eq!(trait_then_enum.message, "class shared is already defined");
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
fn inherited_property_redeclarations_validate_current_compatibility_rules() {
    let valid = run_source(
        r#"<?php
class Base {
    private $privateValue;
    protected $shared;
    public $name;

    public function describeBase() {
        return $this->shared . ":" . $this->name;
    }
}

class Child extends Base {
    public $privateValue;
    public $shared;
    public $name;
}

$child = new Child();
$child->privateValue = "child-private";
$child->shared = "child-shared";
$child->name = "child-name";
echo $child->describeBase(), "\n";
print_r($child);
"#,
    )
    .unwrap();
    assert_eq!(
        valid.stdout,
        "child-shared:child-name\nChild Object\n(\n    [privateValue:Base:private] => \n    [shared] => child-shared\n    [name] => child-name\n    [privateValue] => child-private\n)\n"
    );

    let visibility_error = runtime_error(
        r#"<?php
class Base {
    public $name;
}

class Child extends Base {
    protected $name;
}
"#,
    );
    assert_eq!(visibility_error.line, 6);
    assert_eq!(visibility_error.column, 1);
    assert_eq!(
        visibility_error.message,
        "Access level to Child::$name must be public (as in class Base)"
    );

    let static_error = runtime_error(
        r#"<?php
class Base {
    public static $name;
}

class Child extends Base {
    public $name;
}
"#,
    );
    assert_eq!(static_error.line, 6);
    assert_eq!(static_error.column, 1);
    assert_eq!(
        static_error.message,
        "Cannot redeclare static Base::$name as non static Child::$name"
    );

    let static_error = runtime_error(
        r#"<?php
class Base {
    public $name;
}

class Child extends Base {
    public static $name;
}
"#,
    );
    assert_eq!(static_error.line, 6);
    assert_eq!(static_error.column, 1);
    assert_eq!(
        static_error.message,
        "Cannot redeclare non static Base::$name as static Child::$name"
    );
}

#[test]
fn protected_constructors_execute_from_child_method_context() {
    let execution = run_source(
        r#"<?php
class Base {
    public $id;

    protected function __construct($id = 5) {
        $this->id = $id;
    }

    public function label() {
        return "base:" . $this->id;
    }
}

class Child extends Base {
    public function __construct() {}

    public function makeBase($id) {
        return new Base($id);
    }

    public function makeDefaultBase() {
        return new Base();
    }
}

$child = new Child();
$base = $child->makeBase(12);
echo $base->label(), "\n";
$default = $child->makeDefaultBase();
echo $default->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "base:12\nbase:5");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn class_modifiers_and_abstract_method_signatures_can_register() {
    let execution = run_source(
        r#"<?php
abstract class Base {
    abstract protected function compute();
}

final class Leaf extends Base {
    public function compute() {
        return "ok";
    }
}

$leaf = new Leaf();
echo $leaf->compute();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn abstract_method_implementation_is_enforced_for_concrete_classes() {
    let error = runtime_error(
        r#"<?php
abstract class Base {
    abstract protected function compute();
}

class Child extends Base {}
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: concrete class Child must implement abstract method Base::compute()"
    );

    let own_abstract_error = runtime_error(
        r#"<?php
class Box {
    abstract public function id();
}
"#,
    );

    assert_eq!(own_abstract_error.line, 2);
    assert_eq!(own_abstract_error.column, 1);
    assert_eq!(
        own_abstract_error.message,
        "unsupported class inheritance for Box: concrete class Box must implement abstract method Box::id()"
    );
}

#[test]
fn abstract_child_classes_can_defer_abstract_method_implementation() {
    let execution = run_source(
        r#"<?php
abstract class Base {
    abstract protected function compute();
}

abstract class Mid extends Base {}

class Leaf extends Mid {
    public function compute() {
        return "leaf";
    }
}

$leaf = new Leaf();
echo $leaf->compute();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "leaf");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_method_visibility_compatibility_is_enforced() {
    let execution = run_source(
        r#"<?php
class Base {
    protected function label() {
        return "base";
    }
}

class Child extends Base {
    public function label() {
        return "child";
    }
}

$child = new Child();
echo $child->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child");
    assert_eq!(execution.exit_code, 0);

    let public_error = runtime_error(
        r#"<?php
class Base {
    public function label() {
        return "base";
    }
}

class Child extends Base {
    protected function label() {
        return "child";
    }
}
"#,
    );
    assert_eq!(public_error.line, 9);
    assert_eq!(public_error.column, 15);
    assert_eq!(
        public_error.message,
        "unsupported class inheritance for Child: method Child::label() cannot reduce visibility of inherited public method Base::label()"
    );

    let protected_error = runtime_error(
        r#"<?php
class Base {
    protected function compute() {
        return "base";
    }
}

class Child extends Base {
    private function compute() {
        return "child";
    }
}
"#,
    );
    assert_eq!(protected_error.line, 9);
    assert_eq!(protected_error.column, 13);
    assert_eq!(
        protected_error.message,
        "unsupported class inheritance for Child: method Child::compute() cannot reduce visibility of inherited protected method Base::compute()"
    );
}

#[test]
fn private_parent_methods_do_not_block_child_visibility() {
    let execution = run_source(
        r#"<?php
class Base {
    private function label() {
        return "base";
    }
}

class Child extends Base {
    public function label() {
        return "child";
    }
}

$child = new Child();
echo $child->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_method_static_compatibility_is_enforced() {
    let execution = run_source(
        r#"<?php
class Base {
    public static function label() {
        return "base";
    }
}

class Child extends Base {
    public static function label() {
        return "child";
    }
}

echo Child::label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child");
    assert_eq!(execution.exit_code, 0);

    let static_child_error = runtime_error(
        r#"<?php
class Base {
    public function label() {
        return "base";
    }
}

class Child extends Base {
    public static function label() {
        return "child";
    }
}
"#,
    );
    assert_eq!(static_child_error.line, 9);
    assert_eq!(static_child_error.column, 19);
    assert_eq!(
        static_child_error.message,
        "unsupported class inheritance for Child: cannot redeclare non static method Base::label() as static Child::label()"
    );

    let instance_child_error = runtime_error(
        r#"<?php
class Base {
    public static function compute() {
        return "base";
    }
}

class Child extends Base {
    public function compute() {
        return "child";
    }
}
"#,
    );
    assert_eq!(instance_child_error.line, 9);
    assert_eq!(instance_child_error.column, 12);
    assert_eq!(
        instance_child_error.message,
        "unsupported class inheritance for Child: cannot redeclare static method Base::compute() as non static Child::compute()"
    );
}

#[test]
fn private_parent_methods_do_not_block_child_static_compatibility() {
    let execution = run_source(
        r#"<?php
class Base {
    private function label() {
        return "base";
    }
}

class Child extends Base {
    public static function label() {
        return "child";
    }
}

echo Child::label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_method_required_parameter_compatibility_is_enforced() {
    let execution = run_source(
        r#"<?php
class Base {
    public function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($value, $suffix = "!") {
        return "child:" . $value . $suffix;
    }
}

$child = new Child();
echo $child->label("one"), "\n";
echo $child->label("two", "?");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child:one!\nchild:two?");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Base {
    public function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($prefix, $value) {
        return $prefix . $value;
    }
}
"#,
    );
    assert_eq!(error.line, 9);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: method Child::label() cannot require more parameters than inherited method Base::label()"
    );

    let optional_parent_error = runtime_error(
        r#"<?php
class Base {
    public function compute($value = "base") {
        return $value;
    }
}

class Child extends Base {
    public function compute($value) {
        return $value;
    }
}
"#,
    );
    assert_eq!(optional_parent_error.line, 9);
    assert_eq!(optional_parent_error.column, 12);
    assert_eq!(
        optional_parent_error.message,
        "unsupported class inheritance for Child: method Child::compute() cannot require more parameters than inherited method Base::compute()"
    );
}

#[test]
fn private_parent_methods_do_not_block_child_required_parameter_compatibility() {
    let execution = run_source(
        r#"<?php
class Base {
    private function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($prefix, $value) {
        return $prefix . $value;
    }
}

$child = new Child();
echo $child->label("child:", "ok");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child:ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_method_parameter_type_compatibility_is_enforced() {
    let execution = run_source(
        r#"<?php
class Base {
    public function label(string $value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($value) {
        return "child:" . $value;
    }
}

$child = new Child();
echo $child->label("ok");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "child:ok");
    assert_eq!(execution.exit_code, 0);

    let added_type_error = runtime_error(
        r#"<?php
class Base {
    public function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label(string $value) {
        return "child:" . $value;
    }
}
"#,
    );
    assert_eq!(added_type_error.line, 9);
    assert_eq!(added_type_error.column, 12);
    assert_eq!(
        added_type_error.message,
        "unsupported class inheritance for Child: method Child::label() cannot add parameter type string for parameter $value when inherited method Base::label() has no parameter type"
    );

    let changed_type_error = runtime_error(
        r#"<?php
class Base {
    public function label(string $value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label(int $value) {
        return "child:" . $value;
    }
}
"#,
    );
    assert_eq!(changed_type_error.line, 9);
    assert_eq!(changed_type_error.column, 12);
    assert_eq!(
        changed_type_error.message,
        "unsupported class inheritance for Child: method Child::label() parameter $value type int is incompatible with inherited method Base::label() parameter type string"
    );
}

#[test]
fn inherited_method_return_type_compatibility_is_enforced() {
    let execution = run_source(
        r#"<?php
class Base {
    public function id() {
        return "base";
    }
}

class Child extends Base {
    public function id(): string {
        return "child";
    }
}

$child = new Child();
echo method_exists($child, "id") ? "registered" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);

    let omitted_return_type_error = runtime_error(
        r#"<?php
class Base {
    public function id(): string {
        return "base";
    }
}

class Child extends Base {
    public function id() {
        return "child";
    }
}
"#,
    );
    assert_eq!(omitted_return_type_error.line, 9);
    assert_eq!(omitted_return_type_error.column, 12);
    assert_eq!(
        omitted_return_type_error.message,
        "unsupported class inheritance for Child: method Child::id() must declare return type string to match inherited method Base::id()"
    );

    let changed_return_type_error = runtime_error(
        r#"<?php
class Base {
    public function id(): string {
        return "base";
    }
}

class Child extends Base {
    public function id(): int {
        return 1;
    }
}
"#,
    );
    assert_eq!(changed_return_type_error.line, 9);
    assert_eq!(changed_return_type_error.column, 12);
    assert_eq!(
        changed_return_type_error.message,
        "unsupported class inheritance for Child: method Child::id() return type int is incompatible with inherited method Base::id() return type string"
    );
}

#[test]
fn inherited_and_interface_method_class_type_variance_is_bounded() {
    let execution = run_source(
        r#"<?php
interface HookTarget {}
interface ChildHookTarget extends HookTarget {}
class BaseTarget implements HookTarget {}
class ChildTarget extends BaseTarget implements ChildHookTarget {}

interface ParentResolver {
    public function bind(ChildHookTarget $target): HookTarget;
}

interface ChildResolver extends ParentResolver {
    public function bind(HookTarget $target): ChildHookTarget;
}

interface Resolver {
    public function resolve(ChildTarget $target): BaseTarget;
}

class BaseResolver {
    public function resolve(ChildTarget $target): BaseTarget {
        return $target;
    }
}

class PluginResolver extends BaseResolver implements Resolver {
    public function resolve(BaseTarget $target): ChildTarget {
        return new ChildTarget();
    }
}

class InterfaceResolver implements Resolver {
    public function resolve(HookTarget $target): ChildTarget {
        return new ChildTarget();
    }
}

class InterfaceParentResolver {
    public function bind(ChildHookTarget $target): HookTarget {
        return $target;
    }
}

class InterfaceChildResolver extends InterfaceParentResolver {
    public function bind(HookTarget $target): ChildHookTarget {
        return new ChildTarget();
    }
}

echo method_exists(new PluginResolver(), "resolve") ? "inherited:registered\n" : "inherited:missing\n";
echo method_exists(new InterfaceResolver(), "resolve") ? "interface:registered\n" : "interface:missing\n";
echo interface_exists("ChildResolver") ? "child-interface:registered\n" : "child-interface:missing\n";
echo method_exists(new InterfaceChildResolver(), "bind") ? "interface-parent:registered" : "interface-parent:missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inherited:registered\ninterface:registered\nchild-interface:registered\ninterface-parent:registered"
    );
    assert_eq!(execution.exit_code, 0);

    let invalid_parameter_error = runtime_error(
        r#"<?php
class BaseTarget {}
class ChildTarget extends BaseTarget {}
class OtherTarget {}

class BaseResolver {
    public function resolve(ChildTarget $target) {}
}

class PluginResolver extends BaseResolver {
    public function resolve(OtherTarget $target) {}
}
"#,
    );
    assert_eq!(invalid_parameter_error.line, 11);
    assert_eq!(invalid_parameter_error.column, 12);
    assert_eq!(
        invalid_parameter_error.message,
        "unsupported class inheritance for PluginResolver: method PluginResolver::resolve() parameter $target type OtherTarget is incompatible with inherited method BaseResolver::resolve() parameter type ChildTarget"
    );

    let invalid_return_error = runtime_error(
        r#"<?php
class BaseTarget {}
class ChildTarget extends BaseTarget {}
class OtherTarget {}

interface Resolver {
    public function resolve(): BaseTarget;
}

class PluginResolver implements Resolver {
    public function resolve(): OtherTarget {}
}
"#,
    );
    assert_eq!(invalid_return_error.line, 10);
    assert_eq!(invalid_return_error.column, 1);
    assert_eq!(
        invalid_return_error.message,
        "unsupported class inheritance for PluginResolver: method PluginResolver::resolve() return type OtherTarget is incompatible with interface method Resolver::resolve() return type BaseTarget"
    );
}

#[test]
fn nested_method_visibility_boundary_preserves_registration_timing() {
    let execution = run_source(
        r#"<?php
class Base {
    public function label() {
        return "base";
    }
}

if (false) {
    class Child extends Base {
        protected function label() {
            return "child";
        }
    }
}

echo class_exists("Child") ? "registered" : "not-registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "not-registered");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Base {
    public function label() {
        return "base";
    }
}

if (true) {
    class Child extends Base {
        protected function label() {
            return "child";
        }
    }
}
"#,
    );

    assert_eq!(error.line, 10);
    assert_eq!(error.column, 19);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: method Child::label() cannot reduce visibility of inherited public method Base::label()"
    );
}

#[test]
fn nested_abstract_method_boundary_preserves_registration_timing() {
    let execution = run_source(
        r#"<?php
abstract class Base {
    abstract protected function compute();
}

if (false) {
    class Child extends Base {}
}

echo class_exists("Child") ? "registered" : "not-registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "not-registered");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
abstract class Base {
    abstract protected function compute();
}

if (true) {
    class Child extends Base {}
}
"#,
    );

    assert_eq!(error.line, 7);
    assert_eq!(error.column, 5);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: concrete class Child must implement abstract method Base::compute()"
    );
}

#[test]
fn final_class_declarations_register_and_instantiate_current_subset() {
    let execution = run_source(
        r#"<?php
final class Base {
    public $label = "base";
}

$base = new Base();
echo get_class($base), ":", $base->label;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Base:base");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_final_methods_execute_current_subset() {
    let execution = run_source(
        r#"<?php
class Base {
    final public function label() {
        return "base";
    }
}

class Child extends Base {}

$child = new Child();
echo $child->label();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "base");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn final_method_overrides_report_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
class Base {
    final public function seal() {
        return "base";
    }
}

class Child extends Base {
    public function SEAL() {
        return "child";
    }
}
"#,
    );

    assert_eq!(error.line, 9);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: cannot override final method Base::seal()"
    );
}

#[test]
fn nested_final_method_override_boundary_preserves_registration_timing() {
    let execution = run_source(
        r#"<?php
class Base {
    final public function seal() {
        return "base";
    }
}

if (false) {
    class Child extends Base {
        public function seal() {
            return "child";
        }
    }
}

echo class_exists("Child") ? "registered" : "not-registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "not-registered");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
class Base {
    final public function seal() {
        return "base";
    }
}

if (true) {
    class Child extends Base {
        public function seal() {
            return "child";
        }
    }
}
"#,
    );

    assert_eq!(error.line, 10);
    assert_eq!(error.column, 16);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: cannot override final method Base::seal()"
    );
}

#[test]
fn final_class_inheritance_reports_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
final class Base {}
class Child extends Base {}
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: cannot extend final class Base"
    );
}

#[test]
fn nested_final_parent_inheritance_boundary_preserves_registration_timing() {
    let execution = run_source(
        r#"<?php
final class Base {}
if (false) {
    class Child extends Base {}
}
echo class_exists("Child") ? "registered" : "not-registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "not-registered");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
final class Base {}
if (true) {
    class Child extends Base {}
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 5);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Child: cannot extend final class Base"
    );
}

#[test]
fn abstract_class_instantiation_reports_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
abstract class Base {}
new Base();
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object instantiation for Base: abstract classes are not instantiable in the current subset"
    );
}

#[test]
fn unsupported_object_execution_syntax_is_rejected_with_stable_parse_errors() {
    let object_static_constant = runtime_error(
        r#"<?php
$box::NAME;
"#,
    );
    assert_eq!(object_static_constant.line, 2);
    assert_eq!(object_static_constant.column, 1);
    assert_eq!(
        object_static_constant.message,
        "unsupported call ::NAME: dynamic class constant receiver must be object or class string, got null"
    );

    for supported in [
        r#"<?php
class Box {
    public $name, $email;
}
"#,
        r#"<?php
class Box {
    public const VERSION = 1, NAME = "box";
}
"#,
    ] {
        let execution = run_source(supported).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.stderr, "");
        assert_eq!(execution.exit_code, 0);
    }

    let cases = [
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
trait Logs {
    protected static function write($message) {}
}
"#,
            3,
            22,
            "unsupported trait method declaration: only simple public instance and public static trait methods are implemented; abstract, final, non-public methods, __TRAIT__ context, references/copy-on-write, and native lowering remain unsupported",
        ),
        (
            r#"<?php
trait Logs {
    protected const CHANNEL = "debug";
}
"#,
            3,
            15,
            "unsupported trait constant declaration: only public trait constants are implemented",
        ),
        (
            r#"<?php
trait Logs {
    public const string CHANNEL = "debug";
}
"#,
            3,
            18,
            "unsupported trait constant declaration: typed trait constants are not implemented",
        ),
        (
            r#"<?php
trait Logs {
    public const CHANNEL = "debug", FALLBACK = "info";
}
"#,
            3,
            35,
            "unsupported trait constant declaration: multiple trait constants in one declaration are not implemented",
        ),
        (
            r#"<?php
interface Logger {
    public const string NAME = "logger";
}
"#,
            3,
            18,
            "unsupported interface constant declaration: typed interface constants are not implemented",
        ),
        (
            r#"<?php
enum Status {
    case Draft = "draft";
}
"#,
            3,
            16,
            "unsupported enum case value: backed enum case values are not implemented",
        ),
        (
            r#"<?php
if (true) {
    trait NestedTrait {}
}
"#,
            3,
            5,
            "unsupported trait declaration: only top-level trait declarations are implemented",
        ),
        (
            r#"<?php
if (true) {
    interface NestedLogger {}
}
"#,
            3,
            5,
            "unsupported interface declaration: only top-level interface declarations are implemented",
        ),
        (
            r#"<?php
if (true) {
    enum NestedStatus {}
}
"#,
            3,
            5,
            "unsupported enum declaration: only top-level enum declarations are implemented",
        ),
        (
            r#"<?php
class Box {
    public (Countable&Iterator)|ArrayAccess $id;
}
"#,
            3,
            12,
            "unsupported DNF type declaration: parenthesized union/intersection type declarations are not implemented",
        ),
        (
            r#"<?php
class Value {
    public readonly $id;
}
"#,
            3,
            12,
            "unsupported readonly property declaration: readonly property metadata, initialization rules, write-once enforcement, reflection, and native lowering are not implemented",
        ),
        (
            r#"<?php
class Box {
    public $name = make_name();
}
"#,
            3,
            20,
            "instance property default values only support constant expressions in the current subset",
        ),
        (
            r#"<?php
class Box {
    use Labels, OtherLabels, ThirdLabels {
        label insteadof OtherLabels;
    }
}
"#,
            4,
            9,
            "unsupported trait use adaptation: unqualified insteadof adaptations are not implemented",
        ),
        (
            r#"<?php
class Box {
    private const string NAME = "box";
}
"#,
            3,
            19,
            "unsupported class constant declaration: typed class constants are not implemented",
        ),
        (
            r#"<?php
class Box {
    public static const VERSION = 1;
}
"#,
            3,
            19,
            "unsupported class constant declaration: static class constants are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn instanceof_checks_current_object_relationships() {
    let execution = run_source(
        r#"<?php
class Box {}
class Child extends Box {}
class Other {}

$box = new Box();
$child = new Child();

echo $box instanceof Box ? "1" : "0";
echo $child instanceof Child ? "1" : "0";
echo $child instanceof Box ? "1" : "0";
echo $box instanceof Child ? "1" : "0";
echo $child instanceof Other ? "1" : "0";
echo $child instanceof Missing ? "1" : "0";
echo "x" instanceof Box ? "1" : "0";
echo $child INSTANCEOF box ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11100001");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn instanceof_handles_wordpress_countable_shape_as_false_for_current_values() {
    let execution = run_source(
        r#"<?php
$items = [1, 2, 3];
$name = "Ada";
echo $items instanceof Countable ? "countable" : "plain";
echo "\n";
echo $name instanceof Countable ? "countable" : "plain";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "plain\nplain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn instanceof_supports_dynamic_class_name_variables_and_expressions() {
    let execution = run_source(
        r#"<?php
interface Contract {}
class Box implements Contract {}
class Child extends Box {}

$box = new Box();
$child = new Child();
$class = "Box";
$lower = "box";
$contract = "Contract";
$missing = "Missing";
$targetObject = new Box();

echo $box instanceof $class ? "1" : "0";
echo $child instanceof $class ? "1" : "0";
echo $box instanceof $lower ? "1" : "0";
echo $box instanceof $contract ? "1" : "0";
echo $box instanceof $missing ? "1" : "0";
echo $child instanceof $targetObject ? "1" : "0";
echo $child instanceof ("Box") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1111011");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn instanceof_supports_relative_class_targets_from_class_context() {
    let execution = run_source(
        r#"<?php
class RelativeRoot {}
class RelativeBase extends RelativeRoot {
    public function check($target) {
        echo $target instanceof self ? "S" : "-";
        echo $target instanceof parent ? "P" : "-";
        echo $target instanceof static ? "T" : "-";
        echo "|";
    }
    public static function stat($target) {
        echo $target instanceof self ? "s" : "-";
        echo $target instanceof parent ? "p" : "-";
        echo $target instanceof static ? "t" : "-";
        echo "|";
    }
}
class RelativeChild extends RelativeBase {}

$root = new RelativeRoot();
$base = new RelativeBase();
$child = new RelativeChild();

$base->check($root);
$base->check($base);
$child->check($base);
$child->check($child);
RelativeBase::stat($root);
RelativeChild::stat($child);
$baseName = "RelativeBase";
echo $child instanceof RelativeBase ? "N" : "-";
echo $child instanceof $baseName ? "D" : "-";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "-P-|SPT|SP-|SPT|-p-|spt|ND");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn public_instance_method_dispatch_binds_this_and_preserves_handles() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
    public $count;

    public function label($prefix = "box") {
        return $prefix . ":" . $this->name;
    }

    public function rename($name) {
        $this->name = $name;
        $this->count++;
        return $this->label("renamed");
    }

    public function touch() {
        $this->count = $this->count + 1;
    }
}

$box = new Box();
$box->name = "Ada";
$box->count = 0;
$alias = $box;

echo $box->label("user"), "\n";
echo $box->LABEL(), "\n";
echo $box->rename("Grace"), "\n";
echo $alias->name, "\n";
$box->touch();
echo $box->count;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "user:Ada\nbox:Ada\nrenamed:Grace\nGrace\n2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn private_instance_methods_execute_from_same_class_context() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function label() {
        return $this->prefix() . ":" . $this->name;
    }

    public function labelOther($other) {
        return $other->prefix() . ":" . $other->name;
    }

    private function prefix() {
        return "private";
    }
}

$left = new Box("Ada");
$right = new Box("Grace");
echo $left->label(), "\n";
echo $left->labelOther($right);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "private:Ada\nprivate:Grace");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inherited_public_and_protected_instance_methods_execute_from_child_context() {
    let execution = run_source(
        r#"<?php
class Base {
    public function inherited($other) {
        return "inherited:" . $other->seal();
    }

    public function sameBase($other) {
        return "base:" . $other->seal();
    }

    protected function seal() {
        return "sealed";
    }
}

class Child extends Base {
    public function childCall($other) {
        return "child:" . $other->seal();
    }
}

$base = new Base();
$child = new Child();
echo $base->sameBase($base), "\n";
echo $child->inherited($base), "\n";
echo $child->childCall($base), "\n";
echo $child->childCall($child);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:sealed\ninherited:sealed\nchild:sealed\nchild:sealed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn arrayaccess_reference_return_executes_method_body_and_local_aliases() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = [];
    public $hits = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
        if (!isset($this->items[$offset])) {
            $this->items[$offset] = [];
        }
        $bucket =& $this->items[$offset];
        $leaf = "leaf";
        return $bucket[$leaf];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class MagicBox {
    public $store = [];
    public $hits = [];

    public function &__get($name) {
        $this->hits[] = $name;
        if (!isset($this->store[$name])) {
            $this->store[$name] = [];
        }
        $bucket =& $this->store[$name];
        return $bucket;
    }
}

$source = "seed";
$node = ["value" => &$source, "plain" => ["value" => "copy"]];

$bag = new Bag();
$bag["outer"]["child"] = $node;
$bag->items["outer"]["leaf"]["child"]["value"] = "changed";
$bag->items["outer"]["leaf"]["child"]["plain"]["value"] = "plain-changed";

$alias =& $bag["outer"]["child"];
$alias["value"] = "alias-changed";
$alias["plain"]["value"] = "alias-plain";

$magicSource = "magic-seed";
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$box = new MagicBox();
$box->missing["child"] = $magicNode;
$box->store["missing"]["child"]["value"] = "magic-changed";
$magicAlias =& $box->missing["child"];
$magicAlias["value"] = "magic-alias";
$magicAlias["plain"]["value"] = "magic-plain";

echo $source,
    "|",
    $bag->items["outer"]["leaf"]["child"]["plain"]["value"],
    "|",
    $bag->hits[0],
    "|",
    $bag->hits[1],
    "|",
    $bag["outer"]["child"]["value"],
    "|",
    $magicSource,
    "|",
    $box->store["missing"]["child"]["plain"]["value"],
    "|",
    $box->hits[0],
    "|",
    $box->hits[1];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "alias-changed|alias-plain|outer|outer|alias-changed|magic-alias|magic-plain|missing|missing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn instance_method_dispatch_reports_current_unsupported_boundaries() {
    let non_object = runtime_error(
        r#"<?php
$value = 1;
$value->label();
"#,
    );
    assert_eq!(non_object.line, 3);
    assert_eq!(non_object.column, 1);
    assert_eq!(
        non_object.message,
        "Call to a member function label() on int"
    );

    let missing = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$box->missing();
"#,
    );
    assert_eq!(missing.line, 4);
    assert_eq!(missing.column, 1);
    assert_eq!(missing.message, "Call to undefined method Box::missing()");

    let private_method = runtime_error(
        r#"<?php
class Box {
    private function secret() {}
}
$box = new Box();
$box->secret();
"#,
    );
    assert_eq!(private_method.line, 6);
    assert_eq!(private_method.column, 1);
    assert_eq!(
        private_method.message,
        "unsupported call Box::secret(): private method dispatch requires same-class method context"
    );

    let protected_method = runtime_error(
        r#"<?php
class Box {
    protected function seal() {
        return "sealed";
    }
}
$box = new Box();
$box->seal();
"#,
    );
    assert_eq!(protected_method.line, 8);
    assert_eq!(protected_method.column, 1);
    assert_eq!(
        protected_method.message,
        "unsupported call Box::seal(): protected method dispatch requires same-class or child method context"
    );

    let parent_private_from_child = runtime_error(
        r#"<?php
class Base {
    private function secret() {
        return "base";
    }
}
class Child extends Base {
    public function reveal() {
        return $this->secret();
    }
}
$child = new Child();
$child->reveal();
"#,
    );
    assert_eq!(parent_private_from_child.line, 9);
    assert_eq!(parent_private_from_child.column, 16);
    assert_eq!(
        parent_private_from_child.message,
        "unsupported call Base::secret(): private method dispatch requires same-class method context"
    );

    let static_method = runtime_error(
        r#"<?php
class Box {
    public static function make() {}
}
$box = new Box();
$box->make();
"#,
    );
    assert_eq!(static_method.line, 6);
    assert_eq!(static_method.column, 1);
    assert_eq!(
        static_method.message,
        "unsupported call Box::make(): static method dispatch through object receivers is not implemented"
    );

    let top_level_this = runtime_error(
        r#"<?php
echo $this;
"#,
    );
    assert_eq!(top_level_this.line, 2);
    assert_eq!(top_level_this.column, 1);
    assert_eq!(
        top_level_this.message,
        "Using $this when not in object context"
    );
}

#[test]
fn uncaught_exception_fatal_renders_message_and_throw_site() {
    let source = r#"<?php
throw new Exception("boom");
"#;

    let execution = run_source_with_source_file(source, "/tmp/uncaught_exception.php").unwrap();
    assert_eq!(execution.exit_code, 255);
    assert_eq!(execution.stderr, "");
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Exception: boom in /tmp/uncaught_exception.php:2\nStack trace:\n#0 {main}\n  thrown in /tmp/uncaught_exception.php on line 2"
    );
}

#[test]
fn throwing_finally_overrides_pending_exception_for_uncaught_fatal() {
    let source = r#"<?php
try {
    $e = new Exception("outer");
    try {
        throw new Exception("inner", 0, $e);
    } finally {
        throw $e;
    }
} finally {}
"#;

    let execution = run_source_with_source_file(source, "/tmp/finally_override.php").unwrap();
    assert_eq!(execution.exit_code, 255);
    assert_eq!(execution.stderr, "");
    assert!(execution
        .stdout
        .contains("Fatal error: Uncaught Exception: outer in /tmp/finally_override.php:"));
    assert!(execution
        .stdout
        .contains("\nStack trace:\n#0 {main}\n  thrown in /tmp/finally_override.php on line "));
    assert!(!execution.stdout.contains("inner"));
}

#[test]
fn spl_object_storage_identity_map_offsets_iteration_and_clone() {
    let source = r#"<?php
class ChildStorage extends SplObjectStorage {
    public function current(): object {
        return parent::current();
    }
}

$store = new ChildStorage();
$first = new stdClass();
$second = new stdClass();
$missing = new stdClass();

$store[$first] = "one";
$store->attach($second, "two");
echo $store->offsetGet($first), "|", $store[$second], "\n";
var_dump(isset($store[$first]));
var_dump(isset($store[$missing]));
var_dump(empty($store[$missing]));
echo count($store), "|", $store->count(), "\n";

foreach ($store as $key => $object) {
    echo $key, ":", $store->offsetGet($object), "\n";
}

$copy = clone $store;
echo "copy=", count($copy), "\n";
$store->removeAll($copy);
echo "after=", count($store), "|", count($copy), "\n";

$store[$first] ??= "again";
echo $store[$first], "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "one|two\nbool(true)\nbool(false)\nbool(true)\n2|2\n0:one\n1:two\ncopy=2\nafter=0|2\nagain\n"
    );
}

#[test]
fn spl_object_storage_get_hash_return_type_errors_are_php_shaped() {
    let source = r#"<?php
class BadHashStorage extends SplObjectStorage {
    #[ReturnTypeWillChange]
    public function getHash($object) {
        return 2;
    }
}

$store = new BadHashStorage();
$object = new stdClass();
try {
    $store[$object] = "value";
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "TypeError: BadHashStorage::getHash(): Return value must be of type string, int returned"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_object_storage_unset_array_syntax_preserves_iterator_key() {
    let source = r#"<?php
function named_object($name) {
    $object = new stdClass();
    $object->name = $name;
    return $object;
}

$syntax = new SplObjectStorage();
$a = named_object("a");
$b = named_object("b");
$c = named_object("c");
$syntax[$a] = "a";
$syntax[$b] = "b";
$syntax[$c] = "c";
$syntax->next();
unset($syntax[$a]);
echo "syntax:", $syntax->key(), ":", $syntax->current()->name, "\n";
$syntax->next();
echo "syntax:", $syntax->key(), ":", $syntax->current()->name, "\n";
$syntax->next();
echo "syntax:", $syntax->key(), ":", $syntax->valid() ? "valid" : "invalid", "\n";

$method = new SplObjectStorage();
$ma = named_object("a");
$mb = named_object("b");
$mc = named_object("c");
$method[$ma] = "a";
$method[$mb] = "b";
$method[$mc] = "c";
$method->next();
$method->detach($ma);
echo "method:", $method->key(), ":", $method->current()->name, "\n";
$method->next();
echo "method:", $method->key(), ":", $method->current()->name, "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "syntax:1:b\nsyntax:2:c\nsyntax:3:invalid\nmethod:0:b\nmethod:1:c\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn iterator_helpers_materialize_arrays_and_bounded_iterators() {
    let source = r#"<?php
$array = array("a" => 1, "b" => 2, 5 => 3);
print_r(iterator_to_array($array));
print_r(iterator_to_array($array, false));
echo "count-array=", iterator_count($array), "\n";

$it = new ArrayIterator(array("x" => "ex", "y" => "why"));
print_r(iterator_to_array($it));
echo "after-arrayiterator=", $it->valid() ? "valid" : "invalid", "\n";

$again = new ArrayIterator(array("x" => "ex", "y" => "why"));
print_r(iterator_to_array($again, false));

$ao = new ArrayObject(array("p" => 7, "q" => 8));
print_r(iterator_to_array($ao, false));

$counted = new ArrayIterator(array(10, 20));
echo "count-iterator=", iterator_count($counted), "|", ($counted->valid() ? "valid" : "invalid"), "\n";

try {
    iterator_count("bad");
} catch (Throwable $e) {
    echo $e::class, ":", $e->getMessage(), "\n";
}

try {
    iterator_to_array(array(1), array());
} catch (Throwable $e) {
    echo $e::class, ":", $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [a] => 1\n    [b] => 2\n    [5] => 3\n)\nArray\n(\n    [0] => 1\n    [1] => 2\n    [2] => 3\n)\ncount-array=3\nArray\n(\n    [x] => ex\n    [y] => why\n)\nafter-arrayiterator=invalid\nArray\n(\n    [0] => ex\n    [1] => why\n)\nArray\n(\n    [0] => 7\n    [1] => 8\n)\ncount-iterator=2|invalid\nTypeError:iterator_count(): Argument #1 ($iterator) must be of type Traversable|array, string given\nTypeError:iterator_to_array(): Argument #2 ($preserve_keys) must be of type bool, array given"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_fixed_array_offsets_iteration_resize_static_constructor_and_errors() {
    let source = r#"<?php
class ChildFixedArray extends SplFixedArray {
    public function offsetGet($index): mixed {
        return parent::offsetGet($index);
    }
    public function count(): int {
        return parent::count();
    }
}

$items = new ChildFixedArray(3);
$items[0] = "zero";
$items["1"] = "one";
$items->offsetSet(2, "two");
echo $items[0], "|", $items->offsetGet(1), "|", $items["2"], "\n";
var_dump(isset($items[0]));
var_dump(isset($items[9]));
echo count($items), "|", $items->getSize(), "|", $items->count(), "\n";
foreach ($items as $key => $value) {
    echo $key, ":", $value, "\n";
}
$items->setSize(2);
echo count($items), "|", $items->offsetExists(2) ? "bad" : "trimmed", "\n";
$copy = clone $items;
$copy[1] = "copy";
echo $items[1], "|", $copy[1], "\n";
$from = SplFixedArray::fromArray(array(0 => "a", 2 => "c"));
print_r($from->toArray());
try {
    $items[] = "append";
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    echo $items[9];
} catch (OutOfBoundsException $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "zero|one|two\nbool(true)\nbool(false)\n3|3|3\n0:zero\n1:one\n2:two\n2|trimmed\none|copy\nArray\n(\n    [0] => a\n    [1] => \n    [2] => c\n)\n[] operator not supported for SplFixedArray\nOutOfBoundsException:Index invalid or out of range\n"
    );
}

#[test]
fn spl_fixed_array_var_dump_uses_runtime_storage_slots() {
    let source = r#"<?php
$from = SplFixedArray::fromArray(array(1 => "one", 3 => false));
var_dump($from);

class FixedDumpChild extends SplFixedArray {
    public $label = "declared";
}

$child = new FixedDumpChild(2);
$child[0] = "slot";
var_dump($child);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object(SplFixedArray)#1 (4) {\n  [0]=>\n  NULL\n  [1]=>\n  string(3) \"one\"\n  [2]=>\n  NULL\n  [3]=>\n  bool(false)\n}\nobject(FixedDumpChild)#2 (3) {\n  [0]=>\n  string(4) \"slot\"\n  [1]=>\n  NULL\n  [\"label\"]=>\n  string(8) \"declared\"\n}\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_fixed_array_offset_unset_finalizes_released_slot_before_var_dump() {
    let source = r#"<?php
class FixedArrayUnsetDrop {
    function __destruct() {
        global $arr;
        $arr->setSize(0);
    }
}

$arr = new SplFixedArray(2);
$arr[0] = new FixedArrayUnsetDrop;
unset($arr[0]);
var_dump($arr);

class FixedArraySharedDrop {
    function __destruct() {
        echo "shared destructed\n";
    }
}

$shared = new FixedArraySharedDrop;
$arr = new SplFixedArray(2);
$arr[0] = $shared;
$arr[1] = $shared;
unset($arr[0]);
echo "after shared unset\n";
var_dump($arr);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object(SplFixedArray)#1 (0) {\n}\nafter shared unset\nobject(SplFixedArray)#4 (2) {\n  [0]=>\n  NULL\n  [1]=>\n  object(FixedArraySharedDrop)#3 (0) {\n  }\n}\nshared destructed\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_fixed_array_print_r_null_size_exists_and_nested_iteration_edges() {
    let source = r#"<?php
$nullConstruct = new SplFixedArray(null);
print_r($nullConstruct);

$sizeNull = new SplFixedArray(2);
$sizeNull->setSize(null);
var_dump($sizeNull);

$items = new SplFixedArray(2);
$items[0] = "Value 1";
$items[1] = "Value 2";
$items->setSize(4);
$items[2] = "Value 3";
$items[3] = "Value 4";
print_r($items);
$items->setSize(3);
print_r($items);

class MyFixed extends SplFixedArray {
    public function offsetGet($key): mixed {
        return "prefix_" . parent::offsetGet($key);
    }
}
$overridden = new MyFixed(1);
var_dump(isset($overridden[0]));
$overridden[0] = "abc";
var_dump(isset($overridden[0]));
var_dump($overridden[0]);

try {
    new SplFixedArray(new SplFixedArray(3));
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

echo "nested\n";
$nested = SplFixedArray::fromArray([0, 1]);
foreach ($nested as $value1) {
    foreach ($nested as $value2) {
        echo "$value1 $value2\n";
    }
}

echo "shrink\n";
$shrink = SplFixedArray::fromArray(["a", "b", "c"]);
foreach ($shrink as $key => $value) {
    echo "$key => $value\n";
    if ($key == 0) {
        $shrink->setSize(2);
    }
}

$indirect = new SplFixedArray(1);
$indirect[0][] = 3;
var_dump($indirect);
"#;

    let execution = run_source(source).unwrap();
    assert!(execution.stdout.contains(
        "Deprecated: SplFixedArray::__construct(): Passing null to parameter #1 ($size) of type int is deprecated"
    ));
    assert!(execution.stdout.contains(
        "Deprecated: SplFixedArray::setSize(): Passing null to parameter #1 ($size) of type int is deprecated"
    ));
    assert!(execution.stdout.contains("SplFixedArray Object\n(\n)\n"));
    assert!(execution
        .stdout
        .contains("object(SplFixedArray)#2 (0) {\n}\n"));
    assert!(execution.stdout.contains(
        "SplFixedArray Object\n(\n    [0] => Value 1\n    [1] => Value 2\n    [2] => Value 3\n    [3] => Value 4\n)\nSplFixedArray Object\n(\n    [0] => Value 1\n    [1] => Value 2\n    [2] => Value 3\n)\n"
    ));
    assert!(execution
        .stdout
        .contains("bool(false)\nbool(true)\nstring(10) \"prefix_abc\"\n"));
    assert!(execution.stdout.contains(
        "SplFixedArray::__construct(): Argument #1 ($size) must be of type int, SplFixedArray given\n"
    ));
    assert!(execution.stdout.contains("nested\n0 0\n0 1\n1 0\n1 1\n"));
    assert!(execution.stdout.contains("shrink\n0 => a\n1 => b\n"));
    assert!(execution.stdout.contains(
        "Notice: Indirect modification of overloaded element of SplFixedArray has no effect"
    ));
    assert!(execution.stdout.contains("(1) {\n  [0]=>\n  NULL\n}\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_object_array_iterator_offsets_iteration_clone_and_sort() {
    let source = r#"<?php
$ao = new ArrayObject(array('b' => 2, 'a' => 1));
$ao[] = 3;
$ao['c'] = null;
var_dump($ao->offsetExists('c'));
echo count($ao), "|", $ao['a'], "\n";
unset($ao['c']);

foreach ($ao as $key => $value) {
    echo $key, ":", $value, "\n";
}

$copy = clone $ao;
$copy['a'] = 'copy';
echo $ao['a'], "|", $copy['a'], "\n";

$ao->ksort();
foreach ($ao->getArrayCopy() as $key => $value) {
    echo $key, "=", $value, "\n";
}

$it = new ArrayIterator(array('x' => 'ex', 'y' => 'why'));
foreach ($it as $key => $value) {
    echo $key, ":", $value, "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\n4|1\nb:2\na:1\n0:3\n1|copy\n0=3\na=1\nb=2\nx:ex\ny:why\n"
    );
}

#[test]
fn array_object_user_sort_methods_use_comparators_and_guard_reentrant_mutation() {
    let source = r#"<?php
function desc_cmp($left, $right) {
    if ($left == $right) {
        return 0;
    }
    return $left < $right ? 1 : -1;
}

$values = new ArrayObject(array(2, 3, 1));
var_dump($values->uasort("desc_cmp"));
foreach ($values as $key => $value) {
    echo "v:$key=$value;";
}
echo "\n";

$keys = new ArrayObject(array(3 => 0, 2 => 1, 5 => 2, 6 => 3, 1 => 4));
var_dump($keys->uksort("desc_cmp"));
foreach ($keys as $key => $value) {
    echo "k:$key=$value;";
}
echo "\n";

try {
    $values->uasort();
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}

try {
    $keys->uksort("desc_cmp", "extra");
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}

$guarded = new ArrayObject(array(1, 2, 3));
$i = 0;
$guarded->uasort(function ($left, $right) use ($guarded, &$i) {
    if ($i++ == 0) {
        try {
            $guarded->exchangeArray(array(4, 5, 6));
        } catch (Error $e) {
            echo $e->getMessage(), "\n";
        }
        echo "guard:", count($guarded), ":", $guarded[0], "\n";
    }
    return $left <=> $right;
});

ini_set("disable_functions", "asort, ksort, natsort, natcasesort, uasort, uksort");
$disabled = new ArrayObject(array("hello", "world"));
try {
    $disabled->asort();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $disabled->ksort();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $disabled->natsort();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $disabled->natcasesort();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $values->uasort("desc_cmp");
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $keys->uksort("desc_cmp");
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nv:1=3;v:0=2;v:2=1;\nbool(true)\nk:6=3;k:5=2;k:3=0;k:2=1;k:1=4;\nArrayObject::uasort() expects exactly 1 argument, 0 given\nArrayObject::uksort() expects exactly 1 argument, 2 given\nModification of ArrayObject during sorting is prohibited\nguard:3:1\nCannot call method asort when function asort is disabled\nCannot call method ksort when function ksort is disabled\nCannot call method natsort when function natsort is disabled\nCannot call method natcasesort when function natcasesort is disabled\nCannot call method uasort when function uasort is disabled\nCannot call method uksort when function uksort is disabled\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_object_disabled_sort_fatal_records_internal_method_frame() {
    let execution = run_source(
        r#"<?php
ini_set("disable_functions", "asort");
$ao = new ArrayObject(array(2, 1));
$ao->asort();
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Fatal error: Uncaught Error: Cannot call method asort when function asort is disabled in Command line code:4"
    ));
    assert!(execution
        .stdout
        .contains("#0 Command line code(4): ArrayObject->asort()"));
    assert!(execution.stdout.contains("#1 {main}"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn array_iterator_seek_out_of_range_errors_are_catchable() {
    let source = r#"<?php
$it = new ArrayIterator(array(0, 1, 2));
$it->seek(1);
echo $it->key(), ":", $it->current(), "\n";

try {
    $it->seek(-1);
} catch (OutOfBoundsException $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

try {
    $it->seek(3);
} catch (Exception $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$it->seek(2);
echo $it->key(), ":", $it->current(), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "1:1\nOutOfBoundsException:Seek position -1 is out of range\nOutOfBoundsException:Seek position 3 is out of range\n2:2\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_object_nested_storage_mutations_reach_inner_array_object() {
    let source = r#"<?php
$base = new ArrayObject(array(1 => "one", 2 => "two", 3 => "three"));
$base[] = "four";
$copy = new ArrayObject($base);
$copy[] = "five";
$copy[6] = "six";
unset($copy[2]);

foreach ($base as $key => $value) {
    echo $key, "=", $value, "\n";
}
print_r($copy->getArrayCopy());

$it = new ArrayIterator(new stdClass());
try {
    $it->append("bad");
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert!(execution.stdout.contains(
        "Deprecated: ArrayObject::__construct(): Using an object as a backing array for ArrayObject is deprecated"
    ));
    assert!(execution.stdout.contains(
        "Deprecated: ArrayIterator::__construct(): Using an object as a backing array for ArrayIterator is deprecated"
    ));
    assert!(execution.stdout.contains(
        "1=one\n3=three\n4=four\n5=five\n6=six\nArray\n(\n    [1] => one\n    [3] => three\n    [4] => four\n    [5] => five\n    [6] => six\n)\n"
    ));
    assert!(execution.stdout.ends_with(
        "Cannot append properties to objects, use ArrayIterator::offsetSet() instead\n"
    ));
}

#[test]
fn array_object_array_as_props_and_iterator_class_metadata() {
    let source = r#"<?php
class ChildArrayObject extends ArrayObject {
    public $p = "object";
    private $x = "secret";

    static function inside($value) {
        return $value->x;
    }
}

class ChildArrayIterator extends ArrayIterator {
    function rewind(): void {
        parent::rewind();
    }

    function valid(): bool {
        return parent::valid();
    }

    function current(): mixed {
        return parent::current();
    }

    function key(): string|int|null {
        return parent::key();
    }

    function next(): void {
        parent::next();
    }
}

$ao = new ChildArrayObject(array("p" => "array", "x" => "public"));
$ao->setFlags(ArrayObject::ARRAY_AS_PROPS);
echo $ao->p, "\n";
unset($ao->p);
echo $ao->p, "\n";
$ao->p = "changed";
echo $ao["p"], "\n";
echo ChildArrayObject::inside($ao), "\n";
echo $ao->x, "\n";

$iterable = new ArrayObject(array("a" => 1), 0, "ChildArrayIterator");
echo $iterable->getIteratorClass(), "\n";
echo get_class($iterable->getIterator()), "\n";
foreach ($iterable as $key => $value) {
    echo $key, "=", $value, "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object\narray\nchanged\nsecret\npublic\nChildArrayIterator\nChildArrayIterator\na=1\n"
    );
}

#[test]
fn array_object_foreach_retires_unrooted_iterator_class_temporaries() {
    let source = r#"<?php
class ReusedHandleIterator extends ArrayIterator {
    function rewind(): void {
        parent::rewind();
    }
    function valid(): bool {
        return parent::valid();
    }
    function current(): mixed {
        return parent::current();
    }
    function key(): string|int|null {
        return parent::key();
    }
    function next(): void {
        parent::next();
    }
}

class CapturingIterator extends ReusedHandleIterator {
    function rewind(): void {
        $GLOBALS["captured_iterator"] = $this;
        parent::rewind();
    }
}

$ao = new ArrayObject(array("a" => 1, "b" => 2), 0, "ReusedHandleIterator");
$first = $ao->getIterator();
echo spl_object_id($first), "\n";
foreach ($ao as $key => $value) {}
$second = $ao->getIterator();
echo spl_object_id($second), "\n";

$capturing = new ArrayObject(array("x" => 1), 0, "CapturingIterator");
foreach ($capturing as $value) {}
$after_capture = $capturing->getIterator();
echo spl_object_id($captured_iterator), "|", spl_object_id($after_capture), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "2\n3\n5|6\n");
}

#[test]
fn spl_doubly_linked_list_iteration_offsets_and_exceptions() {
    let source = r#"<?php
$list = new SplDoublyLinkedList();
$list->push("a");
$list->push("b");
$list->add(1, "x");
echo $list->count(), "|", $list->bottom(), "|", $list->top(), "\n";
var_dump($list->offsetExists(0));
var_dump($list->offsetExists(3));
echo $list->offsetGet(1), "\n";
$list->offsetUnset(1);
echo $list->offsetGet(1), "\n";
$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
echo $list->getIteratorMode(), "\n";
$list->rewind();
while ($list->valid()) {
    echo $list->key(), ":", $list->current(), "\n";
    $list->next();
}
try {
    $list->add([], "bad");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    $list->offsetUnset(99);
} catch (OutOfRangeException $e) {
    echo $e->getMessage(), "\n";
}
try {
    (new SplDoublyLinkedList())->bottom();
} catch (RuntimeException $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3|a|b\nbool(true)\nbool(false)\nx\nb\n2\n1:b\n0:a\nSplDoublyLinkedList::add(): Argument #1 ($index) must be of type int, array given\nSplDoublyLinkedList::offsetUnset(): Argument #1 ($index) is out of range\nCan't peek at an empty datastructure\n"
    );
}

#[test]
fn spl_doubly_linked_list_foreach_keeps_rewind_direction() {
    let source = r#"<?php
$list = new SplDoublyLinkedList();
$list->push(1);
$list->push(2);
$list->push(3);

$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO);
foreach ($list as $item) {
    $list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
    echo $item, "\n";
}
echo "***\n";
$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
foreach ($list as $item) {
    $list->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO);
    echo $item, "\n";
}

try {
    (new SplQueue())->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}
try {
    (new SplStack())->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO);
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "1\n2\n3\n***\n3\n2\n1\nIterators' LIFO/FIFO modes for SplStack/SplQueue objects are frozen\nIterators' LIFO/FIFO modes for SplStack/SplQueue objects are frozen\n"
    );
}

#[test]
fn spl_doubly_linked_list_prev_moves_against_active_direction() {
    let source = r#"<?php
$list = new SplDoublyLinkedList();
$list->push(1);
$list->push(2);
$list->push(3);
$list->push(4);

$list->rewind();
$list->prev();
var_dump($list->current());
$list->rewind();
var_dump($list->current());
$list->next();
var_dump($list->current());
$list->next();
$list->next();
var_dump($list->current());
$list->prev();
var_dump($list->current());

$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
$list->rewind();
var_dump($list->current());
$list->next();
var_dump($list->current());
$list->prev();
var_dump($list->current());

$empty = new SplDoublyLinkedList();
$empty->rewind();
$empty->prev();
var_dump($empty->current());
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "NULL\nint(1)\nint(2)\nint(4)\nint(3)\nint(4)\nint(3)\nint(4)\nNULL\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_doubly_linked_list_count_arrayaccess_and_delete_iteration() {
    let source = r#"<?php
class CountedList extends SplDoublyLinkedList {
    public function count(): int {
        return -parent::count();
    }
}

$list = new SplDoublyLinkedList();
$list->push(null);
$list->push(null);
echo count($list), "|", $list->count(), "\n";
var_dump($list->pop());
var_dump($list->pop());

$list[] = "append";
$list[0] = "first";
echo count($list), "|", $list[0], "\n";

$list->push("second");
$list->push("third");
$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO | SplDoublyLinkedList::IT_MODE_DELETE);
foreach ($list as $key => $value) {
    echo "$key=$value;";
}
echo "\n", count($list), "\n";

$lifo = new SplDoublyLinkedList();
$lifo->push("a");
$lifo->push("b");
$lifo->push("c");
$lifo->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO | SplDoublyLinkedList::IT_MODE_DELETE);
foreach ($lifo as $key => $value) {
    echo "$key=$value;";
}
echo "\n", count($lifo), "\n";

$counted = new CountedList();
$counted[] = "one";
$counted[] = "two";
echo count($counted), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "2|2\nNULL\nNULL\n1|first\n0=first;0=second;0=third;\n0\n2=c;1=b;0=a;\n0\n-2\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_file_object_local_file_line_cursor_methods() {
    use std::fs;

    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-spl-file-object-{}", std::process::id()));
    fs::create_dir_all(&fixture_dir).unwrap();
    let fixture = fixture_dir.join("cursor.php");
    fs::write(&fixture, "<?php\n//line 2\n//line 3\n//line 4\n?>\n").unwrap();

    let source = r#"<?php
$file = new SplFileObject(__FILE__);
echo $file->current();
$file->seek(2);
echo $file->key(), ":", $file->current();
echo $file->current();
$file->next();
echo $file->key(), ":", $file->current();
$file->seek(20);
var_dump($file->valid());
$file->rewind();
var_dump($file->valid());
foreach ($file as $key => $line) {
    if ($key > 1) {
        break;
    }
    echo $key, "=", $line;
}
try {
    $file->seek(-1);
} catch (ValueError $e) {
    echo "caught:", $e->getMessage(), "\n";
}
"#;

    let execution = run_source_with_source_file(source, fixture.display().to_string()).unwrap();
    assert_eq!(
        execution.stdout,
        "<?php\n2://line 3\n//line 3\n3://line 4\nbool(false)\nbool(true)\n0=<?php\n1=//line 2\ncaught:SplFileObject::seek(): Argument #1 ($line) must be greater than or equal to 0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_file_object_flags_and_csv_controls_use_local_line_state() {
    use std::fs;

    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-spl-file-object-csv-{}", std::process::id()));
    fs::create_dir_all(&fixture_dir).unwrap();
    let fixture = fixture_dir.join("records.csv");
    fs::write(&fixture, "'green apples'|10\n'yellow bananas'|20\n").unwrap();
    let fixture_path = fixture.display().to_string().replace('\\', "\\\\");

    let source = format!(
        r#"<?php
$file = new SplFileObject("{fixture_path}");
$file->setFlags(SplFileObject::DROP_NEW_LINE);
var_dump($file->getFlags());
echo $file->current(), "|";
var_dump($file->getCsvControl());
$file->setFlags(SplFileObject::READ_CSV);
$file->setCsvControl("|", "'", "");
var_dump($file->getFlags());
var_dump($file->getCsvControl());
foreach ($file as $row) {{
    echo $row[0], "=", $row[1], "\n";
}}
$file->rewind();
var_dump($file->fgetcsv());
try {{
    $file->setCsvControl("||");
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}
"#
    );

    let execution = run_source(&source).unwrap();
    assert_eq!(
        execution.stdout,
        r#"int(1)
'green apples'|10|array(3) {
  [0]=>
  string(1) ","
  [1]=>
  string(1) """
  [2]=>
  string(1) "\"
}
int(8)
array(3) {
  [0]=>
  string(1) "|"
  [1]=>
  string(1) "'"
  [2]=>
  string(0) ""
}
green apples=10
yellow bananas=20
array(2) {
  [0]=>
  string(12) "green apples"
  [1]=>
  string(2) "10"
}
SplFileObject::setCsvControl(): Argument #1 ($separator) must be a single character
"#
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_file_object_csv_escape_diagnostics_and_reflection_metadata() {
    use std::fs;

    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-spl-file-object-csv-params-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let input = fixture_dir.join("records.csv");
    let output = fixture_dir.join("write.csv");
    fs::write(&input, "first,second\n").unwrap();
    let input_path = input.display().to_string().replace('\\', "\\\\");
    let output_path = output.display().to_string().replace('\\', "\\\\");

    let source = format!(
        r#"<?php
$reader = new SplFileObject("{input_path}");
var_dump($reader->fgetcsv());
$control = new SplFileObject("{input_path}");
$control->setCsvControl();
$writer = new SplFileObject("{output_path}", "w");
try {{
    $writer->fputcsv(array("water", "fruit"), ",,", "\"");
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}
try {{
    $writer->fputcsv(array("water", "fruit"), ",", "\"\"");
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}

$method = new ReflectionMethod("SplFileObject", "setCsvControl");
foreach ($method->getParameters() as $param) {{
    echo $param->getName(), "|";
}}
echo "\n";
$method = new ReflectionMethod("SplFileObject", "fputcsv");
foreach ($method->getParameters() as $param) {{
    echo $param->getName(), "|";
}}
echo "\n";
"#
    );

    let execution = run_source(&source).unwrap();
    assert!(execution.stdout.contains(
        "Deprecated: SplFileObject::fgetcsv(): the $escape parameter must be provided, as its default value will change, either explicitly or via SplFileObject::setCsvControl() in Command line code on line 3"
    ));
    assert!(execution.stdout.contains(
        "Deprecated: SplFileObject::setCsvControl(): the $escape parameter must be provided as its default value will change in Command line code on line 5"
    ));
    assert!(execution.stdout.contains(
        "SplFileObject::fputcsv(): Argument #2 ($separator) must be a single character\n"
    ));
    assert!(execution.stdout.contains(
        "SplFileObject::fputcsv(): Argument #3 ($enclosure) must be a single character\n"
    ));
    assert!(execution
        .stdout
        .contains("separator|enclosure|escape|\nfields|separator|enclosure|escape|eol|\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_empty_infinite_and_limit_iterators_wrap_bounded_iterators() {
    let source = r#"<?php
class EmptyIteratorEx extends EmptyIterator {
    function rewind(): void {
        echo __METHOD__ . "\n";
        parent::rewind();
    }
    function valid(): false {
        echo __METHOD__ . "\n";
        return parent::valid();
    }
}

class ArrayIteratorEx extends ArrayIterator {
    function rewind(): void {
        echo __METHOD__ . "\n";
        parent::rewind();
    }
    function valid(): bool {
        echo __METHOD__ . "\n";
        return parent::valid();
    }
    function current(): mixed {
        echo __METHOD__ . "\n";
        return parent::current();
    }
    function key(): string|int|null {
        echo __METHOD__ . "\n";
        return parent::key();
    }
    function next(): void {
        echo __METHOD__ . "\n";
        parent::next();
    }
}

echo "empty\n";
foreach (new EmptyIteratorEx() as $value) {
    echo "unreachable";
}

echo "infinite\n";
$it = new InfiniteIterator(new ArrayIteratorEx(range(0, 2)));
$pos = 0;
foreach ($it as $value) {
    echo "value=$value\n";
    if ($pos++ > 5) {
        break;
    }
}

echo "limit-empty\n";
foreach (new LimitIterator(new EmptyIterator(), 0, 3) as $key => $value) {
    echo "$key=>$value\n";
}

echo "limit-infinite\n";
$it = new ArrayIterator(array(0 => "A", 1 => "B", 2 => "C", 3 => "D"));
$it = new LimitIterator(new InfiniteIterator($it), 2, 5);
foreach ($it as $key => $value) {
    echo "$key=>$value\n";
}

echo "nested\n";
$it = new ArrayIterator(array(0 => "A", 1 => "B", 2 => "C", 3 => "D"));
$it = new LimitIterator(new InfiniteIterator(new LimitIterator($it, 1, 2)), 2, 5);
foreach ($it as $key => $value) {
    echo "$key=>$value\n";
}

try {
    new LimitIterator(new ArrayIterator(array(1)), -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}

try {
    foreach (new LimitIterator(new ArrayIterator(array("x")), 3) as $value) {
        echo $value;
    }
} catch (OutOfBoundsException $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "empty\nEmptyIteratorEx::rewind\nEmptyIteratorEx::valid\ninfinite\nArrayIteratorEx::rewind\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=0\nArrayIteratorEx::next\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=1\nArrayIteratorEx::next\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=2\nArrayIteratorEx::next\nArrayIteratorEx::valid\nArrayIteratorEx::rewind\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=0\nArrayIteratorEx::next\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=1\nArrayIteratorEx::next\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=2\nArrayIteratorEx::next\nArrayIteratorEx::valid\nArrayIteratorEx::rewind\nArrayIteratorEx::valid\nArrayIteratorEx::current\nArrayIteratorEx::key\nvalue=0\nlimit-empty\nlimit-infinite\n2=>C\n3=>D\n0=>A\n1=>B\n2=>C\nnested\n1=>B\n2=>C\n1=>B\n2=>C\n1=>B\nLimitIterator::__construct(): Argument #2 ($offset) must be greater than or equal to 0\nSeek position 3 is out of range\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_file_object_byte_reads_passthru_and_max_line_length() {
    use std::fs;

    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-spl-file-object-read-{}", std::process::id()));
    fs::create_dir_all(&fixture_dir).unwrap();
    let fixture = fixture_dir.join("read.txt");
    fs::write(&fixture, "0\n1\n2").unwrap();
    let fixture_path = fixture.display().to_string().replace('\\', "\\\\");

    let source = format!(
        r#"<?php
$file = new SplFileObject("{fixture_path}");
var_dump($file->key());
var_dump($file->fgetc());
var_dump($file->key(), $file->eof());
var_dump($file->fgetc());
var_dump($file->key(), $file->eof());
var_dump($file->fread(1));
var_dump($file->key());
var_dump($file->fread(99));
var_dump($file->key(), $file->eof());
var_dump($file->fgetc());

$again = new SplFileObject("{fixture_path}");
var_dump($again->fpassthru());
var_dump($again->eof(), $again->key());

$limited = new SplFileObject("{fixture_path}");
$limited->setMaxLineLen(1);
var_dump($limited->getMaxLineLen());
var_dump($limited->getCurrentLine());
try {{
    $limited->setMaxLineLen(-1);
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}
"#
    );

    let execution = run_source(&source).unwrap();
    assert_eq!(
        execution.stdout,
        "int(0)\nstring(1) \"0\"\nint(0)\nbool(false)\nstring(1) \"\n\"\nint(1)\nbool(false)\nstring(1) \"1\"\nint(1)\nstring(2) \"\n2\"\nint(2)\nbool(true)\nbool(false)\n0\n1\n2int(5)\nbool(true)\nint(2)\nint(1)\nstring(1) \"0\"\nSplFileObject::setMaxLineLen(): Argument #1 ($maxLength) must be greater than or equal to 0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_file_object_writable_modes_fwrite_and_fputcsv_use_local_stream_state() {
    use std::fs;

    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-spl-file-object-write-{}", std::process::id()));
    fs::create_dir_all(&fixture_dir).unwrap();
    let fixture = fixture_dir.join("write.csv");
    let fixture_path = fixture.display().to_string().replace('\\', "\\\\");

    let source = format!(
        r#"<?php
$file = "{fixture_path}";
$object = new SplFileObject($file, "w+");
$object->setCsvControl("|", "'", "");
var_dump($object->fputcsv(array("a|b", "c")));
var_dump($object->ftell(), $object->eof());
var_dump($object->fwrite("tail", 2));
var_dump($object->fflush());
var_dump(file_get_contents($file));
"#
    );

    let execution = run_source(&source).unwrap();
    assert_eq!(
        execution.stdout,
        "int(8)\nint(8)\nbool(false)\nint(2)\nbool(true)\nstring(10) \"'a|b'|c\nta\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_doubly_linked_list_debug_info_and_uncaught_type_error_shape() {
    let source = r#"<?php
$list = new SplDoublyLinkedList();
$list->push(1);
$list->push(2);
$list->push(3);
print_r($list);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "SplDoublyLinkedList Object\n(\n    [flags:SplDoublyLinkedList:private] => 0\n    [dllist:SplDoublyLinkedList:private] => Array\n        (\n            [0] => 1\n            [1] => 2\n            [2] => 3\n        )\n\n)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let source = r#"<?php
$list = new SplDoublyLinkedList();
$list->offsetGet("fail");
"#;
    let execution = run_source(source).unwrap();
    assert_eq!(execution.exit_code, 255);
    assert!(execution.stdout.contains(
        "Fatal error: Uncaught TypeError: SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, string given in Command line code:3"
    ));
    assert!(!execution.stdout.contains("called in"));
}
