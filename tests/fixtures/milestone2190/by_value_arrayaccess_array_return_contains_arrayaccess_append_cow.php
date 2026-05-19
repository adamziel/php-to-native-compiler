<?php
error_reporting(0);

class Milestone2190_OuterBag implements ArrayAccess {
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

class Milestone2190_InnerBag implements ArrayAccess {
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
        $this->data[] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$leaf = "seed";
$plain = "plain";
$outer = new Milestone2190_OuterBag();
$inner = new Milestone2190_InnerBag();

$outer["box"] = array("inner" => $inner);
$outer["box"]["inner"][] = array("leaf" => &$leaf, "plain" => $plain);
$out = $inner[0];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"], "|", isset($outer["box"][0]) ? "bad" : "detached";
