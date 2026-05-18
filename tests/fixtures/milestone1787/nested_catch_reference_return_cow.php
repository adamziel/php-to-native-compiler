<?php
class Milestone1787_ArrayBag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        try {
            throw new Exception();
        } catch (Exception $e) {
            foreach (["skip", "target"] as $candidate) {
                if ($candidate !== "target") {
                    continue;
                }
                if (!isset($this->items[$offset][$candidate])) {
                    $this->items[$offset][$candidate] = [];
                }
                return $this->items[$offset][$candidate];
            }
        }
        return $this->items[$offset]["fallback"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset]["target"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$source = "nested-seed";
$node = ["value" => &$source, "plain" => ["value" => "nested-copy"]];

$bag = new Milestone1787_ArrayBag();
$bag["slot"] = $node;
$bag["slot"]["value"] = "nested-caught";
$bag["slot"]["plain"]["value"] = "nested-plain-caught";

echo $source, "|", $bag->items["slot"]["target"]["plain"]["value"];
