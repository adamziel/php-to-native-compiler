<?php
error_reporting(0);

class Milestone2087_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $key = $offset;
        $this->store[$key] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2087_Bag();
$ref = array("value" => "original");
$payload = array(
    "ref" => &$ref,
    "plain" => array("value" => "plain-original"),
);

$bag["slot"] = $payload;

$copy = $bag->store["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo $ref["value"], "|", $bag->store["slot"]["plain"]["value"];
