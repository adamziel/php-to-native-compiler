<?php
class Milestone1775_GotoBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->events[] = "offset-start";
        goto selected_offset;
        return $this->items["fallback"];
selected_offset:
        $this->events[] = "offset-selected";
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

class Milestone1775_GotoMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        $this->events[] = "magic-start";
        goto selected_magic;
        return $this->store["fallback"];
selected_magic:
        $this->events[] = "magic-selected";
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

$bag = new Milestone1775_GotoBag();
$box = new Milestone1775_GotoMagicBox();

$bag->items["target"] = [];
$box->store["missing"] = [];

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-goto";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-goto";
$box->writeValue("missing", "node", "magic-goto");
$box->writePlain("missing", "node", "magic-plain-goto");

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
