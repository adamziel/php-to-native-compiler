<?php
class Milestone1772_ForeachBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        foreach (["skip", $offset] as $candidate) {
            $this->events[] = $candidate;
            if ($candidate === "skip") {
                continue;
            }
            if (!isset($this->items[$candidate])) {
                $this->items[$candidate] = [];
            }
            return $this->items[$candidate];
        }
        return $this->items["fallback"];
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

class Milestone1772_MagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        foreach (["unused", $name] as $index => $candidate) {
            $this->events[] = $index . ":" . $candidate;
            if ($candidate === "unused") {
                continue;
            }
            return $this->store[$candidate];
        }
        return $this->store["fallback"];
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

$bag = new Milestone1772_ForeachBag();
$box = new Milestone1772_MagicBox();

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag->items["target"]["node"]["value"] = "bag-foreach";
$bag->items["target"]["node"]["plain"]["value"] = "bag-plain-foreach";
$box->writeValue("missing", "node", "magic-foreach");
$box->writePlain("missing", "node", "magic-plain-foreach");

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
