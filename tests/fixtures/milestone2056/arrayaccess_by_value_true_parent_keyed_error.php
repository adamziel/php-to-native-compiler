<?php
error_reporting(0);

class Milestone2056_Bag implements ArrayAccess {
    public $items;

    public function __construct() {
        $this->items = array("slot" => true);
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
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
        unset($this->items[$offset]);
    }
}

$bag = new Milestone2056_Bag();
$bag["slot"]["leaf"] = "x";

