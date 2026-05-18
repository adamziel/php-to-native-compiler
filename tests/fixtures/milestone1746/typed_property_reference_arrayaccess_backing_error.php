<?php
class Box1746 {
    public int $id = 1;
}

class Bag1746 implements ArrayAccess {
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

class Holder1746 {
    public $bag;
}

$box = new Box1746();
$alias =& $box->id;

$bag = new Bag1746();
$bag->items["outer"] = array();
$bag->items["outer"]["copy"] =& $alias;

$holder = new Holder1746();
$holder->bag = $bag;
$holder->bag["outer"]["copy"] = array("bad");
