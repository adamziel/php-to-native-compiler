<?php
error_reporting(0);

class Milestone2171_SourceBag implements ArrayAccess {
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

class Milestone2171_Holder {
    public $items = array("outer" => array());
    public $prop = "items";
}

$source = new Milestone2171_SourceBag();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$holder = new Milestone2171_Holder();
$p = $holder->prop;
$holder->$p["outer"][] = $source["slot"];
$holder->items["outer"][0]["ref"] = "new";
$holder->items["outer"][0]["plain"] = "copy";

echo $alias,
    "|",
    $source->store["slot"]["ref"],
    "|",
    $holder->items["outer"][0]["ref"],
    "|",
    $source->store["slot"]["plain"],
    "|",
    $holder->items["outer"][0]["plain"];
