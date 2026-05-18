<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
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

class Holder {
    public $bag;
    public $dynamicBag;
}

$holder = new Holder();
$holder->bag = new Bag();

$alias =& $holder->bag["name"];
$alias = "changed";
echo $alias, "|", $holder->bag->items["name"], "\n";

$nested =& $holder->bag["outer"]["slot"];
$nested = "changed-nested";
echo $nested, "|", $holder->bag->items["outer"]["slot"], "\n";

$holder->dynamicBag = new Bag();
$property = "dynamicBag";
$dynamic =& $holder->{$property}["outer"]["slot"];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $holder->dynamicBag->items["outer"]["slot"];
