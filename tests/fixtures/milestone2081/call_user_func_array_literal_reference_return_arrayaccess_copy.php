<?php
error_reporting(0);

function &milestone2081_pick_ref(&$value) {
    return $value["ref"];
}

class Milestone2081_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        if ($offset === "slot") {
            $bucket =& $this->store[$offset];
        } else {
            $bucket =& $this->store["slot"];
        }
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2081_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$leaf =& $bag->store["slot"]["ref"];

$alias =& call_user_func_array(
    "milestone2081_pick_ref",
    array($bag["slot"])
);
$alias["value"] = "copy";

echo $leaf["value"], "|", $bag->store["slot"]["plain"]["value"];
