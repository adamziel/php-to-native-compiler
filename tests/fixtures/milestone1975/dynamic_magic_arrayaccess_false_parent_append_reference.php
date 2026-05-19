<?php
error_reporting(0);

class Milestone1975_Bag implements ArrayAccess {
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

class Milestone1975_Box {
    public $bag;

    public function __construct() {
        $this->bag = new Milestone1975_Bag();
    }

    public function __get($name) {
        return $this->bag;
    }
}

$name = "missing";
$box = new Milestone1975_Box();
$alias =& $box->{$name}["x"][];
$alias = "dynamic";

echo "type=", gettype($box->bag->items["x"]), "|value=", $box->bag->items["x"][0];
