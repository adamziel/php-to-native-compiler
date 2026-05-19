<?php
error_reporting(0);

class Milestone2187_Inner implements ArrayAccess {
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

class Milestone2187_Outer implements ArrayAccess {
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
        $GLOBALS["milestone2187_registry"][$key][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$leaf = "seed";
$plain = "plain";
$inner = new Milestone2187_Inner();
$GLOBALS["milestone2187_registry"] = array("selected" => $inner);
$outer = new Milestone2187_Outer();
$src = array("leaf" => &$leaf, "plain" => $plain);

$outer["slot"] = $src;
$out = $inner["slot"];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"];
