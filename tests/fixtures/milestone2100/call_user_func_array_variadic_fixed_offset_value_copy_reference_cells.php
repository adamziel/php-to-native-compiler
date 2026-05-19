<?php
error_reporting(0);

class Milestone2100_Bag implements ArrayAccess {
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

function &milestone2100_pick($label, &...$values) {
    return $values[1]["ref"];
}

$bag = new Milestone2100_Bag();
$ref = "original";
$other = "other-original";
$bag->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");
$args = array("label", array("ref" => &$other), $bag["slot"]);

$alias =& call_user_func_array("milestone2100_pick", $args);
$alias = "copy";

echo $ref, "|", $other, "|", $bag->store["slot"]["plain"];
