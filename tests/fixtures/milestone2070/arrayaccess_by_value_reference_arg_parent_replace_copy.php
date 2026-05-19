<?php
error_reporting(0);

function milestone2070_replace_parent(&$value) {
    $value["ref"] = array("value" => "parent-copy");
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2070_Bag implements ArrayAccess {
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
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2070_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$result = milestone2070_replace_parent($bag["slot"]);

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $result;
