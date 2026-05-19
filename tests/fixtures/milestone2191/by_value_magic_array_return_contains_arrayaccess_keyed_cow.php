<?php
error_reporting(0);

class Milestone2191_InnerBag implements ArrayAccess {
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

class Milestone2191_Box {
    public $data = array();

    public function __get($name) {
        return $this->data[$name];
    }
}

$leaf = "seed";
$plain = "plain";
$inner = new Milestone2191_InnerBag();
$box = new Milestone2191_Box();

$box->data["box"] = array("inner" => $inner);
$box->box["inner"]["slot"] = array("leaf" => &$leaf, "plain" => $plain);
$out = $inner["slot"];
$out["leaf"] = "changed";
$out["plain"] = "changed-plain";

echo $leaf, "|", $plain, "|", $out["leaf"], "|", $out["plain"], "|", isset($box->box["slot"]) ? "bad" : "detached";
