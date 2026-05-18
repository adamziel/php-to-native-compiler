<?php
class Milestone1780_InlineComputedKeyBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["slot-" . $offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->events[] = "slot-" . $offset;
        if (!isset($this->items["slot-" . $offset])) {
            $this->items["slot-" . $offset] = [];
        }
        return $this->items["slot-" . $offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items["slot-" . $offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["slot-" . $offset]);
    }
}

class Milestone1780_InlineComputedKeyMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        $this->events[] = "slot-" . $name;
        if (!isset($this->store["slot-" . $name])) {
            $this->store["slot-" . $name] = [];
        }
        return $this->store["slot-" . $name];
    }

    public function writeValue($name, $key, $value) {
        $this->store["slot-" . $name][$key]["value"] = $value;
    }

    public function writePlain($name, $key, $value) {
        $this->store["slot-" . $name][$key]["plain"]["value"] = $value;
    }

    public function readPlain($name, $key) {
        return $this->store["slot-" . $name][$key]["plain"]["value"];
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1780_InlineComputedKeyBag();
$box = new Milestone1780_InlineComputedKeyMagicBox();

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-inline-key";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-inline-key";
$box->writeValue("missing", "node", "magic-inline-key");
$box->writePlain("missing", "node", "magic-plain-inline-key");

echo $bagSource,
    "|",
    $bag->items["slot-target"]["node"]["plain"]["value"],
    "|",
    $bag->events[0],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node"),
    "|",
    $box->events[0];
