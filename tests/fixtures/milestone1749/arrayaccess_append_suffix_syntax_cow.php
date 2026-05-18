<?php
class Milestone1749_ArrayAccessSuffixBag implements ArrayAccess {
    public $items = array();

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

class Milestone1749_ArrayAccessSuffixHolder {
    public $bag;
    public $dynamicBag;
}

class Milestone1749_ArrayAccessSuffixMagicHolder {
    public $bag;

    public function __get($name) {
        return $this->bag;
    }
}

$bag = new Milestone1749_ArrayAccessSuffixBag();
$bag->items["outer"] = array();

$bag["outer"][]["leaf"] = "direct-root";

$holder = new Milestone1749_ArrayAccessSuffixHolder();
$holder->bag = $bag;
$holder->dynamicBag = $bag;
$property = "dynamicBag";
$holder->{$property}["outer"][]["leaf"] = "direct-dynamic";

$holders = array("box" => $holder);
$holders["box"]->bag["outer"][]["leaf"] = "non-direct";
$holders["box"]->{$property}["outer"][]["leaf"] = "non-direct-dynamic";

$magic = new Milestone1749_ArrayAccessSuffixMagicHolder();
$magic->bag = $bag;
$holders["magic"] = $magic;
$holders["magic"]->missing["outer"][]["leaf"] = "non-direct-magic";

$function = "original";
$node = array(
    "function" => &$function,
    "plain" => "plain-original",
);
$bag["outer"][]["node"] = array(
    "id" => $node,
    "plain" => array(
        "function" => "copy-original",
    ),
);
$bag->items["outer"][5]["node"]["id"]["function"] = "reference-updated";
$bag->items["outer"][5]["node"]["plain"]["function"] = "copy-updated";

echo $bag->items["outer"][0]["leaf"],
    "|",
    $bag->items["outer"][1]["leaf"],
    "|",
    $bag->items["outer"][2]["leaf"],
    "|",
    $bag->items["outer"][3]["leaf"],
    "|",
    $bag->items["outer"][4]["leaf"],
    "|",
    $function,
    "|",
    $bag->items["outer"][5]["node"]["plain"]["function"];
