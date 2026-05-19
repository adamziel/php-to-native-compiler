<?php
error_reporting(0);

class Milestone2099_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = array(
            "ref" => &$this->store[$offset]["ref"],
            "plain" => $this->store[$offset]["plain"],
        );
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

function &milestone2099_pick(&...$values) {
    return $values[0]["ref"];
}

$bag = new Milestone2099_Bag();
$ref = "original";
$bag->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");

$alias =& call_user_func_array("milestone2099_pick", array($bag["slot"]));
$alias = "copy";

echo $ref, "|", $bag->store["slot"]["plain"];
