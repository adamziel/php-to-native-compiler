<?php
class Milestone1844_Bag implements ArrayAccess {
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

function milestone1844_make_bag() {
    global $milestone1844_bag;
    return $milestone1844_bag;
}

$milestone1844_bag = new Milestone1844_Bag();
$alias =& milestone1844_make_bag()[];
$alias = "changed";

echo $milestone1844_bag->items[""], "|", $milestone1844_bag->hits[0];
