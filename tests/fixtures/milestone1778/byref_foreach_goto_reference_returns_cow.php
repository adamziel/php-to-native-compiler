<?php
class Milestone1778_ByRefForeachGotoBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        foreach ($this->items as $candidate => &$slot) {
            $this->events[] = $candidate;
            if ($candidate === $offset) {
                goto selected_offset;
            }
        }
        return $this->items["fallback"];
selected_offset:
        return $slot;
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

class Milestone1778_ByRefForeachGotoMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        foreach ($this->store as $candidate => &$slot) {
            $this->events[] = $candidate;
            if ($candidate === $name) {
                goto selected_magic;
            }
        }
        return $this->store["fallback"];
selected_magic:
        return $slot;
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

$bag = new Milestone1778_ByRefForeachGotoBag();
$box = new Milestone1778_ByRefForeachGotoMagicBox();

$bag->items["skip"] = [];
$bag->items["target"] = [];
$box->store["unused"] = [];
$box->store["missing"] = [];

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-byref-goto";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-byref-goto";
$box->writeValue("missing", "node", "magic-byref-goto");
$box->writePlain("missing", "node", "magic-plain-byref-goto");

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
