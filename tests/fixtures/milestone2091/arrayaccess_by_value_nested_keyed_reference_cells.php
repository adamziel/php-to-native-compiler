<?php
error_reporting(0);

class Milestone2091_Bag implements ArrayAccess {
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

$bag = new Milestone2091_Bag();
$ref = "original";
$child = "child-original";
$bag->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");

$bag["slot"]["ref"] = "copy";
$first = $ref;
$bag["slot"]["plain"] = "plain-copy";
$bag["slot"]["ref"] = array("child" => &$child);

$copy = $ref;
$copy["child"] = "child-copy";

echo $first, "|", $child, "|", $bag->store["slot"]["plain"];
