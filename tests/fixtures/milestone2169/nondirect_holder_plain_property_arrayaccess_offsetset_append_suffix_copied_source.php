<?php
error_reporting(0);

class Milestone2169_SourceBag implements ArrayAccess {
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

class Milestone2169_Holder {
    public $store = array("outer" => array());
}

class Milestone2169_TargetBag implements ArrayAccess {
    public $holder;

    public function __construct() {
        $this->holder = new Milestone2169_Holder();
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->holder->store["outer"][$offset]["payload"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->holder->store["outer"][]["payload"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = new Milestone2169_SourceBag();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$target = new Milestone2169_TargetBag();
$target["ignored"] = $source["slot"];
$target->holder->store["outer"][0]["payload"]["ref"] = "new";
$target->holder->store["outer"][0]["payload"]["plain"] = "copy";

echo $alias,
    "|",
    $source->store["slot"]["ref"],
    "|",
    $target->holder->store["outer"][0]["payload"]["ref"],
    "|",
    $source->store["slot"]["plain"],
    "|",
    $target->holder->store["outer"][0]["payload"]["plain"];
