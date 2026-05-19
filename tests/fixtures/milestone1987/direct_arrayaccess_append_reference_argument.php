<?php
error_reporting(0);

function milestone1987_set_ref(&$value) {
    $value = "arrayaccess";
}

class Milestone1987_Bag implements ArrayAccess {
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

$bag = new Milestone1987_Bag();
milestone1987_set_ref($bag["x"][]);

echo gettype($bag->items["x"]), "|", $bag->items["x"][0];
