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
        "invalid foreach: can only iterate arrays in the current subset, got int"
    );
}

#[test]
fn foreach_key_value_rejects_object_iteration() {
    let error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
foreach ($box as $key => $value) {
    echo $key, $value;
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays in the current subset, got object"
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
        "invalid foreach: can only iterate arrays in the current subset, got int"
    );
}
