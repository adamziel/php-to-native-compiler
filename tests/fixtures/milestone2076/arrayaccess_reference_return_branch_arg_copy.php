<?php
error_reporting(0);

function &milestone2076_pick_ref(&$value) {
    return $value["ref"];
}

class Milestone2076_Bag implements ArrayAccess {
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

$bag = new Milestone2076_Bag();
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
$leaf =& $bag->alt["slot"]["ref"];
$storeLeaf =& $bag->store["slot"]["ref"];

$alias =& milestone2076_pick_ref($bag["slot"]);
$alias["value"] = "copy";

echo $leaf["value"], "|", $bag->alt["slot"]["ref"]["value"], "|",
    $storeLeaf["value"], "|", $bag->alt["slot"]["plain"]["value"];
