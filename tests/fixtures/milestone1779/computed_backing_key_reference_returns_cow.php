<?php
class Milestone1779_ComputedKeyBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $key = "slot-" . $offset;
        return isset($this->items[$key]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $key = "slot-" . $offset;
        $this->events[] = $key;
        if (!isset($this->items[$key])) {
            $this->items[$key] = [];
        }
        return $this->items[$key];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $key = "slot-" . $offset;
        $this->items[$key] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        $key = "slot-" . $offset;
        unset($this->items[$key]);
    }
}

class Milestone1779_ComputedKeyMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        $key = "slot-" . $name;
        $this->events[] = $key;
        if (!isset($this->store[$key])) {
            $this->store[$key] = [];
        }
        return $this->store[$key];
    }

    public function writeValue($name, $key, $value) {
        $storeKey = "slot-" . $name;
        $this->store[$storeKey][$key]["value"] = $value;
    }

    public function writePlain($name, $key, $value) {
        $storeKey = "slot-" . $name;
        $this->store[$storeKey][$key]["plain"]["value"] = $value;
    }

    public function readPlain($name, $key) {
        $storeKey = "slot-" . $name;
        return $this->store[$storeKey][$key]["plain"]["value"];
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1779_ComputedKeyBag();
$box = new Milestone1779_ComputedKeyMagicBox();

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-computed-key";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-computed-key";
$box->writeValue("missing", "node", "magic-computed-key");
$box->writePlain("missing", "node", "magic-plain-computed-key");

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
