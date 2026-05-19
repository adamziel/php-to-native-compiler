<?php
class Milestone2038_Bag implements ArrayAccess {
    public $store = array();

    public function key($name) {
        return $name;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $holder = array();
        $holder["copy"] = $this->store[$this->key($offset)];
        $holder["copy"]["plain"]["value"] = "plain-inside";
        return $holder["copy"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2038_Bag();
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
