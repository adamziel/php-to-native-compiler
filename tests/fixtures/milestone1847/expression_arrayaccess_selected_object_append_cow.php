<?php
class Milestone1847_Inner implements ArrayAccess {
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

class Milestone1847_Outer implements ArrayAccess {
    public $items = array();
    public $hits = array();

    public function __construct() {
        $this->items["box"] = new Milestone1847_Inner();
    }

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

function milestone1847_make_outer() {
    global $milestone1847_outer;
    return $milestone1847_outer;
}

$milestone1847_outer = new Milestone1847_Outer();
$alias =& milestone1847_make_outer()["box"][];
$alias = "changed";
$inner = $milestone1847_outer->items["box"];

echo $inner->items[""], "|", $milestone1847_outer->hits[0], "|", $inner->hits[0];
