<?php
error_reporting(0);

function milestone1990_set_ref(&$value) {
    $value = "holder";
}

class Milestone1990_Bag implements ArrayAccess {
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

class Milestone1990_Box {
    public $bag;

    public function __construct() {
        $this->bag = new Milestone1990_Bag();
    }

    public function __get($name) {
        return $this->bag;
    }
}

$holders = array("box" => new Milestone1990_Box());
milestone1990_set_ref($holders["box"]->missing["x"][]);

echo gettype($holders["box"]->bag->items["x"]), "|",
    $holders["box"]->bag->items["x"][0];
