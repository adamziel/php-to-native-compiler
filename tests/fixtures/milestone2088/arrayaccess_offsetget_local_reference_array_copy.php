<?php
error_reporting(0);

class Milestone2088_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = array(
            "ref" => &$this->store[$offset]["ref"],
            "plain" => array("value" => "plain-original"),
        );
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2088_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
    ),
);
$leaf =& $bag->store["slot"]["ref"];

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo $leaf["value"], "|", $copy["plain"]["value"];
