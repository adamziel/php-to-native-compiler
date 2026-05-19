<?php
error_reporting(0);

class Milestone2184_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2184_Box {
    public $bags = array();

    public function __construct($bag) {
        $this->bags["selected"] = $bag;
    }

    public function __set($name, $value) {
        $key = "selected";
        $bags = $this->bags;
        $bags[$key][$name] = $value;
    }
}

$leaf = "seed";
$plain = "plain";
$bag = new Milestone2184_Bag();
$box = new Milestone2184_Box($bag);
$src = array("leaf" => &$leaf, "plain" => $plain);

$box->slot = $src;
$out = $bag["slot"];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"];
