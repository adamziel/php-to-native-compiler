<?php
class Milestone1738_Bag implements ArrayAccess {
    public $items = [
        "parent" => false,
        "appendParent" => false,
        "dynamicParent" => false,
        "nonDirectParent" => false,
        "nonDirectDynamicParent" => false,
    ];

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

class Milestone1738_Holder {
    public $bag;
}

class Milestone1738_MagicBox {
    public $store = [
        "missing" => [
            "parent" => false,
        ],
    ];

    public function &__get($name) {
        return $this->store[$name];
    }
}

error_reporting(0);

$arrayAccessValue = "arrayaccess-original";
$bag = new Milestone1738_Bag();
$holder = new Milestone1738_Holder();
$holder->bag = $bag;
$holder->bag["parent"]["leaf"] =& $arrayAccessValue;
$arrayAccessValue = "arrayaccess-variable";
$bag->items["parent"]["leaf"] = "arrayaccess-bucket";

$appendValue = "append-original";
$holder->bag["appendParent"][] =& $appendValue;
$appendValue = "append-variable";
$bag->items["appendParent"][0] = "append-bucket";

$dynamicValue = "dynamic-original";
$property = "bag";
$holder->{$property}["dynamicParent"]["leaf"] =& $dynamicValue;
$dynamicValue = "dynamic-variable";
$bag->items["dynamicParent"]["leaf"] = "dynamic-bucket";

$nonDirectValue = "nondirect-original";
$holders = ["box" => $holder];
$holders["box"]->bag["nonDirectParent"]["leaf"] =& $nonDirectValue;
$nonDirectValue = "nondirect-variable";
$bag->items["nonDirectParent"]["leaf"] = "nondirect-bucket";

$nonDirectDynamicValue = "nondirect-dynamic-original";
$holders["box"]->{$property}["nonDirectDynamicParent"]["leaf"] =& $nonDirectDynamicValue;
$nonDirectDynamicValue = "nondirect-dynamic-variable";
$bag->items["nonDirectDynamicParent"]["leaf"] = "nondirect-dynamic-bucket";

$magicValue = "magic-original";
$magic = new Milestone1738_MagicBox();
$magic->missing["parent"]["leaf"] =& $magicValue;
$magicValue = "magic-variable";
$magic->store["missing"]["parent"]["leaf"] = "magic-bucket";

echo $arrayAccessValue,
    "|",
    $bag->items["parent"]["leaf"],
    "|",
    $appendValue,
    "|",
    $bag->items["appendParent"][0],
    "|",
    $dynamicValue,
    "|",
    $bag->items["dynamicParent"]["leaf"],
    "|",
    $nonDirectValue,
    "|",
    $bag->items["nonDirectParent"]["leaf"],
    "|",
    $nonDirectDynamicValue,
    "|",
    $bag->items["nonDirectDynamicParent"]["leaf"],
    "|",
    $magicValue,
    "|",
    $magic->store["missing"]["parent"]["leaf"];
