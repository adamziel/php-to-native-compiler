<?php
error_reporting(0);

class Milestone2058_Bag implements ArrayAccess {
    public $items;

    public function __construct($value) {
        $this->items = array("slot" => $value);
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

class Milestone2058_Holder {
    public $bag;
}

$holder = new Milestone2058_Holder();
$holder->bag = new Milestone2058_Bag(3);
$alias =& $holder->bag["slot"]["leaf"];
$alias = "x";

