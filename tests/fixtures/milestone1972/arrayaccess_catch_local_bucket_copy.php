<?php
class Milestone1972_Picked extends Exception {}

class Milestone1972_Bag implements ArrayAccess {
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
            throw new Milestone1972_Picked();
        } catch (Milestone1972_Picked $e) {
            $bucket = $this->items[$offset];
            return $bucket;
        }
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone1972_Bag();
$alias =& $bag->items["x"]["leaf"];
$copy = $bag["x"];
$copy["leaf"] = "local";

echo "leaf=", $alias, "|backing=", $bag->items["x"]["leaf"];
