<?php
error_reporting(0);

class Milestone1835Bag implements ArrayAccess {
    public $items = array("x" => "seed");
    public $hits = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
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

$bag = new Milestone1835Bag();

function milestone1835_make_bag() {
    global $bag;
    return $bag;
}

$alias =& milestone1835_make_bag()["x"];
$alias = "changed";

echo $bag->items["x"], "|", $alias, "|hits=", implode(",", $bag->hits);
