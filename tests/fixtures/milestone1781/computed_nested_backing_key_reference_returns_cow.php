<?php
class Milestone1781_ComputedNestedKeyBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $bucket = "bucket-" . $offset;
        return isset($this->items[$offset][$bucket]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $bucket = "bucket-" . $offset;
        $this->events[] = $bucket;
        if (!isset($this->items[$offset])) {
            $this->items[$offset] = [];
        }
        if (!isset($this->items[$offset][$bucket])) {
            $this->items[$offset][$bucket] = [];
        }
        return $this->items[$offset][$bucket];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $bucket = "bucket-" . $offset;
        $this->items[$offset][$bucket] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        $bucket = "bucket-" . $offset;
        unset($this->items[$offset][$bucket]);
    }
}

class Milestone1781_ComputedNestedKeyMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        $bucket = "bucket-" . $name;
        $this->events[] = $bucket;
        if (!isset($this->store[$name])) {
            $this->store[$name] = [];
        }
        if (!isset($this->store[$name][$bucket])) {
            $this->store[$name][$bucket] = [];
        }
        return $this->store[$name][$bucket];
    }

    public function writeValue($name, $key, $value) {
        $bucket = "bucket-" . $name;
        $this->store[$name][$bucket][$key]["value"] = $value;
    }

    public function writePlain($name, $key, $value) {
        $bucket = "bucket-" . $name;
        $this->store[$name][$bucket][$key]["plain"]["value"] = $value;
    }

    public function readPlain($name, $key) {
        $bucket = "bucket-" . $name;
        return $this->store[$name][$bucket][$key]["plain"]["value"];
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1781_ComputedNestedKeyBag();
$box = new Milestone1781_ComputedNestedKeyMagicBox();

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-nested-key";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-nested-key";
$box->writeValue("missing", "node", "magic-nested-key");
$box->writePlain("missing", "node", "magic-plain-nested-key");

echo $bagSource,
    "|",
    $bag->items["target"]["bucket-target"]["node"]["plain"]["value"],
    "|",
    $bag->events[0],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node"),
    "|",
    $box->events[0];
