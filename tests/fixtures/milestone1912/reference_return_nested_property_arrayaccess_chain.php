<?php
class Milestone1912_InnerBag implements ArrayAccess {
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

class Milestone1912_OuterBag implements ArrayAccess {
    public $inner;

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->inner[$offset]["leaf"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }
}

$inner = new Milestone1912_InnerBag();
$inner->items["slot"] = array("leaf" => "seed");
$outer = new Milestone1912_OuterBag();
$outer->inner = $inner;

$alias =& $outer["slot"];
$alias = "changed";
$inner->items["slot"]["leaf"] = "bucket";

echo $alias, "|", $inner->items["slot"]["leaf"];
