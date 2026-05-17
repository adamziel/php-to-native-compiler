use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn unset_direct_array_offsets_removes_existing_keys_and_ignores_missing_keys() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items["city"] = "Paris";
$items["2"] = "two";
$items[] = "next-before";

unset($items["name"]);
unset($items["missing"]);
unset($items[2]);
$items[] = "next-after";

echo "count:", count($items), "\n";
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
if (isset($items["name"])) {
    echo "name:set\n";
} else {
    echo "name:unset\n";
}
if (array_key_exists(2, $items)) {
    echo "two:set\n";
} else {
    echo "two:unset\n";
}
if (array_key_exists(4, $items)) {
    echo "append:4";
} else {
    echo "append:other";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "count:3\ncity=Paris\n3=next-before\n4=next-after\nname:unset\ntwo:unset\nappend:4"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_array_offset_treats_null_or_undefined_targets_as_noop() {
    let source = r#"<?php
$nullable = null;
unset($nullable["missing"]);
unset($undefined["missing"]);

if (isset($nullable)) {
    echo "nullable:set\n";
} else {
    echo "nullable:unset\n";
}
if (isset($undefined)) {
    echo "undefined:set";
} else {
    echo "undefined:unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "nullable:unset\nundefined:unset");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_nested_array_offsets_removes_existing_keys_and_ignores_missing_paths() {
    let source = r#"<?php
$options = [];
$options["group"]["first"] = "one";
$options["group"]["second"] = "two";
$options["other"] = null;

unset($options["group"]["first"]);
unset($options["group"]["missing"]);
unset($options["missing"]["child"]);
unset($options["other"]["child"]);
unset($undefined["missing"]["child"]);

if (array_key_exists("first", $options["group"])) {
    echo "first:set\n";
} else {
    echo "first:unset\n";
}
echo "second:", $options["group"]["second"], "\n";
if (isset($undefined)) {
    echo "undefined:set";
} else {
    echo "undefined:unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "first:unset\nsecond:two\nundefined:unset");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_array_offsets_detaches_covered_reference_aliases() {
    let source = r#"<?php
$items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
$alias =& $items["slot"];
$leaf =& $items["outer"]["leaf"];

unset($items["slot"]);
echo array_key_exists("slot", $items) ? "slot:set" : "slot:unset";
echo "|alias=", $alias, "\n";
$alias = "after";
echo array_key_exists("slot", $items) ? "slot:set:" . $items["slot"] : "slot:unset";
echo "|alias=", $alias, "\n";

unset($items["outer"]);
echo array_key_exists("outer", $items) ? "outer:set" : "outer:unset";
echo "|leaf=", $leaf, "\n";
$leaf = "changed";
echo array_key_exists("outer", $items) ? "outer:set" : "outer:unset";
echo "|leaf=", $leaf;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "slot:unset|alias=seed\nslot:unset|alias=after\nouter:unset|leaf=nested\nouter:unset|leaf=changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_object_property_array_offsets_detaches_covered_reference_aliases() {
    let source = r#"<?php
class RefcowUnsetAliasBag {
    public $items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
}

$bag = new RefcowUnsetAliasBag();
$alias =& $bag->items["slot"];
$leaf =& $bag->items["outer"]["leaf"];

unset($bag->items["slot"]);
echo array_key_exists("slot", $bag->items) ? "slot:set" : "slot:unset";
echo "|alias=", $alias, "\n";
$alias = "after";
echo array_key_exists("slot", $bag->items) ? "slot:set:" . $bag->items["slot"] : "slot:unset";
echo "|alias=", $alias, "\n";

unset($bag->items["outer"]);
echo array_key_exists("outer", $bag->items) ? "outer:set" : "outer:unset";
echo "|leaf=", $leaf, "\n";
$leaf = "changed";
echo array_key_exists("outer", $bag->items) ? "outer:set" : "outer:unset";
echo "|leaf=", $leaf;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "slot:unset|alias=seed\nslot:unset|alias=after\nouter:unset|leaf=nested\nouter:unset|leaf=changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_root_variables_detaches_covered_reference_aliases() {
    let source = r#"<?php
class RefcowUnsetRootBag {
    public $items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
}

$items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
$alias =& $items["slot"];
$leaf =& $items["outer"]["leaf"];

unset($items);
echo isset($items) ? "array:set" : "array:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";
$alias = "after";
$leaf = "changed";
echo isset($items) ? "array:set" : "array:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";

$bag = new RefcowUnsetRootBag();
$propertyAlias =& $bag->items["slot"];
$propertyLeaf =& $bag->items["outer"]["leaf"];

unset($bag);
echo isset($bag) ? "object:set" : "object:unset";
echo "|alias=", $propertyAlias, "|leaf=", $propertyLeaf, "\n";
$propertyAlias = "property-after";
$propertyLeaf = "property-changed";
echo isset($bag) ? "object:set" : "object:unset";
echo "|alias=", $propertyAlias, "|leaf=", $propertyLeaf;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array:unset|alias=seed|leaf=nested\narray:unset|alias=after|leaf=changed\nobject:unset|alias=seed|leaf=nested\nobject:unset|alias=property-after|leaf=property-changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_object_properties_detaches_covered_reference_aliases() {
    let source = r#"<?php
class RefcowUnsetPropertyBag {
    public $items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
    private $privateItems = ["slot" => "private-seed", "outer" => ["leaf" => "private-nested"]];

    public function clearPrivate() {
        $alias =& $this->privateItems["slot"];
        $leaf =& $this->privateItems["outer"]["leaf"];

        unset($this->privateItems);
        echo isset($this->privateItems) ? "private:set" : "private:unset";
        echo "|alias=", $alias, "|leaf=", $leaf, "\n";
        $alias = "private-after";
        $leaf = "private-changed";
        echo isset($this->privateItems) ? "private:set" : "private:unset";
        echo "|alias=", $alias, "|leaf=", $leaf;
    }
}

$bag = new RefcowUnsetPropertyBag();
$alias =& $bag->items["slot"];
$leaf =& $bag->items["outer"]["leaf"];

unset($bag->items);
echo isset($bag->items) ? "public:set" : "public:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";
$alias = "after";
$leaf = "changed";
echo isset($bag->items) ? "public:set" : "public:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";

$bag->clearPrivate();
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "public:unset|alias=seed|leaf=nested\npublic:unset|alias=after|leaf=changed\nprivate:unset|alias=private-seed|leaf=private-nested\nprivate:unset|alias=private-after|leaf=private-changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_array_offset_rejects_non_array_targets() {
    let error = runtime_error("<?php\n$value = 1;\nunset($value[0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot unset offset on int"
    );
}

#[test]
fn unset_nested_array_offset_rejects_non_array_intermediate_values() {
    let error =
        runtime_error("<?php\n$items = ['outer' => 1];\nunset($items['outer']['inner']);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot unset offset on int"
    );
}
