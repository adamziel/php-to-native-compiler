<?php
class Milestone1849_Inner implements ArrayAccess {
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

class Milestone1849_Outer implements ArrayAccess {
    public $items = array();
    public $hits = array();

    public function __construct() {
        $this->items["box"] = new Milestone1849_Inner();
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
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

function milestone1849_make_outer() {
    global $milestone1849_outer;
    return $milestone1849_outer;
}

$milestone1849_outer = new Milestone1849_Outer();
$alias =& milestone1849_make_outer()["box"][];
$alias = "changed";
$inner = $milestone1849_outer->items["box"];

echo $inner->items[""], "|", $milestone1849_outer->hits[0], "|", $inner->hits[0];
