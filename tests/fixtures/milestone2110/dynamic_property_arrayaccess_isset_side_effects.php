<?php
error_reporting(0);

class Milestone2110_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $this->log[] = "exists:" . $offset;
        $this->store[$offset]["ref"]["value"] = "exists";
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store[$offset];
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

class Milestone2110_Holder {
    public $bag;
}

$ref = array("value" => "original");
$bag = new Milestone2110_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$holder = new Milestone2110_Holder();
$holder->bag = $bag;
$property = "bag";

echo (isset($holder->$property["slot"]) ? "yes" : "no"),
    "|",
    implode(",", $bag->log),
    "|",
    $ref["value"],
    "|",
    $bag->store["slot"]["plain"]["value"];
