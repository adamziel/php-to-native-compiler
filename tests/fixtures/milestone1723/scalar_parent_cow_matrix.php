<?php
class Milestone1723_Bag implements ArrayAccess {
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

class Milestone1723_Holder {
    public $items = [];
    public $bag;
}

class Milestone1723_MagicBox {
    public $store = [];

    public function &__get($name) {
        return $this->store[$name];
    }
}

error_reporting(0);

$directFalseValue = "direct-false-original";
$directFalseNode = ["function" => &$directFalseValue];
$directFalse = ["parent" => false];
$directFalse["parent"]["leaf"] = ["id" => $directFalseNode];
$directFalse["parent"]["leaf"]["id"]["function"] = "direct-false-cow";

$directNullValue = "direct-null-original";
$directNullNode = ["function" => &$directNullValue];
$directNull = ["parent" => null];
$directNull["parent"][] = ["id" => $directNullNode];
$directNull["parent"][0]["id"]["function"] = "direct-null-append-cow";

$objectFalseValue = "object-false-original";
$objectFalseNode = ["function" => &$objectFalseValue];
$holder = new Milestone1723_Holder();
$holder->items["parent"] = false;
$holder->items["parent"]["leaf"] = ["id" => $objectFalseNode];
$holder->items["parent"]["leaf"]["id"]["function"] = "object-false-cow";

$magicFalseValue = "magic-false-original";
$magicFalseNode = ["function" => &$magicFalseValue];
$magic = new Milestone1723_MagicBox();
$magic->store["missing"]["parent"] = false;
$magic->missing["parent"]["leaf"] = ["id" => $magicFalseNode];
$magic->store["missing"]["parent"]["leaf"]["id"]["function"] = "magic-false-cow";

$arrayAccessFalseValue = "arrayaccess-false-original";
$arrayAccessFalseNode = ["function" => &$arrayAccessFalseValue];
$bag = new Milestone1723_Bag();
$bag->items["parent"] = false;
$bagHolder = new Milestone1723_Holder();
$bagHolder->bag = $bag;
$bagHolder->bag["parent"]["leaf"] = ["id" => $arrayAccessFalseNode];
$bag->items["parent"]["leaf"]["id"]["function"] = "arrayaccess-false-cow";

echo $directFalseValue,
    "|",
    $directNullValue,
    "|",
    $objectFalseValue,
    "|",
    $magicFalseValue,
    "|",
    $arrayAccessFalseValue;
