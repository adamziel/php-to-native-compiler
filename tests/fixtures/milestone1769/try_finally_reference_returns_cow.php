<?php
class Milestone1769_TryBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        try {
            $this->events[] = $offset;
            if (!isset($this->items[$offset])) {
                $this->items[$offset] = [];
            }
            return $this->items[$offset];
        } catch (Exception $e) {
            $this->events[] = "catch";
            return $this->items["catch"];
        } finally {
            $this->events[] = "finally";
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

class Milestone1769_FinallyBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        try {
            $this->events[] = "try";
            return $this->store[$name]["try"];
        } finally {
            $this->events[] = "finally";
            if (!isset($this->store[$name]["finally"])) {
                $this->store[$name]["finally"] = [];
            }
            return $this->store[$name]["finally"];
        }
    }

    public function writeFinallyValue($name, $key, $value) {
        $this->store[$name]["finally"][$key]["value"] = $value;
    }

    public function writeFinallyPlain($name, $key, $value) {
        $this->store[$name]["finally"][$key]["plain"]["value"] = $value;
    }

    public function readFinallyPlain($name, $key) {
        return $this->store[$name]["finally"][$key]["plain"]["value"];
    }

    public function tryBucketType($name) {
        return gettype($this->store[$name]["try"]);
    }
}

$trySource = "try-seed";
$magicSource = "magic-seed";

$tryNode = ["value" => &$trySource, "plain" => ["value" => "try-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1769_TryBag();
$box = new Milestone1769_FinallyBox();

$bag["slot"]["node"] = $tryNode;
$box->missing["node"] = $magicNode;

$bag->items["slot"]["node"]["value"] = "try-finally";
$bag->items["slot"]["node"]["plain"]["value"] = "try-plain-finally";

$box->writeFinallyValue("missing", "node", "magic-finally");
$box->writeFinallyPlain("missing", "node", "magic-plain-finally");

echo $trySource,
    "|",
    $bag->items["slot"]["node"]["plain"]["value"],
    "|",
    $bag->events[0],
    "|",
    $bag->events[1],
    "|",
    $magicSource,
    "|",
    $box->readFinallyPlain("missing", "node"),
    "|",
    $box->events[0],
    "|",
    $box->events[1],
    "|",
    $box->tryBucketType("missing");
