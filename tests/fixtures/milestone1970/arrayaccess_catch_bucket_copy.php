<?php
class Milestone1970_Picked extends Exception {}

class Milestone1970_Bag implements ArrayAccess {
    public $items;

    public function __construct() {
        $this->items = array("x" => array("leaf" => "seed"));
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        try {
            throw new Milestone1970_Picked();
        } catch (Milestone1970_Picked $e) {
            return $this->items[$offset];
        }
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone1970_Bag();
$alias =& $bag->items["x"]["leaf"];
$copy = $bag["x"];
$copy["leaf"] = "arrayaccess";

echo "leaf=", $alias, "|backing=", $bag->items["x"]["leaf"];
