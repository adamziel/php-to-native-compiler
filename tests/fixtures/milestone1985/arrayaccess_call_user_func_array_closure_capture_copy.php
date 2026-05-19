<?php
class Milestone1985_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $helper = function() use ($bucket) {
            return $bucket;
        };
        $args = array();
        return call_user_func_array($helper, $args);
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone1985_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $bag->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
