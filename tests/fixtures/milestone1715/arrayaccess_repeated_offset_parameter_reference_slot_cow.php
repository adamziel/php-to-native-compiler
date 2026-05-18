<?php
class Milestone1715_RepeatedOffsetBag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset][$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset][$offset]);
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$bag = new Milestone1715_RepeatedOffsetBag();
$bag["leaf"] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$bag->items["leaf"]["leaf"]["id"]["function"] = "repeated-set-cow";
$copy = $bag["leaf"];
$copy["id"]["label"] = "repeated-get-cow";
$bag->items["leaf"]["leaf"]["plain"]["function"] = "plain-repeated-mutated";
$bag->items["leaf"]["leaf"]["plain"]["label"] = "plain-repeated-mutated";

echo $function,
    "|",
    $label,
    "|",
    $bag->items["leaf"]["leaf"]["id"]["function"],
    "|",
    $bag->items["leaf"]["leaf"]["id"]["label"],
    "|",
    $copy["id"]["function"],
    "|",
    $copy["id"]["label"],
    "|",
    $bag->items["leaf"]["leaf"]["plain"]["function"],
    "|",
    $bag->items["leaf"]["leaf"]["plain"]["label"];
