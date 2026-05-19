<?php
error_reporting(0);

class Milestone2086_Bag implements ArrayAccess {
    public $store = array();
    public $trace = array();

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
        $this->trace[] = "set";
        $key = $offset;
        $this->store[$key] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2086_Bag();
$ref = array("value" => "original");

$bag["slot"] = array(
    "ref" => &$ref,
    "plain" => array("value" => "plain-original"),
);

$copy = $bag->store["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo $ref["value"], "|", $bag->store["slot"]["plain"]["value"], "|", $bag->trace[0];
