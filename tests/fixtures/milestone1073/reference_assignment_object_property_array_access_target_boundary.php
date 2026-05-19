<?php
error_reporting(0);

class Bag implements ArrayAccess {
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return null; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
}
$holder = new Holder();
$holder->bag = new Bag();
$value = "Grace";
$holder->bag["name"] =& $value;
echo $value;
