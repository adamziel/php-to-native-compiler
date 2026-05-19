<?php
error_reporting(0);

class Milestone1973_Bag implements ArrayAccess {
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

$bag = new Milestone1973_Bag();
$alias =& $bag["x"][];
$alias = "direct";

echo "type=", gettype($bag->items["x"]), "|value=", $bag->items["x"][0];
