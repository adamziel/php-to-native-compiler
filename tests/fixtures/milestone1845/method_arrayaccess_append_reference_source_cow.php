<?php
class Milestone1845_Bag implements ArrayAccess {
    public $items = array();
    public $hits = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset === null ? "NULL" : $offset;
        $bucket =& $this->items[$offset];
        return $bucket;
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

class Milestone1845_Factory {
    public function make() {
        global $milestone1845_bag;
        return $milestone1845_bag;
    }
}

$milestone1845_bag = new Milestone1845_Bag();
$factory = new Milestone1845_Factory();
$alias =& $factory->make()[];
$alias = "changed";

echo $milestone1845_bag->items[""], "|", $milestone1845_bag->hits[0];
