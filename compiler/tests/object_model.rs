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
fn class_exists_requires_string_name_and_bool_autoload_arguments() {
    let name_error = runtime_error("<?php\nvar_dump(class_exists(42));\n");

    assert_eq!(name_error.line, 2);
    assert_eq!(name_error.column, 10);
    assert_eq!(
        name_error.message,
        "unsupported call class_exists(): class name argument must be string, got int"
    );

    let autoload_error = runtime_error("<?php\nvar_dump(class_exists(\"Box\", 1));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call class_exists(): autoload argument must be bool in the current subset, got int"
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

    let autoload_error = runtime_error("<?php\nvar_dump(interface_exists(\"Box\", 1));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call interface_exists(): autoload argument must be bool in the current subset, got int"
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

    let autoload_error = runtime_error("<?php\nvar_dump(trait_exists(\"Box\", 1));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call trait_exists(): autoload argument must be bool in the current subset, got int"
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

    let autoload_error = runtime_error("<?php\nvar_dump(enum_exists(\"Box\", 1));\n");

    assert_eq!(autoload_error.line, 2);
    assert_eq!(autoload_error.column, 10);
    assert_eq!(
        autoload_error.message,
        "unsupported call enum_exists(): autoload argument must be bool in the current subset, got int"
    );
}

#[test]
fn property_exists_checks_declared_property_metadata() {
    let source = r#"<?php
class Box {
    public $name;
    protected $secret;
    private static $cache;
}

$box = new box();
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
        "object:name\nobject:secret\nobject:static\nclass:static\nclass:missing\nmissing-class:false\ndynamic:exists\n"
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
class Box {
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
        "Array\n(\n    [name] => \n    [shared] => \n)\n2|1|1\n2|"
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
$vars = get_object_vars($box);
print_r($vars);
echo count($vars), "|", $vars["name"], "|", $vars["count"], "|", array_key_exists("secret", $vars), "\n";

$call = "get_object_vars";
$dynamic = $call($box);
echo count($dynamic), "|", $dynamic["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => Ada\n    [count] => 3\n)\n2|Ada|3|\n2|Ada"
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
fn get_mangled_object_vars_lists_current_public_instance_property_values() {
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
print_r($vars);
echo count($vars), "|", $vars["name"], "|", $vars["count"], "|", array_key_exists("secret", $vars), "\n";

$call = "get_mangled_object_vars";
$dynamic = $call($box);
echo count($dynamic), "|", $dynamic["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => Ada\n    [count] => 3\n)\n2|Ada|3|\n2|Ada"
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
fn is_a_checks_exact_current_class_relationships() {
    let source = r#"<?php
class Box {}
class Crate {}

$box = new box();
if (is_a($box, "Box")) {
    echo "object:box\n";
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
if (is_a("BOX", "box", true)) {
    echo "string:allowed\n";
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
        "object:box\nobject:case-insensitive\nobject:not-crate\nstring:default-false\nstring:allowed\nmissing-source:false\nmissing-target:false\ndynamic:object\n"
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
fn is_subclass_of_reports_false_without_inheritance_metadata() {
    let source = r#"<?php
class Box {}
class Crate {}

$box = new box();
if (!is_subclass_of($box, "Box")) {
    echo "object:exact-false\n";
}
if (!is_subclass_of($box, "Crate")) {
    echo "object:other-false\n";
}
if (!is_subclass_of("Box", "Box")) {
    echo "string:default-false\n";
}
if (!is_subclass_of("BOX", "box", true)) {
    echo "string:allowed-exact-false\n";
}
if (!is_subclass_of("Missing", "Box", true)) {
    echo "missing-source:false\n";
}
if (!is_subclass_of($box, "Missing")) {
    echo "missing-target:false\n";
}
$call = "is_subclass_of";
if (!$call($box, "BOX")) {
    echo "dynamic:false\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:exact-false\nobject:other-false\nstring:default-false\nstring:allowed-exact-false\nmissing-source:false\nmissing-target:false\ndynamic:false\n"
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
fn get_parent_class_reports_false_without_inheritance_metadata() {
    let source = r#"<?php
class Box {}

$box = new box();
if (!get_parent_class($box)) {
    echo "object:false\n";
}
if (!get_parent_class("BOX")) {
    echo "string:false\n";
}
$call = "get_parent_class";
if (!$call($box)) {
    echo "dynamic:false";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object:false\nstring:false\ndynamic:false"
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
fn spl_object_id_has_stable_boundary_until_object_handles_exist() {
    let error = runtime_error("<?php\nclass Box {}\nvar_dump(spl_object_id(new Box()));\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported call spl_object_id(): PHP object handle identity is not implemented in the current subset"
    );

    let dynamic_error = runtime_error(
        "<?php\nclass Box {}\n$call = \"spl_object_id\";\nvar_dump($call(new Box()));\n",
    );

    assert_eq!(dynamic_error.line, 4);
    assert_eq!(dynamic_error.column, 10);
    assert_eq!(
        dynamic_error.message,
        "unsupported call spl_object_id(): PHP object handle identity is not implemented in the current subset"
    );
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
fn spl_object_hash_has_stable_boundary_until_object_handles_exist() {
    let error = runtime_error("<?php\nclass Box {}\nvar_dump(spl_object_hash(new Box()));\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported call spl_object_hash(): PHP object handle hash is not implemented in the current subset"
    );

    let dynamic_error = runtime_error(
        "<?php\nclass Box {}\n$call = \"spl_object_hash\";\nvar_dump($call(new Box()));\n",
    );

    assert_eq!(dynamic_error.line, 4);
    assert_eq!(dynamic_error.column, 10);
    assert_eq!(
        dynamic_error.message,
        "unsupported call spl_object_hash(): PHP object handle hash is not implemented in the current subset"
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
            || error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_is_object_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\nclass Box {}\necho is_object(new Box());\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("class declarations")
            || error.message.contains("object instantiation")
            || error.message.contains("function calls"),
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
            || error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_class_exists_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho class_exists(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_interface_exists_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho interface_exists(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_trait_exists_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho trait_exists(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_enum_exists_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho enum_exists(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_property_exists_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho property_exists(\"Box\", \"name\");\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_method_exists_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho method_exists(\"Box\", \"open\");\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_class_methods_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho get_class_methods(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_class_vars_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_class_vars(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
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
            || error.message.contains("function calls"),
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
            || error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_is_a_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho is_a(\"Box\", \"Box\", true);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_is_subclass_of_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho is_subclass_of(\"Box\", \"Box\", true);\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_parent_class_until_native_object_lowering_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\necho get_parent_class(\"Box\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_classes_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_declared_classes();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
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
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_declared_traits_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_declared_traits();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_get_called_class_until_native_object_lowering_exists() {
    let error = php_compiler::emit_ir_source("<?php\necho get_called_class();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
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
            || error.message.contains("function calls"),
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
            || error.message.contains("function calls"),
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
