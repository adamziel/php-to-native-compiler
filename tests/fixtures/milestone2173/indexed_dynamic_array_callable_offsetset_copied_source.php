<?php
error_reporting(0);

class Milestone2173_TargetBag implements ArrayAccess {
    public $store = array();

    public function save($offset, $value) {
        $this->store[$offset] = $value;
    }

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
        $callbacks = array("set" => array($this, "save"));
        $callbacks["set"]($offset, $value);
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = array("slot" => array("ref" => "old", "plain" => "plain"));
$alias =& $source["slot"]["ref"];

$target = new Milestone2173_TargetBag();
$target["slot"] = $source["slot"];
$target->store["slot"]["ref"] = "new";
$target->store["slot"]["plain"] = "copy";

echo $alias,
    "|",
    $source["slot"]["ref"],
    "|",
    $target->store["slot"]["ref"],
    "|",
    $source["slot"]["plain"],
    "|",
    $target->store["slot"]["plain"];
