<?php
class Milestone1777_TryFinallyGotoBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        try {
            $this->events[] = "offset-try";
            goto selected_offset;
        } finally {
            $this->events[] = "offset-finally";
        }
        return $this->items["fallback"];
selected_offset:
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

class Milestone1777_TryFinallyGotoMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        try {
            $this->events[] = "magic-try";
            goto selected_magic;
        } finally {
            $this->events[] = "magic-finally";
        }
        return $this->store["fallback"];
selected_magic:
        return $this->store[$name];
    }

    public function writeValue($name, $key, $value) {
        $this->store[$name][$key]["value"] = $value;
    }

    public function writePlain($name, $key, $value) {
        $this->store[$name][$key]["plain"]["value"] = $value;
    }

    public function readPlain($name, $key) {
        return $this->store[$name][$key]["plain"]["value"];
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1777_TryFinallyGotoBag();
$box = new Milestone1777_TryFinallyGotoMagicBox();

$bag->items["target"] = [];
$box->store["missing"] = [];

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-try-goto";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-try-goto";
$box->writeValue("missing", "node", "magic-try-goto");
$box->writePlain("missing", "node", "magic-plain-try-goto");

echo $bagSource,
    "|",
    $bag->items["target"]["node"]["plain"]["value"],
    "|",
    $bag->events[0],
    "|",
    $bag->events[1],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node"),
    "|",
    $box->events[0],
    "|",
    $box->events[1];
