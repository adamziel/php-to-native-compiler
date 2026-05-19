<?php
error_reporting(0);

class Milestone1974_Bag implements ArrayAccess {
    public $items = array("x" => false);

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone1974_Box {
    public $bag;

    public function __construct() {
        $this->bag = new Milestone1974_Bag();
    }

    public function __get($name) {
        return $this->bag;
    }
}

$box = new Milestone1974_Box();
$alias =& $box->missing["x"][];
$alias = "magic";

echo "type=", gettype($box->bag->items["x"]), "|value=", $box->bag->items["x"][0];
