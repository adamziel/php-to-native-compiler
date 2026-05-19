<?php
error_reporting(0);

function milestone2071_mutate(&$value) {
    $value["ref"]["value"] = "copy";
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2071_Bag implements ArrayAccess {
    public $store = array();
    public $alt = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        if ($offset === "slot") {
            $bucket =& $this->alt[$offset];
        } else {
            $bucket =& $this->store[$offset];
        }
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2071_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "store"),
        "plain" => array("value" => "store-plain"),
    ),
);
$bag->alt = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->alt["slot"]["ref"]["value"];
$storeAlias =& $bag->store["slot"]["ref"]["value"];

$result = milestone2071_mutate($bag["slot"]);

echo $alias, "|", $bag->alt["slot"]["ref"]["value"], "|", $storeAlias, "|",
    $bag->alt["slot"]["plain"]["value"], "|", $result;
