<?php
function milestone1678_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1678_notice_handler", E_NOTICE);

class Milestone1678_Bag implements ArrayAccess {
    public $items = ["name" => "seed", "outer" => ["slot" => "nested"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return false; }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { }
}

class Milestone1678_Holder {
    public $bag;
    public $dynamicBag;

    public function __construct($bag) {
        $this->bag = $bag;
        $this->dynamicBag = $bag;
    }
}

class Milestone1678_Registry {
    public $holder;

    public function holder() {
        return $this->holder;
    }
}

function milestone1678_make_holder($bag) {
    return new Milestone1678_Holder($bag);
}

$bag = new Milestone1678_Bag();
$holders = ["box" => new Milestone1678_Holder($bag)];
$key = "name";
$alias =& $holders["box"]->bag[$key];
$alias = "changed";
echo $alias, "|", $bag->items[$key], "\n";

$property = "dynamicBag";
$dynamic =& $holders["box"]->{$property}["outer"]["slot"];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $bag->items["outer"]["slot"], "\n";

$registry = new Milestone1678_Registry();
$registry->holder = new Milestone1678_Holder($bag);
$method =& $registry->holder()->bag["outer"]["slot"];
$method = "method-changed";
echo $method, "|", $bag->items["outer"]["slot"], "\n";

$expr =& milestone1678_make_holder($bag)->bag["outer"]["slot"];
$expr = "expr-changed";
echo $expr, "|", $bag->items["outer"]["slot"];
