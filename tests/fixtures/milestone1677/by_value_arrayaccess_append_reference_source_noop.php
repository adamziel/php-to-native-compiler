<?php
function milestone1677_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1677_notice_handler", E_NOTICE);

class Milestone1677_Bag implements ArrayAccess {
    public $items = ["" => "empty"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return false; }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { }
}

class Milestone1677_Holder {
    public $bag;
    public $dynamicBag;
}

$bag = new Milestone1677_Bag();
$alias =& $bag[];
$alias = "changed";
echo $alias, "|", $bag->items[""], "\n";

$holder = new Milestone1677_Holder();
$holder->bag = new Milestone1677_Bag();
$held =& $holder->bag[];
$held = "held-changed";
echo $held, "|", $holder->bag->items[""], "\n";

$holder->dynamicBag = new Milestone1677_Bag();
$property = "dynamicBag";
$dynamic =& $holder->{$property}[];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $holder->dynamicBag->items[""], "\n";

$items = ["slot" => "old"];
$target =& $items["slot"];
$detached = new Milestone1677_Bag();
$target =& $detached[];
$target = "detached-changed";
echo $target, "|", $items["slot"], "|", $detached->items[""];
