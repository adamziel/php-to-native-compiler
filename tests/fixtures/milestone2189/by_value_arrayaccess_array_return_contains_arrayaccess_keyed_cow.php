<?php
error_reporting(0);

class Milestone2189_OuterBag implements ArrayAccess {
    public $data = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->data[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->data[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->data[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2189_InnerBag implements ArrayAccess {
    public $data = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->data[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->data[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->data[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$leaf = "seed";
$plain = "plain";
$outer = new Milestone2189_OuterBag();
$inner = new Milestone2189_InnerBag();

$outer["box"] = array("inner" => $inner);
$outer["box"]["inner"]["slot"] = array("leaf" => &$leaf, "plain" => $plain);
$out = $inner["slot"];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"], "|", isset($outer["box"]["slot"]) ? "bad" : "detached";
