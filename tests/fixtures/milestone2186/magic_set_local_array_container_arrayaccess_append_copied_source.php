<?php
error_reporting(0);

class Milestone2186_Bag implements ArrayAccess {
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
        $this->store[] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2186_Box {
    public $bags = array();

    public function __construct($bag) {
        $this->bags["selected"] = $bag;
    }

    public function __set($name, $value) {
        $key = "selected";
        $bags = $this->bags;
        $bags[$key][] = $value;
    }
}

$leaf = "seed";
$plain = "plain";
$bag = new Milestone2186_Bag();
$box = new Milestone2186_Box($bag);
$src = array("leaf" => &$leaf, "plain" => $plain);

$box->slot = $src;
$out = $bag[0];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"];
