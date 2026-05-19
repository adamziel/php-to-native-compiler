<?php
error_reporting(0);

class Milestone2165_SourceBag implements ArrayAccess {
    public $store = array();

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
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2165_TargetBag implements ArrayAccess {
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

class Milestone2165_Holder {
    public $bag;
}

$holder = new Milestone2165_Holder();
$holder->bag = new Milestone2165_SourceBag();
$bag = $holder->bag;
$bag->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $bag->store["slot"]["ref"];

$target = new Milestone2165_TargetBag();
$target->items["outer"] = array();

$target["outer"][] = $holder->bag["slot"];
$target->items["outer"][0]["ref"] = "new";
$target->items["outer"][0]["plain"] = "copy";

echo $alias,
    "|",
    $bag->store["slot"]["ref"],
    "|",
    $target->items["outer"][0]["ref"],
    "|",
    $bag->store["slot"]["plain"],
    "|",
    $target->items["outer"][0]["plain"];
