<?php
class Milestone1724_PropertyHeldNestedAppendBag implements ArrayAccess {
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

class Milestone1724_PropertyHeldNestedAppendHolder {
    public $bag;
    public $dynamicBag;
}

$directFunction = "direct-original";
$directNode = [
    "function" => &$directFunction,
    "plain" => "direct-plain-original",
];
$directBag = new Milestone1724_PropertyHeldNestedAppendBag();
$directBag->items["outer"] = [];
$directHolder = new Milestone1724_PropertyHeldNestedAppendHolder();
$directHolder->bag = $directBag;
$directHolder->bag["outer"][] = [
    "id" => $directNode,
    "plain" => [
        "function" => "direct-copy-original",
    ],
];
$directBag->items["outer"][0]["id"]["function"] = "direct-nested-append-cow";
$directBag->items["outer"][0]["plain"]["function"] = "direct-nested-copy-mutated";

$nonDirectFunction = "non-direct-original";
$nonDirectNode = [
    "function" => &$nonDirectFunction,
    "plain" => "non-direct-plain-original",
];
$nonDirectBag = new Milestone1724_PropertyHeldNestedAppendBag();
$nonDirectBag->items["outer"] = [];
$nonDirectHolder = new Milestone1724_PropertyHeldNestedAppendHolder();
$nonDirectHolder->bag = $nonDirectBag;
$holders = ["box" => $nonDirectHolder];
$holders["box"]->bag["outer"][] = [
    "id" => $nonDirectNode,
    "plain" => [
        "function" => "non-direct-copy-original",
    ],
];
$nonDirectBag->items["outer"][0]["id"]["function"] = "non-direct-nested-append-cow";
$nonDirectBag->items["outer"][0]["plain"]["function"] = "non-direct-nested-copy-mutated";

$dynamicFunction = "dynamic-original";
$dynamicNode = [
    "function" => &$dynamicFunction,
    "plain" => "dynamic-plain-original",
];
$dynamicBag = new Milestone1724_PropertyHeldNestedAppendBag();
$dynamicBag->items["outer"] = [];
$dynamicHolder = new Milestone1724_PropertyHeldNestedAppendHolder();
$dynamicHolder->dynamicBag = $dynamicBag;
$dynamicHolders = ["box" => $dynamicHolder];
$property = "dynamicBag";
$dynamicHolders["box"]->{$property}["outer"][] = [
    "id" => $dynamicNode,
    "plain" => [
        "function" => "dynamic-copy-original",
    ],
];
$dynamicBag->items["outer"][0]["id"]["function"] = "dynamic-nested-append-cow";
$dynamicBag->items["outer"][0]["plain"]["function"] = "dynamic-nested-copy-mutated";

echo $directFunction,
    "|",
    $directBag->items["outer"][0]["plain"]["function"],
    "|",
    $nonDirectFunction,
    "|",
    $nonDirectBag->items["outer"][0]["plain"]["function"],
    "|",
    $dynamicFunction,
    "|",
    $dynamicBag->items["outer"][0]["plain"]["function"];
