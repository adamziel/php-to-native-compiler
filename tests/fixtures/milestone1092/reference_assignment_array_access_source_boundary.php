<?php
class Bag implements ArrayAccess {
    public $items = [];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}

$bag = new Bag();
$alias =& $bag["name"];
