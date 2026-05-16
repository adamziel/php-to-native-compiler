<?php
class Bag implements ArrayAccess {
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return null; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}
$bag = new Bag();
$value = "Grace";
$bag["name"] =& $value;
