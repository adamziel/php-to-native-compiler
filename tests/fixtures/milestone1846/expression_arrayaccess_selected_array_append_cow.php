<?php
class Milestone1846_Bag implements ArrayAccess {
    public $items = array("outer" => array());
    public $hits = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
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

function milestone1846_make_bag() {
    global $milestone1846_bag;
    return $milestone1846_bag;
}

$milestone1846_bag = new Milestone1846_Bag();
$alias =& milestone1846_make_bag()["outer"][];
$alias = "changed";

echo $milestone1846_bag->items["outer"][0], "|", $milestone1846_bag->hits[0];
