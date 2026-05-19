<?php
class Milestone2045_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store["bucket"][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2045_Bag();
$bag->store = array(
    "bucket" => array(
        "slot" => array(
            "ref" => array("value" => "original"),
            "plain" => array("value" => "plain-original"),
        ),
    ),
);
$alias =& $bag->store["bucket"]["slot"]["ref"]["value"];

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $bag->store["bucket"]["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $bag->store["bucket"]["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
