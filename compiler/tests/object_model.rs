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
    assert_eq!(classes.classes().len(), 8);
    assert_eq!(classes.classes()[0].name(), "Exception");
    assert_eq!(classes.classes()[1].name(), "stdClass");
    assert_eq!(classes.classes()[2].name(), "mysqli");
    assert_eq!(classes.classes()[3].name(), "mysqli_result");
    assert_eq!(classes.classes()[4].name(), "mysqli_stmt");
    assert_eq!(classes.classes()[5].name(), "PDO");
    assert_eq!(classes.classes()[6].name(), "PDOStatement");

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
    assert_eq!(missing.column, 8);
    assert_eq!(missing.message, "undefined class Missing");
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
        "loaded\ncharge\nfallback\nroot-append\nArray\n(\n    [translation.mo] => Array\n    (\n        [en_US] => Array\n        (\n            [default] => loaded\n            [0] => fallback\n        )\n        [fr_FR] => Array\n        (\n            [default] => charge\n        )\n    )\n    [0] => root-append\n)\n"
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
        "fallback\ncharge\nArray\n(\n    [translation.mo] => Array\n    (\n        [en_US] => Array\n        (\n            [fallback] => fallback\n        )\n        [fr_FR] => Array\n        (\n            [default] => charge\n        )\n    )\n)\n"
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
fn keyword_method_names_remain_explicitly_unsupported() {
    let error = parse_error(
        r#"<?php
$data = new stdClass();
$data->public();
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported keyword method call: keyword method names after '->' are not implemented"
    );
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
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
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
    let error = runtime_error("<?php\necho get_class(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call get_class(): argument must be object, got int"
    );
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
fn class_exists_requires_string_name_and_bool_autoload_arguments() {
    let name_error = runtime_error("<?php\nvar_dump(class_exists(42));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call class_exists(): class name argument must be string, got int"
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
echo in_array("Stringable", get_declared_interfaces(), true) ? "declared\n" : "not-declared\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "interface\ninstanceof\nchild:is-a\nchild:subclass\nexplicit:is-a\nplain:no\ndeclared\n"
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

final class WP_Hook implements Iterator, ArrayAccess {}
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
fn core_interface_catalog_reports_bounded_internal_interfaces() {
    let execution = run_source(
        r#"<?php
foreach (array("Traversable", "IteratorAggregate", "Iterator", "Serializable", "ArrayAccess", "Countable", "Stringable") as $name) {
    echo interface_exists($name) ? $name . ":yes\n" : $name . ":no\n";
}

echo interface_exists("DefinitelyMissingInterface") ? "missing:yes" : "missing:no";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Traversable:yes\nIteratorAggregate:yes\nIterator:yes\nSerializable:yes\nArrayAccess:yes\nCountable:yes\nStringable:yes\nmissing:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn interface_exists_requires_string_name_and_bool_autoload_arguments() {
    let name_error = runtime_error("<?php\nvar_dump(interface_exists(42));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call interface_exists(): interface name argument must be string, got int"
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
fn trait_exists_requires_string_name_and_bool_autoload_arguments() {
    let name_error = runtime_error("<?php\nvar_dump(trait_exists(42));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call trait_exists(): trait name argument must be string, got int"
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
    let name_error = runtime_error("<?php\nvar_dump(enum_exists(42));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call enum_exists(): enum name argument must be string, got int"
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
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call property_exists(): object_or_class argument must be object or string, got int"
    );

    let property_error = runtime_error("<?php\nvar_dump(property_exists(\"Box\", 42));\n");

    assert_eq!(property_error.line, 2);
    assert_eq!(property_error.column, 10);
    assert_eq!(
        property_error.message,
        "unsupported call property_exists(): property argument must be string in the current subset, got int"
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
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call method_exists(): object_or_class argument must be object or string, got int"
    );

    let method_error = runtime_error("<?php\nvar_dump(method_exists(\"Box\", 42));\n");

    assert_eq!(method_error.line, 2);
    assert_eq!(method_error.column, 10);
    assert_eq!(
        method_error.message,
        "unsupported call method_exists(): method argument must be string in the current subset, got int"
    );
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
fn get_class_methods_requires_object_or_declared_class_string_argument() {
    let target_error = runtime_error("<?php\nvar_dump(get_class_methods(42));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call get_class_methods(): object_or_class argument must be object or declared class string, got int"
    );

    let missing_class_error = runtime_error("<?php\nvar_dump(get_class_methods(\"Missing\"));\n");

    assert_eq!(missing_class_error.line, 2);
    assert_eq!(missing_class_error.column, 10);
    assert_eq!(
        missing_class_error.message,
        "unsupported call get_class_methods(): string argument must name a declared class in the current subset"
    );
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
        "Array\n(\n    [name] => \n    [shared] => \n    [baseName] => \n    [baseShared] => \n)\n4|1|1\n4|"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_class_vars_requires_declared_class_string_argument() {
    let target_error = runtime_error("<?php\nvar_dump(get_class_vars(42));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call get_class_vars(): class name argument must be string, got int"
    );

    let missing_class_error = runtime_error("<?php\nvar_dump(get_class_vars(\"Missing\"));\n");

    assert_eq!(missing_class_error.line, 2);
    assert_eq!(missing_class_error.column, 10);
    assert_eq!(
        missing_class_error.message,
        "unsupported call get_class_vars(): string argument must name a declared class in the current subset"
    );
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
    let source_error = runtime_error("<?php\nvar_dump(is_subclass_of(42, \"Box\"));\n");

    assert_eq!(source_error.line, 2);
    assert_eq!(source_error.column, 10);
    assert_eq!(
        source_error.message,
        "unsupported call is_subclass_of(): object_or_class argument must be object or string, got int"
    );

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
    let target_error = runtime_error("<?php\nvar_dump(get_parent_class(42));\n");

    assert_eq!(target_error.line, 2);
    assert_eq!(target_error.column, 10);
    assert_eq!(
        target_error.message,
        "unsupported call get_parent_class(): object_or_class argument must be object or string, got int"
    );

    let missing_class_error = runtime_error("<?php\nvar_dump(get_parent_class(\"Missing\"));\n");

    assert_eq!(missing_class_error.line, 2);
    assert_eq!(missing_class_error.column, 10);
    assert_eq!(
        missing_class_error.message,
        "unsupported call get_parent_class(): string argument must name a declared class in the current subset"
    );
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
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => Exception\n    [1] => stdClass\n    [2] => mysqli\n    [3] => mysqli_result\n    [4] => mysqli_stmt\n    [5] => PDO\n    [6] => PDOStatement\n    [7] => Box\n    [8] => Profile\n)\n9|Exception|stdClass|mysqli\nException|stdClass|mysqli"
    );
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
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => Exception\n    [1] => stdClass\n    [2] => mysqli\n    [3] => mysqli_result\n    [4] => mysqli_stmt\n    [5] => PDO\n    [6] => PDOStatement\n    [7] => App\\Mode\n    [8] => App\\Status\n)\n9\n"
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
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => Traversable\n    [1] => IteratorAggregate\n    [2] => Iterator\n    [3] => Serializable\n    [4] => ArrayAccess\n    [5] => Countable\n    [6] => Stringable\n    [7] => App\\Logger\n    [8] => App\\Hookable\n)\n9\n9"
    );
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
    assert_eq!(undefined_class.message, "undefined class Missing");

    let missing_method = runtime_error(
        r#"<?php
class Box {}
$class = "Box";
$class::make();
"#,
    );
    assert_eq!(missing_method.message, "undefined function Box::make()");

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
        "unsupported call Box::make(): non-static method dispatch through dynamic static receivers is not implemented"
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
    assert_eq!(arity_error.column, 10);
    assert_eq!(
        arity_error.message,
        "arity mismatch for spl_object_id(): expected 1 argument(s), got 0"
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
fn clone_expression_rejects_non_objects_and_declared_clone_methods() {
    let type_error = runtime_error("<?php\n$copy = clone 42;\n");

    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 9);
    assert_eq!(
        type_error.message,
        "unsupported call clone: clone operand must be object in the current subset, got int"
    );

    let clone_method_error = runtime_error(
        r#"<?php
class Box {
    public function __clone() {
        echo "clone";
    }
}
$box = new Box();
$copy = clone $box;
"#,
    );

    assert_eq!(clone_method_error.line, 8);
    assert_eq!(clone_method_error.column, 9);
    assert_eq!(
        clone_method_error.message,
        "unsupported call clone: __clone dispatch is not implemented"
    );
}

#[test]
fn spl_object_hash_requires_one_object_argument() {
    let arity_error = runtime_error("<?php\nvar_dump(spl_object_hash());\n");

    assert_eq!(arity_error.line, 2);
    assert_eq!(arity_error.column, 10);
    assert_eq!(
        arity_error.message,
        "arity mismatch for spl_object_hash(): expected 1 argument(s), got 0"
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_folds_absent_native_class_interface_trait_enum_exists_calls() {
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

    assert_eq!(ir.matches("c\"0\\00\"").count(), 5, "{ir}");
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
fn emit_ir_folds_absent_native_property_and_method_exists_calls() {
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

    assert_eq!(ir.matches("c\"0\\00\"").count(), 4, "{ir}");
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
        error.message.contains("object metadata builtins"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_class_vars_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_class_vars(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins")
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
        error.message.contains("object metadata builtins"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_classes_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_declared_classes();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object metadata builtins"),
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
        error.message.contains("object metadata builtins"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_traits_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_declared_traits();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object metadata builtins"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_called_class_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_called_class();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
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
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("object metadata builtins"),
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
        error.message.contains("object/class lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_self_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nself::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object/class lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_named_static_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nBox::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object/class lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_late_static_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nstatic::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object/class lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_static_method_calls_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\n$box::make();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object/class lowering rejects"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_object_static_properties_until_native_object_lowering_exists() {
    let read_error = php_compiler::emit_ir_source("<?php\n$box::$value;\n").unwrap_err();
    assert_eq!(read_error.phase, Phase::Codegen);
    assert!(
        read_error.message.contains("object/class lowering rejects"),
        "{}",
        read_error.message
    );

    let write_error = php_compiler::emit_ir_source("<?php\n$box::$value = 1;\n").unwrap_err();
    assert_eq!(write_error.phase, Phase::Codegen);
    assert!(
        write_error.message.contains("mutation lowering rejects"),
        "{}",
        write_error.message
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
        assert!(
            error.message.contains("object/class lowering rejects"),
            "{}",
            error.message
        );
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
    assert_eq!(undefined_class.message, "undefined class Missing");

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
fn emit_ir_rejects_class_constants_until_native_object_lowering_exists() {
    for source in [
        "<?php\necho Box::VERSION;\n",
        "<?php\nself::VERSION;\n",
        "<?php\nparent::VERSION;\n",
        "<?php\nstatic::VERSION;\n",
    ] {
        let error = php_compiler::emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("object/class lowering rejects"),
            "{}",
            error.message
        );
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
    assert_eq!(undefined_class.message, "undefined class Missing");

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
    assert_eq!(undefined_class.message, "undefined class Missing");
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
        assert!(
            error.message.contains("object/class lowering rejects"),
            "{}",
            error.message
        );
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
    assert_eq!(error.column, 8);
    assert_eq!(error.message, "undefined class Missing");
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
    assert_eq!(non_static_method.column, 4);
    assert_eq!(
        non_static_method.message,
        "unsupported call Box::make(): non-static method dispatch through named static receivers is not implemented"
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
    assert_eq!(missing_method.message, "undefined function Box::missing()");

    let missing_class = runtime_error("<?php\nMissing::make();\n");
    assert_eq!(missing_class.message, "undefined class Missing");
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
    assert_eq!(visibility_error.line, 7);
    assert_eq!(visibility_error.column, 15);
    assert_eq!(
        visibility_error.message,
        "unsupported class inheritance for Child: property Child::$name cannot reduce visibility of inherited public property Base::$name"
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
    assert_eq!(static_error.line, 7);
    assert_eq!(static_error.column, 12);
    assert_eq!(
        static_error.message,
        "unsupported class inheritance for Child: cannot redeclare static property Base::$name as non static Child::$name"
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
    assert_eq!(static_error.line, 7);
    assert_eq!(
        static_error.message,
        "unsupported class inheritance for Child: cannot redeclare non static property Base::$name as static Child::$name"
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
    let cases = [
        (
            r#"<?php
$box::NAME;
"#,
            2,
            5,
            "unsupported object static class constant access: object receiver class constants are not implemented",
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
trait Logs {
    public function write($message) {}
}
"#,
            3,
            5,
            "unsupported trait member declaration: trait members and trait use execution are not implemented",
        ),
        (
            r#"<?php
interface Logger {
    const NAME = "logger";
}
"#,
            3,
            5,
            "unsupported interface constant declaration: interface constants are not implemented",
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
class Box {
    public static int $count;
}
"#,
            3,
            19,
            "unsupported static property type declaration: typed static property metadata, uninitialized state, and write enforcement are not implemented",
        ),
        (
            r#"<?php
class Box {
    private static ?string $name = null;
}
"#,
            3,
            20,
            "unsupported static property type declaration: typed static property metadata, uninitialized state, and write enforcement are not implemented",
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
        (
            r#"<?php
class Box {
    public const VERSION = 1, NAME = "box";
}
"#,
            3,
            29,
            "unsupported class constant declaration: multiple class constants in one declaration are not implemented",
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
fn instanceof_rejects_dynamic_class_names_for_now() {
    let error = parse_error(
        r#"<?php
class Box {}
$box = new Box();
$class = "Box";
echo $box instanceof $class;
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 22);
    assert_eq!(
        error.message,
        "unsupported instanceof class expression: dynamic class names are not implemented"
    );
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
        "unsupported call label(): receiver must be object, got int"
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
    assert_eq!(missing.message, "undefined function Box::missing()");

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
    assert_eq!(top_level_this.column, 6);
    assert_eq!(
        top_level_this.message,
        "unsupported call $this: object context is only available during instance method execution"
    );
}
