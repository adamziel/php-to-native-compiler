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
fn class_inheritance_metadata_reports_unsupported_boundaries() {
    let missing_parent = runtime_error("<?php\nclass Child extends Missing {}\n");
    assert_eq!(missing_parent.line, 2);
    assert_eq!(missing_parent.column, 1);
    assert_eq!(missing_parent.message, "undefined class Missing");

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
fn child_context_protected_property_access_remains_explicitly_unsupported() {
    let source = r#"<?php
class Base {
    protected $shared;

    public function seedBase($shared) {
        $this->shared = $shared;
    }
}

class Child extends Base {
    public function readShared() {
        return $this->shared;
    }
}

$child = new Child();
$child->seedBase("base-shared");
echo $child->readShared();
"#;

    let error = runtime_error(source);
    assert_eq!(error.line, 12);
    assert_eq!(error.column, 16);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Child::$shared requires same-class method context in the current subset"
    );
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
        "unsupported call empty(): only direct variables, direct array offset operands, and direct object property operands are supported"
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
fn interface_exists_reports_false_for_current_no_interface_model() {
    let source = r#"<?php
class Box {}

if (!interface_exists("Box")) {
    echo "class:not-interface\n";
}
if (!interface_exists("Missing")) {
    echo "missing:not-interface\n";
}
if (!interface_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}
$call = "interface_exists";
if (!$call("Box", true)) {
    echo "dynamic:not-interface\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "class:not-interface\nmissing:not-interface\nmissing:false-autoload\ndynamic:not-interface\n"
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
fn trait_exists_reports_false_for_current_no_trait_model() {
    let source = r#"<?php
class Box {}

if (!trait_exists("Box")) {
    echo "class:not-trait\n";
}
if (!trait_exists("Missing")) {
    echo "missing:not-trait\n";
}
if (!trait_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}
$call = "trait_exists";
if (!$call("Box", true)) {
    echo "dynamic:not-trait\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "class:not-trait\nmissing:not-trait\nmissing:false-autoload\ndynamic:not-trait\n"
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
fn enum_exists_reports_false_for_current_no_enum_model() {
    let source = r#"<?php
class Box {}

if (!enum_exists("Box")) {
    echo "class:not-enum\n";
}
if (!enum_exists("Missing")) {
    echo "missing:not-enum\n";
}
if (!enum_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}
$call = "enum_exists";
if (!$call("Box", true)) {
    echo "dynamic:not-enum\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "class:not-enum\nmissing:not-enum\nmissing:false-autoload\ndynamic:not-enum\n"
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
echo count($declared), "|", $declared[0], "|", $declared[1], "\n";

$call = "get_declared_classes";
$dynamic = $call();
echo $dynamic[0], "|", $dynamic[1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => Box\n    [1] => Profile\n)\n2|Box|Profile\nBox|Profile"
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
fn get_declared_interfaces_reports_empty_interface_table() {
    let source = r#"<?php
class Box {}

$declared = get_declared_interfaces();
print_r($declared);
echo count($declared), "\n";

$call = "get_declared_interfaces";
$dynamic = $call();
echo count($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "Array\n(\n)\n0\n0");
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
fn get_declared_traits_reports_empty_trait_table() {
    let source = r#"<?php
class Box {}

$declared = get_declared_traits();
print_r($declared);
echo count($declared), "\n";

$call = "get_declared_traits";
$dynamic = $call();
echo count($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "Array\n(\n)\n0\n0");
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
fn get_called_class_has_stable_boundary_until_class_context_exists() {
    let error = runtime_error("<?php\nvar_dump(get_called_class());\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported call get_called_class(): method and static class context are not implemented in the current subset"
    );

    let dynamic_error = runtime_error("<?php\n$call = \"get_called_class\";\nvar_dump($call());\n");

    assert_eq!(dynamic_error.line, 3);
    assert_eq!(dynamic_error.column, 10);
    assert_eq!(
        dynamic_error.message,
        "unsupported call get_called_class(): method and static class context are not implemented in the current subset"
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

    let static_parent_method = runtime_error(
        r#"<?php
class Base {
    public static function make() {}
}
class Child extends Base {
    public function call() {
        parent::make();
    }
}
$child = new Child();
$child->call();
"#,
    );

    assert_eq!(static_parent_method.line, 7);
    assert_eq!(static_parent_method.column, 15);
    assert_eq!(
        static_parent_method.message,
        "unsupported call Base::make(): static method dispatch through parent:: is not implemented"
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

    let static_self_method = runtime_error(
        r#"<?php
class Box {
    public static function make() {}

    public function call() {
        self::make();
    }
}
$box = new Box();
$box->call();
"#,
    );

    assert_eq!(static_self_method.line, 6);
    assert_eq!(static_self_method.column, 13);
    assert_eq!(
        static_self_method.message,
        "unsupported call Box::make(): static method dispatch through self:: is not implemented"
    );
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
fn unsupported_object_execution_syntax_is_rejected_with_stable_parse_errors() {
    let cases = [
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
            "unsupported self static property access: static property storage is not implemented",
        ),
        (
            r#"<?php
self::VERSION;
"#,
            2,
            5,
            "unsupported self class constant access: class constants are not implemented",
        ),
        (
            r#"<?php
self::class;
"#,
            2,
            5,
            "unsupported self class name constant: self::class resolution is not implemented",
        ),
        (
            r#"<?php
parent::$value;
"#,
            2,
            7,
            "unsupported parent static property access: static property storage is not implemented",
        ),
        (
            r#"<?php
parent::VERSION;
"#,
            2,
            7,
            "unsupported parent class constant access: class constants are not implemented",
        ),
        (
            r#"<?php
parent::class;
"#,
            2,
            7,
            "unsupported parent class name constant: parent::class resolution is not implemented",
        ),
        (
            r#"<?php
static::class;
"#,
            2,
            7,
            "unsupported static class name constant: static::class resolution is not implemented",
        ),
        (
            r#"<?php
static::$value;
"#,
            2,
            7,
            "unsupported static:: property access: late static binding and static property storage are not implemented",
        ),
        (
            r#"<?php
static::make();
"#,
            2,
            7,
            "unsupported static:: method call: late static binding and static method dispatch are not implemented",
        ),
        (
            r#"<?php
static::VERSION;
"#,
            2,
            7,
            "unsupported static:: class constant access: late static binding and class constants are not implemented",
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
