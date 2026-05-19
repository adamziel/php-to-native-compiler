<?php
error_reporting(0);

class Milestone2106_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = array(
            "ref" => &$this->store[$offset]["ref"],
            "plain" => $this->store[$offset]["plain"],
        );
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2106_Box {
    public $store = array();

    public function __set($name, $value) {
        $key = $name;
        $this->store[$key] = $value;
    }
}

$ref = "original";
$bag = new Milestone2106_Bag();
$bag->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");

$box = new Milestone2106_Box();
$name = "slot";
$box->$name = $bag["slot"];

$box->store["slot"]["ref"] = "copy";
$box->store["slot"]["plain"] = "local";

echo $ref, "|", $bag->store["slot"]["plain"], "|", $box->store["slot"]["plain"];
