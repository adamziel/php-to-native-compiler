<?php
class Milestone1783_ArrayBag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        if (!isset($this->items[$offset])) {
            $this->items[$offset] = [];
        }
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1783_MagicBox {
    public $store = [];

    public function &__get($name) {
        if (!isset($this->store[$name])) {
            $this->store[$name] = [];
        }
        return $this->store[$name];
    }

    public function readPlain($name, $key) {
        return $this->store[$name][$key]["plain"]["value"];
    }
}

class Milestone1783_Holder {
    public $bag;
    public $box;

    public function __construct($bag, $box) {
        $this->bag = $bag;
        $this->box = $box;
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1783_ArrayBag();
$box = new Milestone1783_MagicBox();
$holders = [new Milestone1783_Holder($bag, $box)];

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bagSlot =& $holders[0]->bag["target"]["node"];
$magicSlot =& $holders[0]->box->missing["node"];

$fn = function () use (&$bagSlot, &$magicSlot) {
    $bagSlot["value"] = "bag-nondirect-capture";
    $bagSlot["plain"]["value"] = "bag-plain-nondirect-capture";
    $magicSlot["value"] = "magic-nondirect-capture";
    $magicSlot["plain"]["value"] = "magic-plain-nondirect-capture";
};

$fn();

echo $bagSource,
    "|",
    $bag->items["target"]["node"]["plain"]["value"],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node");
