<?php
class Box1747 {
    public int $id = 1;
}

class Bag1747 implements ArrayAccess {
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

class Holder1747 {
    public $bag;
}

class MagicHolder1747 {
    public $bag;

    public function __get($name) {
        return $this->bag;
    }
}

$box = new Box1747();
$alias =& $box->id;

$bag = new Bag1747();
$bag->items["outer"] = array();
$bag->items["outer"]["copy"] =& $alias;
$bag["outer"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"], "\n";

$holder = new Holder1747();
$holder->bag = $bag;
$holder->bag["outer"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"], "\n";

$property = "bag";
$holder->{$property}["outer"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"], "\n";

$holders = array("box" => $holder);
$holders["box"]->bag["outer"]["copy"] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"], "\n";

$dynamicHolders = array("box" => $holder);
$dynamicHolders["box"]->{$property}["outer"]["copy"] = "6";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"], "\n";

$magic = new MagicHolder1747();
$magic->bag = $bag;
$magic->missing["outer"]["copy"] = "7";
echo gettype($box->id), ":", $box->id, "|", gettype($bag->items["outer"]["copy"]), ":", $bag->items["outer"]["copy"];
