<?php
class Milestone1848_Bag implements ArrayAccess {
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

class Milestone1848_Factory {
    public function make() {
        global $milestone1848_bag;
        return $milestone1848_bag;
    }
}

$milestone1848_bag = new Milestone1848_Bag();
$factory = new Milestone1848_Factory();
$alias =& $factory->make()["outer"][];
$alias = "changed";

echo $milestone1848_bag->items["outer"][0], "|", $milestone1848_bag->hits[0];
