<?php
function milestone2022_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "plain-inside";
}

class Milestone2022_Holder {
    public $bucket = array();
}

class Milestone2022_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $holder = new Milestone2022_Holder();
        $holder->bucket = $this->store[$offset];
        milestone2022_touch_bucket($holder->bucket);
        return $holder->bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2022_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
echo $alias, "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $bag->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
