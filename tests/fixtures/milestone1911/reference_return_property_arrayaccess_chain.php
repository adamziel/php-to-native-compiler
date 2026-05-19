<?php
class Milestone1911_InnerBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
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

class Milestone1911_OuterBag implements ArrayAccess {
    public $inner;

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->inner[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }
}

$inner = new Milestone1911_InnerBag();
$outer = new Milestone1911_OuterBag();
$outer->inner = $inner;

$alias =& $outer["slot"];
$alias = "changed";
$inner->items["slot"] = "bucket";

echo $alias, "|", $inner->items["slot"];
