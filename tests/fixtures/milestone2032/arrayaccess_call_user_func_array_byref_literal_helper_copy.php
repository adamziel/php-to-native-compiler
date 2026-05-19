<?php
function milestone2032_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "plain-inside";
    return $value;
}

class Milestone2032_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        return call_user_func_array("milestone2032_touch_bucket", array(&$bucket));
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2032_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $bag->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $bag->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
