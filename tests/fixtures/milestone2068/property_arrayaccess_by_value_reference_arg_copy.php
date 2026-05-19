<?php
error_reporting(0);

function milestone2068_mutate(&$value) {
    $value["ref"]["value"] = "copy";
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2068_Bag implements ArrayAccess {
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

class Milestone2068_Holder {
    public $bag;
}

$bag = new Milestone2068_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$holder = new Milestone2068_Holder();
$holder->bag = $bag;

$result = milestone2068_mutate($holder->bag["slot"]);

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $result;
