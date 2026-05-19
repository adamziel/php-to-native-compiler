<?php
error_reporting(0);

class Milestone2188_Bag implements ArrayAccess {
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

class Milestone2188_Box {
    public function __set($name, $value) {
        $GLOBALS["milestone2188_bag"][] = $value;
    }
}

$leaf = "seed";
$plain = "plain";
$bag = new Milestone2188_Bag();
$GLOBALS["milestone2188_bag"] = $bag;
$box = new Milestone2188_Box();
$src = array("leaf" => &$leaf, "plain" => $plain);

$box->slot = $src;
$out = $bag[0];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"];
