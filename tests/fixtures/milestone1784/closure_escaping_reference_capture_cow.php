<?php
class Milestone1784_ArrayBag implements ArrayAccess {
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

class Milestone1784_MagicBox {
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

function milestone1784_make_closure($bag, $box) {
    $bagSlot =& $bag["target"]["node"];
    $magicSlot =& $box->missing["node"];

    return function () use (&$bagSlot, &$magicSlot) {
        $bagSlot["value"] = "bag-escaping-capture";
        $bagSlot["plain"]["value"] = "bag-plain-escaping-capture";
        $magicSlot["value"] = "magic-escaping-capture";
        $magicSlot["plain"]["value"] = "magic-plain-escaping-capture";
    };
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1784_ArrayBag();
$box = new Milestone1784_MagicBox();

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$fn = milestone1784_make_closure($bag, $box);
$fn();

echo $bagSource,
    "|",
    $bag->items["target"]["node"]["plain"]["value"],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node");
