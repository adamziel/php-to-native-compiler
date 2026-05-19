<?php
error_reporting(0);

function milestone1989_set_ref(&$value) {
    $value = "dynamic";
}

class Milestone1989_Bag implements ArrayAccess {
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

class Milestone1989_Box {
    public $bag;

    public function __construct() {
        $this->bag = new Milestone1989_Bag();
    }

    public function __get($name) {
        return $this->bag;
    }
}

$box = new Milestone1989_Box();
$name = "missing";
milestone1989_set_ref($box->{$name}["x"][]);

echo gettype($box->bag->items["x"]), "|", $box->bag->items["x"][0];
