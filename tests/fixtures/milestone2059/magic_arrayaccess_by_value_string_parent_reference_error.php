<?php
error_reporting(0);

class Milestone2059_Bag implements ArrayAccess {
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

class Milestone2059_Box {
    public $bag;

    public function __construct($value) {
        $this->bag = new Milestone2059_Bag($value);
    }

    public function __get($name) {
        return $this->bag;
    }
}

$box = new Milestone2059_Box("abc");
$alias =& $box->missing["slot"]["leaf"];
$alias = "x";

