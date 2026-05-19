<?php
error_reporting(0);

class Milestone1837Bag implements ArrayAccess {
    public $items = array("x" => "seed");

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
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

class Milestone1837Factory {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function make() {
        return $this->bag;
    }
}

$bag = new Milestone1837Bag();
$factory = new Milestone1837Factory($bag);

$alias =& $factory->make()["x"];
$alias = "changed";

echo $bag->items["x"], "|", $alias;
