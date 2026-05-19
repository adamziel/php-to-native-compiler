<?php
class Milestone1936_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $offset === "slot" ? $this->store[$offset] : array();
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$bag = new Milestone1936_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $bag->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
