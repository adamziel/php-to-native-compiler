<?php
class Milestone1748_ArrayAccessAppendBag implements ArrayAccess {
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

class Milestone1748_ArrayAccessAppendHolder {
    public $bag;
    public $dynamicBag;
}

class Milestone1748_ArrayAccessMagicHolder {
    public $bag;

    public function __get($name) {
        return $this->bag;
    }
}

$bag = new Milestone1748_ArrayAccessAppendBag();
$bag->items["outer"] = array();

$bag["outer"][] = "direct";

$holder = new Milestone1748_ArrayAccessAppendHolder();
$holder->bag = $bag;
$holder->dynamicBag = $bag;
$holder->bag["outer"][] = "property";
$holder->bag["outer"][]["leaf"] = "suffix";

$holders = array("box" => $holder);
$holders["box"]->bag["outer"][] = "non-direct";

$property = "dynamicBag";
$holders["box"]->{$property}["outer"][] = "dynamic";

$magic = new Milestone1748_ArrayAccessMagicHolder();
$magic->bag = $bag;
$magic->missing["outer"][] = "magic";
$magic->missing["outer"][]["leaf"] = "magic-suffix";

$function = "original";
$node = array(
    "function" => &$function,
    "plain" => "plain-original",
);
$holder->bag["outer"][]["node"] = array(
    "id" => $node,
    "plain" => array(
        "function" => "copy-original",
    ),
);
$bag->items["outer"][7]["node"]["id"]["function"] = "reference-updated";
$bag->items["outer"][7]["node"]["plain"]["function"] = "copy-updated";

echo $bag->items["outer"][0],
    "|",
    $bag->items["outer"][1],
    "|",
    $bag->items["outer"][2]["leaf"],
    "|",
    $bag->items["outer"][3],
    "|",
    $bag->items["outer"][4],
    "|",
    $bag->items["outer"][5],
    "|",
    $bag->items["outer"][6]["leaf"],
    "|",
    $function,
    "|",
    $bag->items["outer"][7]["node"]["plain"]["function"];
