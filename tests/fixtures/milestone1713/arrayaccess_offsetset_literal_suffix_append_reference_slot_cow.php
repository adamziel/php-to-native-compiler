<?php
class Milestone1713_SuffixAppendBag implements ArrayAccess {
    public $items = [];

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
        if ($offset === null) {
            $this->items[]["bucket"] = $value;
            return;
        }
        $this->items[$offset]["bucket"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]["bucket"]);
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$bag = new Milestone1713_SuffixAppendBag();
$bag[] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$bag->items[0]["bucket"]["id"]["function"] = "append-suffix-cow";
$bag->items[0]["bucket"]["id"]["label"] = "append-suffix-label";
$bag->items[0]["bucket"]["plain"]["function"] = "plain-suffix-mutated";
$bag->items[0]["bucket"]["plain"]["label"] = "plain-suffix-mutated";

echo $function,
    "|",
    $label,
    "|",
    $bag->items[0]["bucket"]["id"]["function"],
    "|",
    $bag->items[0]["bucket"]["id"]["label"],
    "|",
    $bag->items[0]["bucket"]["plain"]["function"],
    "|",
    $bag->items[0]["bucket"]["plain"]["label"];
