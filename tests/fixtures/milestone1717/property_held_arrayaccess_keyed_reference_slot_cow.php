<?php
class Milestone1717_PropertyHeldKeyedBag implements ArrayAccess {
    public $items = [];

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

class Milestone1717_PropertyHeldKeyedHolder {
    public $bag;
}

$directFunction = "direct-original";
$directLabel = "direct-label-original";
$directNode = [
    "function" => &$directFunction,
    "label" => &$directLabel,
];

$directHolder = new Milestone1717_PropertyHeldKeyedHolder();
$directBag = new Milestone1717_PropertyHeldKeyedBag();
$directHolder->bag = $directBag;
$directHolder->bag["leaf"] = [
    "id" => $directNode,
    "plain" => [
        "function" => "direct-plain-original",
        "label" => "direct-plain-original",
    ],
];

$directBag->items["leaf"]["id"]["function"] = "direct-property-keyed-cow";
$directBag->items["leaf"]["id"]["label"] = "direct-property-keyed-label";
$directBag->items["leaf"]["plain"]["function"] = "direct-plain-mutated";
$directBag->items["leaf"]["plain"]["label"] = "direct-plain-mutated";

$nonDirectFunction = "non-direct-original";
$nonDirectLabel = "non-direct-label-original";
$nonDirectNode = [
    "function" => &$nonDirectFunction,
    "label" => &$nonDirectLabel,
];

$nonDirectHolder = new Milestone1717_PropertyHeldKeyedHolder();
$nonDirectBag = new Milestone1717_PropertyHeldKeyedBag();
$nonDirectHolder->bag = $nonDirectBag;
$holders = ["box" => $nonDirectHolder];
$holders["box"]->bag["leaf"] = [
    "id" => $nonDirectNode,
    "plain" => [
        "function" => "non-direct-plain-original",
        "label" => "non-direct-plain-original",
    ],
];

$nonDirectBag->items["leaf"]["id"]["function"] = "non-direct-property-keyed-cow";
$nonDirectBag->items["leaf"]["id"]["label"] = "non-direct-property-keyed-label";
$nonDirectBag->items["leaf"]["plain"]["function"] = "non-direct-plain-mutated";
$nonDirectBag->items["leaf"]["plain"]["label"] = "non-direct-plain-mutated";

$dynamicFunction = "dynamic-original";
$dynamicLabel = "dynamic-label-original";
$dynamicNode = [
    "function" => &$dynamicFunction,
    "label" => &$dynamicLabel,
];

$dynamicHolder = new Milestone1717_PropertyHeldKeyedHolder();
$dynamicBag = new Milestone1717_PropertyHeldKeyedBag();
$dynamicHolder->bag = $dynamicBag;
$dynamicHolders = ["box" => $dynamicHolder];
$property = "bag";
$dynamicHolders["box"]->{$property}["leaf"] = [
    "id" => $dynamicNode,
    "plain" => [
        "function" => "dynamic-plain-original",
        "label" => "dynamic-plain-original",
    ],
];

$dynamicBag->items["leaf"]["id"]["function"] = "dynamic-property-keyed-cow";
$dynamicBag->items["leaf"]["id"]["label"] = "dynamic-property-keyed-label";
$dynamicBag->items["leaf"]["plain"]["function"] = "dynamic-plain-mutated";
$dynamicBag->items["leaf"]["plain"]["label"] = "dynamic-plain-mutated";

echo $directFunction,
    "|",
    $directLabel,
    "|",
    $directBag->items["leaf"]["plain"]["function"],
    "|",
    $nonDirectFunction,
    "|",
    $nonDirectLabel,
    "|",
    $nonDirectBag->items["leaf"]["plain"]["function"],
    "|",
    $dynamicFunction,
    "|",
    $dynamicLabel,
    "|",
    $dynamicBag->items["leaf"]["plain"]["function"];
