<?php
error_reporting(0);

class Milestone2115_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
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
        $this->log[] = "unset:" . $offset;
        $this->store[$offset]["ref"]["value"] = "unset";
        unset($this->store[$offset]["plain"]);
    }
}

class Milestone2115_Holder {
    public $bag;
}

$ref = array("value" => "original");
$bag = new Milestone2115_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$holder = new Milestone2115_Holder();
$holder->bag = $bag;
$property = "bag";

unset($holder->$property["slot"]);

echo (isset($bag->store["slot"]) ? "still" : "gone"),
    "|",
    (isset($bag->store["slot"]["plain"]) ? "plain" : "plain-gone"),
    "|",
    implode(",", $bag->log),
    "|",
    $ref["value"];
