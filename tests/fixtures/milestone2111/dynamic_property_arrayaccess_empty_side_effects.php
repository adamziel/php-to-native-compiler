<?php
error_reporting(0);

class Milestone2111_Bag implements ArrayAccess {
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
        $this->log[] = "get:" . $offset;
        $this->store[$offset]["ref"]["value"] = "get";
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

class Milestone2111_Holder {
    public $bag;
}

$ref = array("value" => "original");
$bag = new Milestone2111_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$holder = new Milestone2111_Holder();
$holder->bag = $bag;
$property = "bag";

echo (empty($holder->$property["slot"]) ? "empty" : "filled"),
    "|",
    implode(",", $bag->log),
    "|",
    $ref["value"],
    "|",
    $bag->store["slot"]["plain"]["value"];
