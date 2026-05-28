use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn foreach_iterates_ordered_array_values() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

foreach ($items as $item) {
    echo $item, "|";
}
echo "\nlast:", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Ada|five|two updated|zero two|next|\nlast:next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_iterates_ordered_array_keys_and_values() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
echo "\nlast:", $key, "=", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "name:Ada|5:five|2:two updated|02:zero two|6:next|\nlast:6=next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_consumes_innermost_break_and_continue() {
    let source = r#"<?php
$items = [1, 2, 3, 4, 5];

foreach ($items as $item) {
    if ($item == 2) {
        continue;
    }
    if ($item == 4) {
        break;
    }
    echo $item, ",";
}
echo "after:", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "1,3,after:4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_syntax_inside_unexecuted_function_body_is_registered() {
    let execution = php_compiler::run_source(
        r#"<?php
function sort_recursive(&$items) {
    foreach ($items as &$item) {
        if (is_array($item)) {
            sort_recursive($item);
        }
    }
}
echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_copy_back_updates_direct_array_variable_elements() {
    let source = r#"<?php
$items = ["a", ["nested" => "b"]];

foreach ($items as $key => &$item) {
    if (is_array($item)) {
        $item["seen"] = $key;
    } else {
        $item = $item . "!";
    }
}
unset($item);

echo $items[0], "|", $items[1]["nested"], "|", $items[1]["seen"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_visits_appended_direct_array_elements() {
    let source = r#"<?php
$items = [1, 2];

foreach ($items as $key => &$item) {
    echo $key, ":", $item, "|";
    if ($item === 1) {
        $items[] = 3;
    }
}
unset($item);
echo "\n";
foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "0:1|1:2|2:3|\n0:1|1:2|2:3|");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_loop_variable_tracks_current_direct_array_slot() {
    let source = r#"<?php
$items = ["a" => 1, "b" => 2];

foreach ($items as $key => &$item) {
    echo "before=", $item, "|";
    if ($key === "a") {
        $items["a"] = 10;
        echo "after-direct=", $item, "|";
    }
}
unset($item);
echo "\n";
foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "before=1|after-direct=10|before=2|\na:10|b:2|"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_unset_current_slot_detaches_loop_variable() {
    let source = r#"<?php
$items = ["a" => 1, "b" => 2];

foreach ($items as $key => &$item) {
    echo $key, ":", $item, "|";
    if ($key === "a") {
        unset($items["a"]);
        $items["a"] = 10;
        echo "after=", $item, "|";
        $item = 11;
        echo "assigned=", $items["a"], "|";
    }
}
unset($item);
echo "\n";
foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "a:1|after=1|assigned=10|b:2|a:10|after=10|assigned=10|\nb:2|a:10|"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_lingers_as_last_direct_array_slot_after_loop() {
    let source = r#"<?php
$items = ["a", "b", "c"];

foreach ($items as $key => &$item) {
    $item = $item . $key;
}

$items[2] = "direct";
echo $item;
echo "|";
$item = "tail";
echo $items[0], "|", $items[1], "|", $items[2], "|", $key, "|", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "direct|a0|b1|tail|2|tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_lingering_reference_is_cleared_by_unset_or_empty_iteration() {
    let source = r#"<?php
$items = ["a", "b", "c"];

foreach ($items as &$item) {
    $item = $item . "!";
}

unset($item);
$item = "tail";
echo $items[0], "|", $items[1], "|", $items[2], "|", $item;
echo "\n";

$empty = [];
$value = "before";
foreach ($empty as &$value) {
    $value = "unreached";
}
$value = "after";
echo count($empty), "|", $value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b!|c!|tail\n0|after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_value_assignment_reuses_existing_lingering_reference_like_php() {
    let source = r#"<?php
$items = ["a", "b", "c"];

foreach ($items as &$item) {
    $item = $item . "!";
}

foreach (["x"] as $item) {
}

echo $items[0], "|", $items[1], "|", $items[2], "|", $item;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b!|x|x");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn print_r_reads_reference_backed_array_slots_after_foreach_mutation() {
    let source = r#"<?php
$items = ["alpha" => "A", "beta" => "B"];

foreach ($items as $key => $value) {
    $value = $key;
}

echo $items["alpha"], "|", $items["beta"], "\n";

foreach ($items as $key => &$value) {
    $value = $key;
}

print_r($items);
$value = "tail";
echo $items["beta"], "|", $value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "A|B\nArray\n(\n    [alpha] => alpha\n    [beta] => beta\n)\ntail|tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_accepts_temporary_array_literals() {
    let source = r#"<?php
foreach (["a", "b"] as &$value) {
    $value = $value . "!";
    echo $value, "|";
}

echo "after=", $value, "|";
$value = "changed";
echo $value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a!|b!|after=b!|changed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_accepts_temporary_function_return_arrays() {
    let source = r#"<?php
function items() {
    return ["x" => 1, "y" => 2];
}

foreach (items() as $key => &$value) {
    $value = $value + 10;
    echo $key, ":", $value, "|";
}

echo "after=", $value, "|";
$value = 99;
echo $value;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "x:11|y:12|after=12|99");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_object_property_value_target() {
    let source = r#"<?php
$obj = new stdClass();
$items = ["a" => 1, "b" => 2];

foreach ($items as $key => &$obj->slot) {
    echo $key, ":", $obj->slot, "|";
    if ($key === "a") {
        $obj->slot = 10;
    }
}

echo $items["a"], "|", $items["b"], "|";
$items["b"] = 20;
echo $obj->slot, "|";
$obj->slot = 30;
echo $items["b"], "|";

$property = "dynamic";
foreach (["x"] as &$obj->{$property}) {
    $obj->{$property} = $obj->{$property} . "!";
}
echo $obj->dynamic;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "a:1|b:2|10|2|20|30|x!");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_string_keyed_globals_array_root() {
    let source = r#"<?php
$GLOBALS["bag"] = ["one" => "a", "two" => "b"];

foreach ($GLOBALS["bag"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "one") {
        $GLOBALS["bag"]["three"] = "c";
    }
}

echo $GLOBALS["bag"]["one"], "|", $GLOBALS["bag"]["two"], "|", $GLOBALS["bag"]["three"], "|", $value, "|", $key, "\n";
$GLOBALS["bag"]["three"] = "direct";
echo $value, "|";
$value = "tail";
echo $GLOBALS["bag"]["three"], "|", $value, "\n";
unset($value);

function mutate_global_bag() {
    foreach ($GLOBALS["bag"] as $key => &$value) {
        if ($key === "one") {
            $value = "fn";
        }
    }
    unset($value);
}

mutate_global_bag();
echo $GLOBALS["bag"]["one"], "|", $GLOBALS["bag"]["two"], "|", $GLOBALS["bag"]["three"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "a:one|b:two|c:three|c:three|three\ndirect|tail|tail\nfn|b:two|tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_nested_array_and_global_request_paths() {
    let source = r#"<?php
$items = ["outer" => ["a" => "one", "b" => "two"]];

foreach ($items["outer"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $items["outer"]["c"] = "three";
    }
}

echo $items["outer"]["a"], "|", $items["outer"]["b"], "|", $items["outer"]["c"], "|", $value, "\n";
$value = "tail";
echo $items["outer"]["c"], "|", $value, "\n";
unset($value);

$GLOBALS["bag"] = ["child" => ["x" => "ex", "y" => "why"]];
foreach ($GLOBALS["bag"]["child"] as $key => &$value) {
    $value = $key . "=" . $value;
}
echo $GLOBALS["bag"]["child"]["x"], "|", $GLOBALS["bag"]["child"]["y"], "|", $value, "\n";
unset($value);

$_REQUEST["payload"] = ["first" => "alpha", "second" => "beta"];
function mutate_request_payload() {
    foreach ($_REQUEST["payload"] as $key => &$value) {
        if ($key === "second") {
            $value = "changed";
        }
    }
    unset($value);
}
mutate_request_payload();
echo $_REQUEST["payload"]["first"], "|", $_REQUEST["payload"]["second"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "one:a|two:b|three:c|three:c\ntail|tail\nx=ex|y=why|y=why\nalpha|changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_object_property_array_roots() {
    let source = r#"<?php
class Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
}

$bag = new Bag();
foreach ($bag->items["outer"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "three";
    }
}

echo $bag->items["outer"]["a"], "|", $bag->items["outer"]["b"], "|", $bag->items["outer"]["c"], "|", $value, "\n";
$bag->items["outer"]["c"] = "direct";
echo $value, "|";
$value = "tail";
echo $bag->items["outer"]["c"], "|", $value, "\n";
unset($value);

foreach ($bag->items as $key => &$value) {
    if ($key === "outer") {
        $value["d"] = "delta";
    }
}
unset($value);
echo $bag->items["outer"]["d"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "one:a|two:b|three:c|three:c\ndirect|tail|tail\ndelta"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_dynamic_object_property_array_roots() {
    let source = r#"<?php
class Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
    private $privateItems = ["x" => "ex", "y" => "why"];

    public function mutatePrivate($property) {
        foreach ($this->{$property} as $key => &$value) {
            $value = "private:" . $key;
            if ($key === "x") {
                $this->privateItems["z"] = "zed";
            }
        }
        echo $this->{$property}["x"], "|", $this->{$property}["y"], "|", $this->{$property}["z"], "|", $value, "\n";
        $this->privateItems["z"] = "direct-private";
        echo $value, "|";
        $value = "tail-private";
        echo $this->{$property}["z"], "|", $value;
    }
}

$bag = new Bag();
$property = "items";
foreach ($bag->{$property}["outer"] as $key => &$value) {
    $value = "public:" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "three";
    }
}
echo $bag->{$property}["outer"]["a"], "|", $bag->{$property}["outer"]["b"], "|", $bag->{$property}["outer"]["c"], "|", $value, "\n";
$bag->items["outer"]["c"] = "direct-public";
echo $value, "|";
$value = "tail-public";
echo $bag->{$property}["outer"]["c"], "|", $value, "\n";
unset($value);

$bag->mutatePrivate("privateItems");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "public:a|public:b|public:c|public:c\ndirect-public|tail-public|tail-public\nprivate:x|private:y|private:z|private:z\ndirect-private|tail-private|tail-private"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_non_direct_dynamic_property_holder_roots() {
    let source = r#"<?php
class Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
}

class PrivateBag {
    private $privateItems = ["x" => "ex", "y" => "why"];

    private function holder() {
        return $this;
    }

    public function mutate($property) {
        foreach ($this->holder()->{$property} as $key => &$value) {
            $value = "private:" . $key;
            if ($key === "x") {
                $this->privateItems["z"] = "zed";
            }
        }
        echo $this->{$property}["x"], "|", $this->{$property}["y"], "|", $this->{$property}["z"], "|", $value, "\n";
        $this->privateItems["z"] = "direct-private";
        echo $value, "|";
        $value = "tail-private";
        echo $this->{$property}["z"], "|", $value;
    }
}

$holders = ["bag" => new Bag()];
$property = "items";
foreach ($holders["bag"]->{$property}["outer"] as $key => &$value) {
    $value = "public:" . $key;
}
echo $holders["bag"]->{$property}["outer"]["a"], "|", $holders["bag"]->{$property}["outer"]["b"], "|", $value, "\n";
$bag = $holders["bag"];
$bag->items["outer"]["b"] = "direct-public";
echo $value, "|";
$value = "tail-public";
echo $holders["bag"]->{$property}["outer"]["b"], "|", $value, "\n";
unset($value);

$private = new PrivateBag();
$private->mutate("privateItems");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "public:a|public:b|public:b\ndirect-public|tail-public|tail-public\nprivate:x|private:y|private:z|private:z\ndirect-private|tail-private|tail-private"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_non_direct_named_property_holder_roots() {
    let source = r#"<?php
class Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
}

class PrivateBag {
    private $items = ["x" => "ex", "y" => "why"];

    private function holder() {
        return $this;
    }

    public function mutate() {
        foreach ($this->holder()->items as $key => &$value) {
            $value = "private:" . $key;
            if ($key === "x") {
                $this->items["z"] = "zed";
            }
        }
        echo $this->items["x"], "|", $this->items["y"], "|", $this->items["z"], "|", $value, "\n";
        $this->items["z"] = "direct-private";
        echo $value, "|";
        $value = "tail-private";
        echo $this->items["z"], "|", $value;
    }
}

$holders = ["bag" => new Bag()];
foreach ($holders["bag"]->items["outer"] as $key => &$value) {
    $value = "public:" . $key;
}
echo $holders["bag"]->items["outer"]["a"], "|", $holders["bag"]->items["outer"]["b"], "|", $value, "\n";
$bag = $holders["bag"];
$bag->items["outer"]["b"] = "direct-public";
echo $value, "|";
$value = "tail-public";
echo $holders["bag"]->items["outer"]["b"], "|", $value, "\n";
unset($value);

$private = new PrivateBag();
$private->mutate();
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "public:a|public:b|public:b\ndirect-public|tail-public|tail-public\nprivate:x|private:y|private:z|private:z\ndirect-private|tail-private|tail-private"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_reference_return_iterable_to_caller_cell() {
    let execution = run_source(
        r#"<?php
function &items(&$items) {
    return $items;
}

$items = ["a" => "one", "b" => "two"];
foreach (items($items) as $key => &$item) {
    $item = $item . ":" . $key;
    if ($key === "a") {
        $items["c"] = "three";
    }
}
echo $items["a"], "|", $items["b"], "|", $items["c"], "|", $item, "\n";
$items["c"] = "direct";
echo $item, "|";
$item = "tail";
echo $items["c"], "|", $item, "\n";
unset($item);
$items["c"] = "detached";
echo $items["c"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "one:a|two:b|three:c|three:c\ndirect|tail|tail\ndetached"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_multi_slot_reference_return_iterable_alias() {
    let execution = run_source(
        r#"<?php
function &pick_child(&$items, $key) {
    return $items[$key];
}

$items = ["outer" => ["a" => "one", "b" => "two"]];
$mirror =& $items;

foreach (pick_child($mirror, "outer") as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $items["outer"]["c"] = "three";
    }
}

echo $items["outer"]["a"], "|", $mirror["outer"]["b"], "|", $value, "\n";
$items["outer"]["c"] = "direct";
echo $value, "|";
$value = "tail";
echo $mirror["outer"]["c"], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "one:a|two:b|three:c\ndirect|tail|tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_over_copied_nested_array_preserves_reference_slots() {
    let execution = run_source(
        r#"<?php
$items = array("outer" => array("plain" => "p", "slot" => "orig"));
$alias =& $items["outer"]["slot"];
$copy = $items["outer"];

foreach ($copy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $items["outer"]["slot"], "|", $copy["slot"], "|", $items["outer"]["plain"], "|", $copy["plain"], "|";
$copy["slot"] = "direct";
echo $value, "|";
$value = "tail";
echo $items["outer"]["slot"], "|", $copy["slot"], "\n";
unset($value);

$_REQUEST["payload"] = array("plain" => "r", "slot" => "request");
$requestAlias =& $_REQUEST["payload"]["slot"];
$requestCopy = $_REQUEST["payload"];
foreach ($requestCopy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $_REQUEST["payload"]["slot"], "|", $requestCopy["slot"], "|", $_REQUEST["payload"]["plain"], "|", $requestCopy["plain"], "|";
$requestCopy["slot"] = "request-direct";
echo $value, "|";
$value = "request-tail";
echo $_REQUEST["payload"]["slot"], "|", $requestCopy["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "orig:slot|orig:slot|p|p:plain|direct|tail|tail\nrequest:slot|request:slot|r|r:plain|request-direct|request-tail|request-tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_over_copied_dynamic_object_property_array_preserves_reference_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items;
}

$box = new Box();
$name = "items";
$box->items = array("outer" => array("plain" => "p", "slot" => "orig"));
$alias =& $box->{$name}["outer"]["slot"];
$copy = $box->{$name}["outer"];

foreach ($copy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $box->items["outer"]["slot"], "|", $copy["slot"], "|", $box->items["outer"]["plain"], "|", $copy["plain"], "|";
$copy["slot"] = "direct";
echo $value, "|";
$value = "tail";
echo $box->items["outer"]["slot"], "|", $copy["slot"], "\n";
unset($value);

$box->items = array("plain" => "root", "slot" => "whole");
$rootAlias =& $box->{$name}["slot"];
$rootCopy = $box->{$name};
foreach ($rootCopy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $box->items["slot"], "|", $rootCopy["slot"], "|", $box->items["plain"], "|", $rootCopy["plain"], "|";
$rootCopy["slot"] = "root-direct";
echo $value, "|";
$value = "root-tail";
echo $box->items["slot"], "|", $rootCopy["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "orig:slot|orig:slot|p|p:plain|direct|tail|tail\nwhole:slot|whole:slot|root|root:plain|root-direct|root-tail|root-tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_mutates_public_object_properties_and_lingers() {
    let execution = run_source(
        r#"<?php
class Box {
    public $first = "one";
    public $second = "two";
}

$box = new Box();
foreach ($box as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $box->first, "|", $box->second, "|", $value, "|", $key, "\n";
$box->second = "direct";
echo $value, "|";
$value = "tail";
echo $box->second, "|", $value, "\n";
unset($value);

$std = new stdClass();
$alpha = "alpha";
$beta = "beta";
$gamma = "gamma";
$std->{$alpha} = "a";
$std->{$beta} = "b";
foreach ($std as $key => &$value) {
    $value = $key . "=" . $value;
    if ($key === "alpha") {
        $std->{$gamma} = "g";
    }
}
echo $std->alpha, "|", $std->beta, "|", $std->gamma, "|", $value, "|", $key, "\n";
$std->{$gamma} = "direct-g";
echo $value, "|";
$value = "tail-g";
echo $std->gamma, "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "one:first|two:second|two:second|second\ndirect|tail|tail\nalpha=a|beta=b|gamma=g|gamma=g|gamma\ndirect-g|tail-g|tail-g"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_method_reference_return_iterables_to_caller_cell() {
    let execution = run_source(
        r#"<?php
class Bag {
    public function &items(&$items) {
        return $items;
    }
}

class StaticBag {
    public static function &items(&$items) {
        return $items;
    }
}

$bag = new Bag();
$items = ["a" => "one", "b" => "two"];
foreach ($bag->items($items) as $key => &$item) {
    $item = $item . ":" . $key;
    if ($key === "a") {
        $items["c"] = "three";
    }
}
echo $items["a"], "|", $items["b"], "|", $items["c"], "|", $item, "\n";
$items["c"] = "direct";
echo $item, "|";
$item = "tail";
echo $items["c"], "|", $item, "\n";
unset($item);

$items = ["x" => "ex", "y" => "why"];
foreach (StaticBag::items($items) as $key => &$item) {
    $item = $key . "=" . $item;
    if ($key === "x") {
        $items["z"] = "zed";
    }
}
echo $items["x"], "|", $items["y"], "|", $items["z"], "|", $item, "\n";
$items["z"] = "static-direct";
echo $item, "|";
$item = "static-tail";
echo $items["z"], "|", $item;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "one:a|two:b|three:c|three:c\ndirect|tail|tail\nx=ex|y=why|z=zed|z=zed\nstatic-direct|static-tail|static-tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_extended_reference_return_iterables_to_caller_cell() {
    let execution = run_source(
        r#"<?php
function &items_callback(&$items) {
    return $items;
}

class BaseBag {
    public static function &items(&$items) {
        return $items;
    }
}

class ChildBag extends BaseBag {
    public static function &items(&$items) {
        return $items;
    }

    public function runSelf(&$items) {
        foreach (self::items($items) as $key => &$item) {
            $item = "self:" . $key;
            if ($key === "a") {
                $items["b"] = "bee";
            }
        }
        echo $items["a"], "|", $items["b"], "|", $item, "\n";
        unset($item);
    }

    public function runParent(&$items) {
        foreach (parent::items($items) as $key => &$item) {
            $item = "parent:" . $key;
            if ($key === "a") {
                $items["b"] = "bee";
            }
        }
        echo $items["a"], "|", $items["b"], "|", $item, "\n";
        unset($item);
    }

    public function runStatic(&$items) {
        foreach (static::items($items) as $key => &$item) {
            $item = "static:" . $key;
            if ($key === "a") {
                $items["b"] = "bee";
            }
        }
        echo $items["a"], "|", $items["b"], "|", $item, "\n";
        unset($item);
    }
}

$bag = new ChildBag();

$items = ["a" => "aye"];
$bag->runSelf($items);
$items["b"] = "direct";
echo $items["b"], "\n";

$items = ["a" => "aye"];
$bag->runParent($items);
$items["b"] = "direct";
echo $items["b"], "\n";

$items = ["a" => "aye"];
$bag->runStatic($items);
$items["b"] = "direct";
echo $items["b"], "\n";

$items = ["a" => "aye"];
$class = "ChildBag";
foreach ($class::items($items) as $key => &$item) {
    $item = "class:" . $key;
    if ($key === "a") {
        $items["b"] = "bee";
    }
}
echo $items["a"], "|", $items["b"], "|", $item, "\n";
unset($item);

$items = ["a" => "aye"];
foreach ($bag::items($items) as $key => &$item) {
    $item = "object:" . $key;
    if ($key === "a") {
        $items["b"] = "bee";
    }
}
echo $items["a"], "|", $items["b"], "|", $item, "\n";
unset($item);

$items = ["a" => "aye"];
foreach (call_user_func_array("items_callback", array(&$items)) as $key => &$item) {
    $item = "callback:" . $key;
    if ($key === "a") {
        $items["b"] = "bee";
    }
}
echo $items["a"], "|", $items["b"], "|", $item;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "self:a|self:b|self:b\ndirect\nparent:a|parent:b|parent:b\ndirect\nstatic:a|static:b|static:b\ndirect\nclass:a|class:b|class:b\nobject:a|object:b|object:b\ncallback:a|callback:b|callback:b"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class RefCowForeachArrayAccessBag implements ArrayAccess {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
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

class RefCowForeachArrayAccessHolder {
    public $bag;
}

$bag = new RefCowForeachArrayAccessBag();
foreach ($bag["outer"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "three";
    }
}
echo $bag["outer"]["a"], "|", $bag["outer"]["b"], "|", $value, "\n";
$bag->items["outer"]["c"] = "direct";
echo $value, "|";
$value = "tail";
echo $bag["outer"]["c"], "|", $value, "\n";
unset($value);

$namedHolder = new RefCowForeachArrayAccessHolder();
$namedHolder->bag = new RefCowForeachArrayAccessBag();
$namedBag = $namedHolder->bag;
foreach ($namedHolder->bag["outer"] as $key => &$value) {
    $value = "named:" . $key;
    if ($key === "a") {
        $namedBag->items["outer"]["c"] = "named-c";
    }
}
echo $namedHolder->bag["outer"]["a"], "|", $namedHolder->bag["outer"]["b"], "|", $value, "\n";
$namedBag->items["outer"]["c"] = "named-direct";
echo $value, "|";
$value = "named-tail";
echo $namedHolder->bag["outer"]["c"], "|", $value, "\n";
unset($value);

$holder = new RefCowForeachArrayAccessHolder();
$holder->bag = new RefCowForeachArrayAccessBag();
$heldBag = $holder->bag;
$property = "bag";
foreach ($holder->{$property}["outer"] as $key => &$value) {
    $value = "held:" . $key;
    if ($key === "a") {
        $heldBag->items["outer"]["c"] = "held-c";
    }
}
echo $holder->bag["outer"]["a"], "|", $holder->bag["outer"]["b"], "|", $value, "\n";
$heldBag->items["outer"]["c"] = "held-direct";
echo $value, "|";
$value = "held-tail";
echo $holder->bag["outer"]["c"], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "one:a|two:b|three:c\ndirect|tail|tail\nnamed:a|named:b|named:c\nnamed-direct|named-tail|named-tail\nheld:a|held:b|held:c\nheld-direct|held-tail|held-tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_over_array_access_bucket_copies_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class RefCowForeachArrayAccessBucketCopy implements ArrayAccess {
    public $callbacks = [];

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = [
            "id" => ["function" => &$callback, "accepted_args" => 1],
            "plain" => ["function" => "plain", "accepted_args" => 1],
        ];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->callbacks[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->callbacks[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->callbacks[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->callbacks[$offset]);
    }
}

class RefCowForeachArrayAccessBucketAlias implements ArrayAccess {
    public $callbacks = [];

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = [
            "id" => ["function" => &$callback, "accepted_args" => 1],
            "plain" => ["function" => "plain", "accepted_args" => 1],
        ];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->callbacks[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->callbacks[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->callbacks[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->callbacks[$offset]);
    }
}

function exercise_array_access_bucket_copy($hook, $label) {
    $bucket = $hook[10];
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":copy";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = $label . ":plain-copy";
        }
    }
    unset($node);
}

$callback = "seed";
$valueHook = new RefCowForeachArrayAccessBucketCopy();
$valueHook->add(10, $callback);
exercise_array_access_bucket_copy($valueHook, "value");
echo $callback, "|", $valueHook->callbacks[10]["id"]["function"], "|", $valueHook->callbacks[10]["id"]["accepted_args"], "|", $valueHook->callbacks[10]["plain"]["function"], "\n";

$refCallback = "seed";
$refHook = new RefCowForeachArrayAccessBucketAlias();
$refHook->add(10, $refCallback);
exercise_array_access_bucket_copy($refHook, "ref");
echo $refCallback, "|", $refHook->callbacks[10]["id"]["function"], "|", $refHook->callbacks[10]["id"]["accepted_args"], "|", $refHook->callbacks[10]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "value:copy|value:copy|1|plain\nref:copy|ref:copy|1|plain"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_over_array_access_bucket_helper_parameter_preserves_nested_reference_slots()
{
    let execution = run_source(
        r#"<?php
class RefCowForeachArrayAccessBucketHelperHook implements ArrayAccess {
    public $callbacks = [];

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = [
            "id" => ["function" => &$callback, "accepted_args" => 1],
            "plain" => ["function" => "plain", "accepted_args" => 1],
        ];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->callbacks[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->callbacks[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->callbacks[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->callbacks[$offset]);
    }
}

class RefCowForeachArrayAccessBucketHelperHolder {
    public $hook;
}

function make_array_access_bucket_helper_holder($hook) {
    $holder = new RefCowForeachArrayAccessBucketHelperHolder();
    $holder->hook = $hook;
    return $holder;
}

function mutate_array_access_bucket_helper_copy($bucket, $label) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":helper";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = $label . ":plain-helper";
        }
    }
    unset($node);
}

$callback = "seed";
$holder = new RefCowForeachArrayAccessBucketHelperHolder();
$holder->hook = new RefCowForeachArrayAccessBucketHelperHook();
$holder->hook->add(10, $callback);
$bucket = $holder->hook[10];
mutate_array_access_bucket_helper_copy($bucket, "property");
echo $callback, "|", $holder->hook->callbacks[10]["id"]["function"], "|", $holder->hook->callbacks[10]["id"]["accepted_args"], "|", $holder->hook->callbacks[10]["plain"]["function"], "\n";

$exprCallback = "seed";
$exprHook = new RefCowForeachArrayAccessBucketHelperHook();
$exprHook->add(10, $exprCallback);
$exprBucket = make_array_access_bucket_helper_holder($exprHook)->hook[10];
mutate_array_access_bucket_helper_copy($exprBucket, "expr");
echo $exprCallback, "|", $exprHook->callbacks[10]["id"]["function"], "|", $exprHook->callbacks[10]["id"]["accepted_args"], "|", $exprHook->callbacks[10]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "property:helper|property:helper|1|plain\nexpr:helper|expr:helper|1|plain"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_binds_non_direct_holder_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class RefCowForeachNonDirectArrayAccessBag implements ArrayAccess {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
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

class RefCowForeachNonDirectArrayAccessHolder {
    public $bag;
    public $dynamicBag;
}

$holders = [];
$primary = new RefCowForeachNonDirectArrayAccessHolder();
$primary->bag = new RefCowForeachNonDirectArrayAccessBag();
$holders["primary"] = $primary;
$bag = $holders["primary"]->bag;
foreach ($holders["primary"]->bag["outer"] as $key => &$value) {
    $value = "array:" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "array-c";
    }
}
echo $holders["primary"]->bag["outer"]["a"], "|", $holders["primary"]->bag["outer"]["b"], "|", $value, "\n";
$bag->items["outer"]["c"] = "array-direct";
echo $value, "|";
$value = "array-tail";
echo $holders["primary"]->bag["outer"]["c"], "|", $value, "\n";
unset($value);

$dynamic = new RefCowForeachNonDirectArrayAccessHolder();
$dynamic->dynamicBag = new RefCowForeachNonDirectArrayAccessBag();
$holders["dynamic"] = $dynamic;
$dynamicBag = $holders["dynamic"]->dynamicBag;
$property = "dynamicBag";
foreach ($holders["dynamic"]->{$property}["outer"] as $key => &$value) {
    $value = "dynamic:" . $key;
    if ($key === "a") {
        $dynamicBag->items["outer"]["c"] = "dynamic-c";
    }
}
echo $holders["dynamic"]->dynamicBag["outer"]["a"], "|", $holders["dynamic"]->dynamicBag["outer"]["b"], "|", $value, "\n";
$dynamicBag->items["outer"]["c"] = "dynamic-direct";
echo $value, "|";
$value = "dynamic-tail";
echo $holders["dynamic"]->dynamicBag["outer"]["c"], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array:a|array:b|array:c\narray-direct|array-tail|array-tail\ndynamic:a|dynamic:b|dynamic:c\ndynamic-direct|dynamic-tail|dynamic-tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_value_iterates_public_object_properties_and_iterator_objects() {
    let execution = run_source(
        r#"<?php
class PublicBag {
    public $first = "one";
    public $second = "two";
    public $dynamic = "dyn";
    private $hidden = "secret";
}

$bag = new PublicBag();
foreach ($bag as $key => $value) {
    echo $key, "=", $value, ";";
    if ($key === "first") {
        $bag->second = "mutated";
    }
}
echo "|", $bag->second, "\n";

class HookIterator implements Iterator {
    public $items = array("first" => "alpha", "second" => "beta");
    public $keys = array("first", "second");
    public $pos = 0;

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->keys[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function current() {
        $key = $this->keys[$this->pos];
        return $this->items[$key];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

$iterator = new HookIterator();
foreach ($iterator as $key => $value) {
    echo $key, "=", $value, ";";
    if ($key === "first") {
        $iterator->items["second"] = "changed";
    }
}
echo "|", $iterator->pos;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "first=one;second=mutated;dynamic=dyn;|mutated\nfirst=alpha;second=changed;|2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_value_iterator_current_public_property_bucket_preserves_reference_slots() {
    let execution = run_source(
        r#"<?php
class HookBucketIterator implements Iterator {
    public $callbacks = array("10" => array("a" => "seed", "b" => "plain"));
    public $keys = array("10");
    public $pos = 0;

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->keys[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function current() {
        $priority = $this->keys[$this->pos];
        return $this->callbacks[$priority];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

$iterator = new HookBucketIterator();
$alias =& $iterator->callbacks["10"]["a"];
foreach ($iterator as $priority => $callbacks) {
    foreach ($callbacks as $id => &$callback) {
        $callback = $id . ":seen";
    }
    unset($callback);
    echo $priority, "=", $callbacks["a"], ",", $callbacks["b"], ";";
}
echo "|", $alias, "|", $iterator->callbacks["10"]["a"], "|", $iterator->callbacks["10"]["b"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "10=a:seen,b:seen;|a:seen|a:seen|plain\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_value_iterator_bucket_copy_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class HookBucketCow implements Iterator {
    public $callbacks = array();
    public $priorities = array();
    public $pos = 0;

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = array(
            "id" => array("function" => &$callback, "accepted_args" => 1),
            "plain" => array("function" => "plain", "accepted_args" => 1),
        );
        $this->priorities[] = $priority;
    }

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->priorities[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function current() {
        $priority = $this->priorities[$this->pos];
        return $this->callbacks[$priority];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->priorities[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

$callable = "seed";
$hook = new HookBucketCow();
$hook->add(10, $callable);

foreach ($hook as $priority => $bucket) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = "via-copy";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = "plain-copy";
        }
    }
    unset($node);
}

echo $callable, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["id"]["accepted_args"], "|", $hook->callbacks[10]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "via-copy|via-copy|1|plain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_by_reference_rejects_userland_iterator_like_php() {
    let error = runtime_error(
        r#"<?php
class RefIterator implements Iterator {
    public $items = array("first" => "alpha", "second" => "beta");
    public $keys = array("first", "second");
    public $pos = 0;

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->keys[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function &current() {
        $key = $this->keys[$this->pos];
        return $this->items[$key];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

$iterator = new RefIterator();
foreach ($iterator as $key => &$value) {
    $value = $value . ":" . $key;
}
"#,
    );

    assert_eq!(error.line, 35);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: An iterator cannot be used with foreach by reference"
    );
}

#[test]
fn foreach_by_reference_rejects_iterator_aggregate_userland_iterator_like_php() {
    let error = runtime_error(
        r#"<?php
class RefIteratorAggregateIterator implements Iterator {
    public $items = array("first" => "alpha", "second" => "beta");
    public $keys = array("first", "second");
    public $pos = 0;

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->keys[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function &current() {
        $key = $this->keys[$this->pos];
        return $this->items[$key];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

class RefIteratorAggregate implements IteratorAggregate {
    public $iterator;

    public function __construct() {
        $this->iterator = new RefIteratorAggregateIterator();
    }

    #[ReturnTypeWillChange]
    public function getIterator() {
        return $this->iterator;
    }
}

$aggregate = new RefIteratorAggregate();
foreach ($aggregate as $key => &$value) {
    $value = $value . ":" . $key;
}
"#,
    );

    assert_eq!(error.line, 48);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: An iterator cannot be used with foreach by reference"
    );
}

#[test]
fn foreach_by_reference_rejects_iterator_aggregate_non_traversable_return() {
    let error = runtime_error(
        r#"<?php
class BadAggregate implements IteratorAggregate {
    #[ReturnTypeWillChange]
    public function getIterator() {
        return "not-iterator";
    }
}

$aggregate = new BadAggregate();
foreach ($aggregate as $key => &$value) {
    echo $key, $value;
}
"#,
    );

    assert_eq!(error.line, 10);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: IteratorAggregate::getIterator() must return a Traversable object for by-reference foreach in PHP, got string"
    );
}

#[test]
fn foreach_key_value_requires_array_iterable() {
    let error = runtime_error(
        r#"<?php
foreach (42 as $key => $value) {
    echo $key, $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays, ordinary public-property objects, or bounded Iterator objects in the current subset, got int"
    );
}

#[test]
fn foreach_key_value_rejects_traversable_without_iterator_execution() {
    let error = runtime_error(
        r#"<?php
class BagAggregate implements IteratorAggregate {
    #[ReturnTypeWillChange]
    public function getIterator() {
        return "not-iterator";
    }
}
$box = new BagAggregate();
foreach ($box as $key => $value) {
    echo $key, $value;
}
"#,
    );

    assert_eq!(error.line, 9);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: IteratorAggregate::getIterator() must return an Iterator object in the current subset, got string"
    );
}

#[test]
fn foreach_requires_array_iterable() {
    let error = runtime_error(
        r#"<?php
foreach (42 as $value) {
    echo $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays, ordinary public-property objects, or bounded Iterator objects in the current subset, got int"
    );
}
