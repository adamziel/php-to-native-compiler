<?php
class Milestone1752_Box {
    public int $id = 1;
}

class Milestone1752_Bag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $bucket =& $this->items[$offset];
        return $bucket["leaf"];
    }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Milestone1752_MagicBox {
    private $store = array();

    public function seed(&$value) {
        $this->store["missing"]["copy"] =& $value;
    }

    public function &__get($name) {
        $bucket =& $this->store[$name];
        return $bucket;
    }

    public function read($name, $key) {
        return gettype($this->store[$name][$key]) . ":" . $this->store[$name][$key];
    }
}

$box = new Milestone1752_Box();
$alias =& $box->id;

$bag = new Milestone1752_Bag();
$bag->items["outer"]["leaf"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|",
    gettype($bag->items["outer"]["leaf"]["copy"]), ":", $bag->items["outer"]["leaf"]["copy"], "|",
    gettype($alias), ":", $alias, "\n";

$magic = new Milestone1752_MagicBox();
$magic->seed($alias);
$magic->missing["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", $magic->read("missing", "copy"), "|", gettype($alias), ":", $alias;
