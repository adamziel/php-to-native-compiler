<?php
function milestone1676b_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1676b_notice_handler", E_NOTICE);

class Milestone1676b_Bag implements ArrayAccess {
    public $items;

    public function __construct($items) {
        $this->items = $items;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return false;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }
}

class Milestone1676b_Holder {
    public $bag;
}

$holder = new Milestone1676b_Holder();
$holder->bag = new Milestone1676b_Bag([
    "name" => "seed",
    "outer" => ["slot" => "inner-seed"],
]);

$key = "name";
$alias =& $holder->bag[$key];
$alias = "changed";
echo $alias, "|", $holder->bag->items[$key], "\n";

$nested =& $holder->bag["outer"]["slot"];
$nested = "inner-changed";
echo $nested, "|", $holder->bag->items["outer"]["slot"];
