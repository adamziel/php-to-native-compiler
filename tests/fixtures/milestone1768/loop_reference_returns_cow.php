<?php
class Milestone1768_WhileBag implements ArrayAccess {
    public $items = [];
    public $hits = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
        while (true) {
            if (!isset($this->items[$offset])) {
                $this->items[$offset] = [];
            }
            return $this->items[$offset];
        }
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

class Milestone1768_ForBag implements ArrayAccess {
    public $items = [];
    public $hits = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
        for ($i = 0; $i < 1; $i++) {
            if (!isset($this->items[$offset])) {
                $this->items[$offset] = [];
            }
            return $this->items[$offset];
        }
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

class Milestone1768_MagicBox {
    public $store = [];
    public $hits = [];

    public function &__get($name) {
        $this->hits[] = $name;
        do {
            if (!isset($this->store[$name])) {
                $this->store[$name] = [];
            }
            return $this->store[$name];
        } while (false);
    }

    public function read($name, $key, $field) {
        return $this->store[$name][$key][$field]["value"];
    }
}

$whileSource = "while-seed";
$forSource = "for-seed";
$magicSource = "magic-seed";

$whileNode = ["value" => &$whileSource, "plain" => ["value" => "while-copy"]];
$forNode = ["value" => &$forSource, "plain" => ["value" => "for-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$whileBag = new Milestone1768_WhileBag();
$forBag = new Milestone1768_ForBag();
$box = new Milestone1768_MagicBox();

$whileBag["while"]["node"] = $whileNode;
$forBag["for"]["node"] = $forNode;
$box->magicLoop["node"] = $magicNode;

$whileBag->items["while"]["node"]["value"] = "while-loop";
$forBag->items["for"]["node"]["value"] = "for-loop";
$box->magicLoop["node"]["value"] = "magic-loop";

$whileBag->items["while"]["node"]["plain"]["value"] = "while-plain-loop";
$forBag->items["for"]["node"]["plain"]["value"] = "for-plain-loop";
$box->magicLoop["node"]["plain"]["value"] = "magic-plain-loop";

echo $whileSource,
    "|",
    $forSource,
    "|",
    $magicSource,
    "|",
    $whileBag->items["while"]["node"]["plain"]["value"],
    "|",
    $forBag->items["for"]["node"]["plain"]["value"],
    "|",
    $box->read("magicLoop", "node", "plain"),
    "|",
    $whileBag->hits[0],
    "|",
    $forBag->hits[0],
    "|",
    $box->hits[0],
    "|",
    $box->hits[1];
