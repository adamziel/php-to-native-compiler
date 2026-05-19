<?php
error_reporting(0);

function milestone2072_mutate(&$value) {
    $value["ref"]["value"] = "copy";
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2072_Bag implements ArrayAccess {
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

class Milestone2072_Holder {
    public $bag;
}

$bag = new Milestone2072_Bag();
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
$holder = new Milestone2072_Holder();
$holder->bag = $bag;
$property = "bag";
$alias =& $bag->alt["slot"]["ref"]["value"];
$storeAlias =& $bag->store["slot"]["ref"]["value"];

$result = milestone2072_mutate($holder->{$property}["slot"]);

echo $alias, "|", $bag->alt["slot"]["ref"]["value"], "|", $storeAlias, "|",
    $bag->alt["slot"]["plain"]["value"], "|", $result;
