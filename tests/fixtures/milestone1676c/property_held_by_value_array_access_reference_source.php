<?php
function milestone1676c_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1676c_notice_handler", E_NOTICE);

class Milestone1676c_Bag implements ArrayAccess {
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

class Milestone1676c_Holder {
    public $bag;
    public $nested;
}

$holder = new Milestone1676c_Holder();
$holder->bag = new Milestone1676c_Bag(["name" => "seed"]);
$key = "name";
$alias =& $holder->bag[$key];
$alias = "changed";
echo $alias, "|", $holder->bag->items[$key], "\n";

$holder->nested = new Milestone1676c_Bag(["outer" => ["slot" => "inner-seed"]]);
$nested =& $holder->nested["outer"]["slot"];
$nested = "inner-changed";
echo $nested, "|", $holder->nested->items["outer"]["slot"];
