<?php
class Milestone1711_PrefixSetBag implements ArrayAccess {
    public $items = ["bucket" => []];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["bucket"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items["bucket"][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items["bucket"][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["bucket"][$offset]);
    }
}

class Milestone1711_SuffixSetBag implements ArrayAccess {
    public $items = ["leaf" => ["bucket" => []]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]["bucket"]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items[$offset]["bucket"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset]["bucket"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]["bucket"]);
    }
}

$prefixFunction = "prefix-original";
$prefixLabel = "prefix-label-original";
$prefixNode = [
    "function" => &$prefixFunction,
    "label" => &$prefixLabel,
];
$prefixBag = new Milestone1711_PrefixSetBag();
$prefixBag["leaf"] = ["id" => $prefixNode];
$prefixBag->items["bucket"]["leaf"]["id"]["function"] = "set-prefix-cow";
$prefixBag->items["bucket"]["leaf"]["id"]["label"] = "set-prefix-label";

$suffixFunction = "suffix-original";
$suffixLabel = "suffix-label-original";
$suffixNode = [
    "function" => &$suffixFunction,
    "label" => &$suffixLabel,
];
$suffixBag = new Milestone1711_SuffixSetBag();
$suffixBag["leaf"] = ["id" => $suffixNode];
$suffixBag->items["leaf"]["bucket"]["id"]["function"] = "set-suffix-cow";
$suffixBag->items["leaf"]["bucket"]["id"]["label"] = "set-suffix-label";

echo $prefixFunction,
    "|",
    $prefixLabel,
    "|",
    $prefixBag->items["bucket"]["leaf"]["id"]["function"],
    "|",
    $prefixBag->items["bucket"]["leaf"]["id"]["label"],
    "\n",
    $suffixFunction,
    "|",
    $suffixLabel,
    "|",
    $suffixBag->items["leaf"]["bucket"]["id"]["function"],
    "|",
    $suffixBag->items["leaf"]["bucket"]["id"]["label"];
