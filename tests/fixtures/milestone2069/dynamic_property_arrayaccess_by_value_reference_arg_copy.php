<?php
error_reporting(0);

function milestone2069_mutate(&$value) {
    $value["ref"]["value"] = "copy";
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2069_Bag implements ArrayAccess {
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

class Milestone2069_Holder {
    public $bag;
}

$bag = new Milestone2069_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$holder = new Milestone2069_Holder();
$holder->bag = $bag;
$property = "bag";

$result = milestone2069_mutate($holder->{$property}["slot"]);

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $result;
