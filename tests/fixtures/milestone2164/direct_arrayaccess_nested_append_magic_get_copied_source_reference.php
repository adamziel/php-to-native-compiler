<?php
error_reporting(0);

class Milestone2164_SourceBox {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone2164_TargetBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = new Milestone2164_SourceBox();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$target = new Milestone2164_TargetBag();
$target->items["outer"] = array();

$target["outer"][] = $source->slot;
$target->items["outer"][0]["ref"] = "new";
$target->items["outer"][0]["plain"] = "copy";

echo $alias,
    "|",
    $source->store["slot"]["ref"],
    "|",
    $target->items["outer"][0]["ref"],
    "|",
    $source->store["slot"]["plain"],
    "|",
    $target->items["outer"][0]["plain"];
