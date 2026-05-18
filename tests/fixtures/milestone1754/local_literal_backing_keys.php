<?php
class Milestone1754_Box {
    public int $id = 1;
}

class Milestone1754_Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $leaf = "leaf";
        $bucket =& $this->items[$offset];
        return $bucket[$leaf];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Milestone1754_MagicBox {
    private $store = array();

    public function seed(&$value) {
        $this->store["missing"]["leaf"]["copy"] =& $value;
    }

    public function &__get($name) {
        $leaf = "leaf";
        $bucket =& $this->store[$name];
        return $bucket[$leaf];
    }

    public function read($name, $key) {
        return gettype($this->store[$name]["leaf"][$key]) . ":" . $this->store[$name]["leaf"][$key];
    }
}

$box = new Milestone1754_Box();
$alias =& $box->id;

$bag = new Milestone1754_Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["leaf"]["copy"]), ":", $bag->items["outer"]["leaf"]["copy"], "\n";

$magic = new Milestone1754_MagicBox();
$magic->seed($alias);
$magic->missing["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", $magic->read("missing", "copy");
