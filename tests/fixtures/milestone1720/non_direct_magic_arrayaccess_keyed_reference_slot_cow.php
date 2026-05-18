<?php
class Milestone1720_NonDirectMagicKeyedBag implements ArrayAccess {
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

class Milestone1720_NonDirectMagicKeyedBox {
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

$keyedBag = new Milestone1720_NonDirectMagicKeyedBag();
$keyedBox = new Milestone1720_NonDirectMagicKeyedBox();
$keyedBox->bag = $keyedBag;
$holders = ["box" => $keyedBox];
$holders["box"]->missing["leaf"] = [
    "id" => $keyedNode,
    "plain" => [
        "function" => "keyed-plain-original",
        "label" => "keyed-plain-original",
    ],
];

$keyedBag->items["leaf"]["id"]["function"] = "non-direct-magic-keyed-cow";
$keyedBag->items["leaf"]["id"]["label"] = "non-direct-magic-keyed-label";
$keyedBag->items["leaf"]["plain"]["function"] = "non-direct-magic-keyed-plain-mutated";
$keyedBag->items["leaf"]["plain"]["label"] = "non-direct-magic-keyed-plain-mutated";

$nestedFunction = "nested-original";
$nestedLabel = "nested-label-original";
$nestedNode = [
    "function" => &$nestedFunction,
    "label" => &$nestedLabel,
];

$nestedBag = new Milestone1720_NonDirectMagicKeyedBag();
$nestedBag->items["outer"] = [];
$nestedBox = new Milestone1720_NonDirectMagicKeyedBox();
$nestedBox->bag = $nestedBag;
$nestedHolders = ["box" => $nestedBox];
$nestedHolders["box"]->missing["outer"]["leaf"] = [
    "id" => $nestedNode,
    "plain" => [
        "function" => "nested-plain-original",
        "label" => "nested-plain-original",
    ],
];

$nestedBag->items["outer"]["leaf"]["id"]["function"] = "non-direct-magic-nested-cow";
$nestedBag->items["outer"]["leaf"]["id"]["label"] = "non-direct-magic-nested-label";
$nestedBag->items["outer"]["leaf"]["plain"]["function"] = "non-direct-magic-nested-plain-mutated";
$nestedBag->items["outer"]["leaf"]["plain"]["label"] = "non-direct-magic-nested-plain-mutated";

$dynamicFunction = "dynamic-original";
$dynamicLabel = "dynamic-label-original";
$dynamicNode = [
    "function" => &$dynamicFunction,
    "label" => &$dynamicLabel,
];

$dynamicBag = new Milestone1720_NonDirectMagicKeyedBag();
$dynamicBox = new Milestone1720_NonDirectMagicKeyedBox();
$dynamicBox->bag = $dynamicBag;
$dynamicHolders = ["box" => $dynamicBox];
$property = "missing";
$dynamicHolders["box"]->{$property}["leaf"] = [
    "id" => $dynamicNode,
    "plain" => [
        "function" => "dynamic-plain-original",
        "label" => "dynamic-plain-original",
    ],
];

$dynamicBag->items["leaf"]["id"]["function"] = "non-direct-dynamic-magic-keyed-cow";
$dynamicBag->items["leaf"]["id"]["label"] = "non-direct-dynamic-magic-keyed-label";
$dynamicBag->items["leaf"]["plain"]["function"] = "non-direct-dynamic-magic-keyed-plain-mutated";
$dynamicBag->items["leaf"]["plain"]["label"] = "non-direct-dynamic-magic-keyed-plain-mutated";

$appendFunction = "append-original";
$appendLabel = "append-label-original";
$appendNode = [
    "function" => &$appendFunction,
    "label" => &$appendLabel,
];

$appendBag = new Milestone1720_NonDirectMagicKeyedBag();
$appendBox = new Milestone1720_NonDirectMagicKeyedBox();
$appendBox->bag = $appendBag;
$appendHolders = ["box" => $appendBox];
$appendHolders["box"]->missing[] = [
    "id" => $appendNode,
    "plain" => [
        "function" => "append-plain-original",
        "label" => "append-plain-original",
    ],
];

$appendBag->items[""]["id"]["function"] = "non-direct-magic-append-cow";
$appendBag->items[""]["id"]["label"] = "non-direct-magic-append-label";
$appendBag->items[""]["plain"]["function"] = "non-direct-magic-append-plain-mutated";
$appendBag->items[""]["plain"]["label"] = "non-direct-magic-append-plain-mutated";

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
    $dynamicFunction,
    "|",
    $dynamicLabel,
    "|",
    $dynamicBag->items["leaf"]["plain"]["function"],
    "|",
    $appendFunction,
    "|",
    $appendLabel,
    "|",
    $appendBag->items[""]["plain"]["function"];
