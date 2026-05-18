<?php
class Milestone1716_IfElseKeyedBag implements ArrayAccess {
    public $items = ["outer" => []];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["outer"][$offset]["leaf"]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items["outer"][$offset]["leaf"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        if ($offset === null) {
            $this->items["outer"][]["leaf"] = $value;
        } else {
            $this->items["outer"][$offset]["leaf"] = $value;
        }
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["outer"][$offset]["leaf"]);
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$bag = new Milestone1716_IfElseKeyedBag();
$bag["leaf"] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$bag->items["outer"]["leaf"]["leaf"]["id"]["function"] = "keyed-if-else-cow";
$bag->items["outer"]["leaf"]["leaf"]["id"]["label"] = "keyed-if-else-label";
$bag->items["outer"]["leaf"]["leaf"]["plain"]["function"] = "plain-keyed-if-else-mutated";
$bag->items["outer"]["leaf"]["leaf"]["plain"]["label"] = "plain-keyed-if-else-mutated";

echo $function,
    "|",
    $label,
    "|",
    $bag->items["outer"]["leaf"]["leaf"]["id"]["function"],
    "|",
    $bag->items["outer"]["leaf"]["leaf"]["id"]["label"],
    "|",
    $bag->items["outer"]["leaf"]["leaf"]["plain"]["function"],
    "|",
    $bag->items["outer"]["leaf"]["leaf"]["plain"]["label"];
