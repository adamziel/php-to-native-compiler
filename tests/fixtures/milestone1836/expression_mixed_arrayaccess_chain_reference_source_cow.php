<?php
error_reporting(0);

class Milestone1836Leaf implements ArrayAccess {
    public $items = array("leaf" => "seed");
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

class Milestone1836Outer implements ArrayAccess {
    public $leaf;
    public $hits = array();

    public function __construct($leaf) {
        $this->leaf = $leaf;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return $offset === "outer";
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->hits[] = $offset;
        return $this->leaf;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }
}

$leaf = new Milestone1836Leaf();
$outer = new Milestone1836Outer($leaf);

function milestone1836_make_outer() {
    global $outer;
    return $outer;
}

$alias =& milestone1836_make_outer()["outer"]["leaf"];
$alias = "changed";

echo $leaf->items["leaf"], "|", implode(",", $outer->hits), "|", implode(",", $leaf->hits);
