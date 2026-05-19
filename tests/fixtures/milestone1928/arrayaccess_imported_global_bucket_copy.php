<?php
class Milestone1928_Bag implements ArrayAccess {
    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        global $store;
        return isset($store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        global $store;
        return $store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        global $store;
        $store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        global $store;
        unset($store[$offset]);
    }
}

$store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $store["slot"]["ref"]["value"];
$bag = new Milestone1928_Bag();

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
