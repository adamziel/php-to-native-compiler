<?php
error_reporting(0);

class Milestone2185_Inner implements ArrayAccess {
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

class Milestone2185_Outer implements ArrayAccess {
    public $inners = array();

    public function __construct($inner) {
        $this->inners["selected"] = $inner;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return null;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $key = "selected";
        $inners = $this->inners;
        $inners[$key][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$leaf = "seed";
$plain = "plain";
$inner = new Milestone2185_Inner();
$outer = new Milestone2185_Outer($inner);
$src = array("leaf" => &$leaf, "plain" => $plain);

$outer["slot"] = $src;
$out = $inner["slot"];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"];
