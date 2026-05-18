<?php
class Milestone1721_DynamicMagicKeyedBag implements ArrayAccess {
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

class Milestone1721_DynamicMagicKeyedBox {
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

$keyedBag = new Milestone1721_DynamicMagicKeyedBag();
$keyedBox = new Milestone1721_DynamicMagicKeyedBox();
$keyedBox->bag = $keyedBag;
$property = "missing";
$keyedBox->{$property}["leaf"] = [
    "id" => $keyedNode,
    "plain" => [
        "function" => "keyed-plain-original",
        "label" => "keyed-plain-original",
    ],
];

$keyedBag->items["leaf"]["id"]["function"] = "dynamic-magic-keyed-cow";
$keyedBag->items["leaf"]["id"]["label"] = "dynamic-magic-keyed-label";
$keyedBag->items["leaf"]["plain"]["function"] = "dynamic-magic-keyed-plain-mutated";
$keyedBag->items["leaf"]["plain"]["label"] = "dynamic-magic-keyed-plain-mutated";

$nestedFunction = "nested-original";
$nestedLabel = "nested-label-original";
$nestedNode = [
    "function" => &$nestedFunction,
    "label" => &$nestedLabel,
];

$nestedBag = new Milestone1721_DynamicMagicKeyedBag();
$nestedBag->items["outer"] = [];
$nestedBox = new Milestone1721_DynamicMagicKeyedBox();
$nestedBox->bag = $nestedBag;
$nestedProperty = "missing";
$nestedBox->{$nestedProperty}["outer"]["leaf"] = [
    "id" => $nestedNode,
    "plain" => [
        "function" => "nested-plain-original",
        "label" => "nested-plain-original",
    ],
];

$nestedBag->items["outer"]["leaf"]["id"]["function"] = "dynamic-magic-nested-cow";
$nestedBag->items["outer"]["leaf"]["id"]["label"] = "dynamic-magic-nested-label";
$nestedBag->items["outer"]["leaf"]["plain"]["function"] = "dynamic-magic-nested-plain-mutated";
$nestedBag->items["outer"]["leaf"]["plain"]["label"] = "dynamic-magic-nested-plain-mutated";

$appendFunction = "append-original";
$appendLabel = "append-label-original";
$appendNode = [
    "function" => &$appendFunction,
    "label" => &$appendLabel,
];

$appendBag = new Milestone1721_DynamicMagicKeyedBag();
$appendBox = new Milestone1721_DynamicMagicKeyedBox();
$appendBox->bag = $appendBag;
$appendProperty = "missing";
$appendBox->{$appendProperty}[] = [
    "id" => $appendNode,
    "plain" => [
        "function" => "append-plain-original",
        "label" => "append-plain-original",
    ],
];

$appendBag->items[""]["id"]["function"] = "dynamic-magic-append-cow";
$appendBag->items[""]["id"]["label"] = "dynamic-magic-append-label";
$appendBag->items[""]["plain"]["function"] = "dynamic-magic-append-plain-mutated";
$appendBag->items[""]["plain"]["label"] = "dynamic-magic-append-plain-mutated";

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
    $nestedBag->items["outer"]["leaf"]["plain"]["function"],
    "|",
    $appendFunction,
    "|",
    $appendLabel,
    "|",
    $appendBag->items[""]["plain"]["function"];
