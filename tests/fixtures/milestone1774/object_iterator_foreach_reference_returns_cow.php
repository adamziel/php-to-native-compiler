<?php
class Milestone1774_SelectorObject {
    public $skip = "ignored";
    public $target = "selected";
}

class Milestone1774_SelectorIterator implements Iterator {
    public $keys = ["unused", "missing"];
    public $values = ["ignored", "selected"];
    public $position = 0;

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->position = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return $this->position < 2;
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->position];
    }

    #[ReturnTypeWillChange]
    public function current() {
        return $this->values[$this->position];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->position = $this->position + 1;
    }
}

class Milestone1774_ObjectForeachBag implements ArrayAccess {
    public $items = [];
    public $events = [];
    public $selector;

    public function __construct() {
        $this->selector = new Milestone1774_SelectorObject();
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        foreach ($this->selector as $candidate => $label) {
            $this->events[] = $candidate . ":" . $label;
            if ($candidate !== $offset) {
                continue;
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

class Milestone1774_IteratorForeachMagicBox {
    public $store = [];
    public $events = [];
    public $selector;

    public function __construct() {
        $this->selector = new Milestone1774_SelectorIterator();
    }

    public function &__get($name) {
        foreach ($this->selector as $candidate => $label) {
            $this->events[] = $candidate . ":" . $label;
            if ($candidate !== $name) {
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

$bag = new Milestone1774_ObjectForeachBag();
$box = new Milestone1774_IteratorForeachMagicBox();

$bag->items["target"] = [];
$box->store["missing"] = [];

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-object-foreach";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-object-foreach";
$box->writeValue("missing", "node", "magic-iterator-foreach");
$box->writePlain("missing", "node", "magic-plain-iterator-foreach");

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
