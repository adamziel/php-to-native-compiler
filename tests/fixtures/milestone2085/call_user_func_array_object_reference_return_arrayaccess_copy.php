<?php
error_reporting(0);

class Milestone2085_Picker {
    public function &pick(&$value) {
        return $value["ref"];
    }
}

class Milestone2085_Bag implements ArrayAccess {
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

$picker = new Milestone2085_Picker();
$bag = new Milestone2085_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$leaf =& $bag->store["slot"]["ref"];
$args = array($bag["slot"]);

$alias =& call_user_func_array(array($picker, "pick"), $args);
$alias["value"] = "copy";

echo $leaf["value"], "|", $bag->store["slot"]["plain"]["value"];
