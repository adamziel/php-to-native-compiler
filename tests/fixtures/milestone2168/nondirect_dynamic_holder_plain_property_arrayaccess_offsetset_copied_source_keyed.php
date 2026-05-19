<?php
error_reporting(0);

class Milestone2168_SourceBag implements ArrayAccess {
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

class Milestone2168_Holder {
    public $store = array();
}

class Milestone2168_TargetBag implements ArrayAccess {
    public $holder;
    public $prop = "store";

    public function __construct() {
        $this->holder = new Milestone2168_Holder();
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $p = $this->prop;
        return isset($this->holder->$p[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $p = $this->prop;
        return $this->holder->$p[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $p = $this->prop;
        $this->holder->$p[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = new Milestone2168_SourceBag();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$target = new Milestone2168_TargetBag();
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
