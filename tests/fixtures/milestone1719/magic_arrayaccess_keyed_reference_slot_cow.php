<?php
class Milestone1719_MagicKeyedBag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
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

class Milestone1719_MagicKeyedBox {
    public $bag;

    public function __get($name) {
        return $this->bag;
    }
}

$keyedFunction = "keyed-original";
$keyedLabel = "keyed-label-original";
$keyedNode = [
    "function" => &$keyedFunction,
    "label" => &$keyedLabel,
];

$keyedBag = new Milestone1719_MagicKeyedBag();
$keyedBox = new Milestone1719_MagicKeyedBox();
$keyedBox->bag = $keyedBag;
$keyedBox->missing["leaf"] = [
    "id" => $keyedNode,
    "plain" => [
        "function" => "keyed-plain-original",
        "label" => "keyed-plain-original",
    ],
];

$keyedBag->items["leaf"]["id"]["function"] = "magic-keyed-cow";
$keyedBag->items["leaf"]["id"]["label"] = "magic-keyed-label";
$keyedBag->items["leaf"]["plain"]["function"] = "magic-keyed-plain-mutated";
$keyedBag->items["leaf"]["plain"]["label"] = "magic-keyed-plain-mutated";

$nestedFunction = "nested-original";
$nestedLabel = "nested-label-original";
$nestedNode = [
    "function" => &$nestedFunction,
    "label" => &$nestedLabel,
];

$nestedBag = new Milestone1719_MagicKeyedBag();
$nestedBag->items["outer"] = [];
$nestedBox = new Milestone1719_MagicKeyedBox();
$nestedBox->bag = $nestedBag;
$nestedBox->missing["outer"]["leaf"] = [
    "id" => $nestedNode,
    "plain" => [
        "function" => "nested-plain-original",
        "label" => "nested-plain-original",
    ],
];

$nestedBag->items["outer"]["leaf"]["id"]["function"] = "magic-nested-keyed-cow";
$nestedBag->items["outer"]["leaf"]["id"]["label"] = "magic-nested-keyed-label";
$nestedBag->items["outer"]["leaf"]["plain"]["function"] = "magic-nested-plain-mutated";
$nestedBag->items["outer"]["leaf"]["plain"]["label"] = "magic-nested-plain-mutated";

echo $keyedFunction,
    "|",
    $keyedLabel,
    "|",
    $keyedBag->items["leaf"]["plain"]["function"],
    "|",
    $nestedFunction,
    "|",
    $nestedLabel,
    "|",
    $nestedBag->items["outer"]["leaf"]["plain"]["function"];
