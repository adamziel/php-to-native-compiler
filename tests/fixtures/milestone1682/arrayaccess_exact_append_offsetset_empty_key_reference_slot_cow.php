<?php
class Milestone1682_ExactAppendStoredBucket implements ArrayAccess {
    private $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
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

$target = "seed";
$node = ["function" => "placeholder", "accepted_args" => 1];
$node["function"] =& $target;

$bag = new Milestone1682_ExactAppendStoredBucket();
$bag[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "accepted_args" => 1],
];

$bucket = $bag[null];
$bucket["id"]["function"] = "via-empty-key";
$bucket["id"]["accepted_args"] = 2;
$bucket["plain"]["function"] = "plain-copy";

$again = $bag[null];
echo $target, "|", $again["id"]["function"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"];
