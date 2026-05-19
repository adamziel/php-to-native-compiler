<?php
error_reporting(0);

class Milestone2167_SourceBag implements ArrayAccess {
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

class Milestone2167_Holder {
    public $store = array();
}

class Milestone2167_TargetBag implements ArrayAccess {
    public $holder;

    public function __construct() {
        $this->holder = new Milestone2167_Holder();
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->holder->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->holder->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->holder->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = new Milestone2167_SourceBag();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$target = new Milestone2167_TargetBag();
$target["slot"] = $source["slot"];
$target->holder->store["slot"]["ref"] = "new";
$target->holder->store["slot"]["plain"] = "copy";

echo $alias,
    "|",
    $source->store["slot"]["ref"],
    "|",
    $target->holder->store["slot"]["ref"],
    "|",
    $source->store["slot"]["plain"],
    "|",
    $target->holder->store["slot"]["plain"];
