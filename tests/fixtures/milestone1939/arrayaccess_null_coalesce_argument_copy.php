<?php
function milestone1939_mutate($bucket) {
    $bucket["ref"]["value"] = "callee";
    $bucket["plain"]["value"] = "plain-callee";
}

class Milestone1939_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store[$offset] ?? array();
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

$bag = new Milestone1939_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

milestone1939_mutate($bag["slot"]);

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"];
