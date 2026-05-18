<?php
class Milestone1718_PropertyHeldNestedKeyedBag implements ArrayAccess {
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

class Milestone1718_PropertyHeldNestedKeyedHolder {
    public $bag;
}

$directFunction = "direct-original";
$directLabel = "direct-label-original";
$directNode = [
    "function" => &$directFunction,
    "label" => &$directLabel,
];

$directBag = new Milestone1718_PropertyHeldNestedKeyedBag();
$directBag->items["outer"] = [];
$directHolder = new Milestone1718_PropertyHeldNestedKeyedHolder();
$directHolder->bag = $directBag;
$directHolder->bag["outer"]["leaf"] = [
    "id" => $directNode,
    "plain" => [
        "function" => "direct-plain-original",
        "label" => "direct-plain-original",
    ],
];

$directBag->items["outer"]["leaf"]["id"]["function"] = "direct-nested-keyed-cow";
$directBag->items["outer"]["leaf"]["id"]["label"] = "direct-nested-keyed-label";
$directBag->items["outer"]["leaf"]["plain"]["function"] = "direct-nested-plain-mutated";
$directBag->items["outer"]["leaf"]["plain"]["label"] = "direct-nested-plain-mutated";

$nonDirectFunction = "non-direct-original";
$nonDirectLabel = "non-direct-label-original";
$nonDirectNode = [
    "function" => &$nonDirectFunction,
    "label" => &$nonDirectLabel,
];

$nonDirectBag = new Milestone1718_PropertyHeldNestedKeyedBag();
$nonDirectBag->items["outer"] = [];
$nonDirectHolder = new Milestone1718_PropertyHeldNestedKeyedHolder();
$nonDirectHolder->bag = $nonDirectBag;
$holders = ["box" => $nonDirectHolder];
$holders["box"]->bag["outer"]["leaf"] = [
    "id" => $nonDirectNode,
    "plain" => [
        "function" => "non-direct-plain-original",
        "label" => "non-direct-plain-original",
    ],
];

$nonDirectBag->items["outer"]["leaf"]["id"]["function"] = "non-direct-nested-keyed-cow";
$nonDirectBag->items["outer"]["leaf"]["id"]["label"] = "non-direct-nested-keyed-label";
$nonDirectBag->items["outer"]["leaf"]["plain"]["function"] = "non-direct-nested-plain-mutated";
$nonDirectBag->items["outer"]["leaf"]["plain"]["label"] = "non-direct-nested-plain-mutated";

$dynamicFunction = "dynamic-original";
$dynamicLabel = "dynamic-label-original";
$dynamicNode = [
    "function" => &$dynamicFunction,
    "label" => &$dynamicLabel,
];

$dynamicBag = new Milestone1718_PropertyHeldNestedKeyedBag();
$dynamicBag->items["outer"] = [];
$dynamicHolder = new Milestone1718_PropertyHeldNestedKeyedHolder();
$dynamicHolder->bag = $dynamicBag;
$dynamicHolders = ["box" => $dynamicHolder];
$property = "bag";
$dynamicHolders["box"]->{$property}["outer"]["leaf"] = [
    "id" => $dynamicNode,
    "plain" => [
        "function" => "dynamic-plain-original",
        "label" => "dynamic-plain-original",
    ],
];

$dynamicBag->items["outer"]["leaf"]["id"]["function"] = "dynamic-nested-keyed-cow";
$dynamicBag->items["outer"]["leaf"]["id"]["label"] = "dynamic-nested-keyed-label";
$dynamicBag->items["outer"]["leaf"]["plain"]["function"] = "dynamic-nested-plain-mutated";
$dynamicBag->items["outer"]["leaf"]["plain"]["label"] = "dynamic-nested-plain-mutated";

echo $directFunction,
    "|",
    $directLabel,
    "|",
    $directBag->items["outer"]["leaf"]["plain"]["function"],
    "|",
    $nonDirectFunction,
    "|",
    $nonDirectLabel,
    "|",
    $nonDirectBag->items["outer"]["leaf"]["plain"]["function"],
    "|",
    $dynamicFunction,
    "|",
    $dynamicLabel,
    "|",
    $dynamicBag->items["outer"]["leaf"]["plain"]["function"];
