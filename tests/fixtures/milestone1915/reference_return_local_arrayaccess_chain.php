<?php
class Milestone1915_InnerBag implements ArrayAccess {
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

class Milestone1915_OuterBag implements ArrayAccess {
    public $inner;
    public $trace = array();

    private function &pick($bag, $offset) {
        $this->trace[] = "pick:" . $offset;
        return $bag[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get:" . $offset;
        return $this->pick($this->inner, $offset);
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }
}

$inner = new Milestone1915_InnerBag();
$outer = new Milestone1915_OuterBag();
$outer->inner = $inner;

$alias =& $outer["slot"];
$alias = "changed";
$inner->items["slot"] = "bucket";

echo $alias, "|", $inner->items["slot"], "|", implode(",", $outer->trace);
