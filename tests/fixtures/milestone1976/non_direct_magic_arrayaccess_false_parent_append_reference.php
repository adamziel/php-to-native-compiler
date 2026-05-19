<?php
error_reporting(0);

class Milestone1976_Bag implements ArrayAccess {
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

class Milestone1976_Box {
    public $bag;

    public function __construct() {
        $this->bag = new Milestone1976_Bag();
    }

    public function __get($name) {
        return $this->bag;
    }
}

$holders = array("box" => new Milestone1976_Box());
$alias =& $holders["box"]->missing["x"][];
$alias = "holder";

echo "type=", gettype($holders["box"]->bag->items["x"]),
    "|value=", $holders["box"]->bag->items["x"][0];
