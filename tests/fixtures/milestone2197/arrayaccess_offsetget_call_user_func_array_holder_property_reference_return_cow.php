<?php
error_reporting(0);

function &milestone2197_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2197_ArgsHolder {
    public $args = array();
}

class Milestone2197_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $holder = new Milestone2197_ArgsHolder();
        $holder->args = array($bucket);
        $alias =& call_user_func_array("milestone2197_pick_ref", $holder->args);
        $alias = "inside";
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2197_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
echo $ref, "|", $copy["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
