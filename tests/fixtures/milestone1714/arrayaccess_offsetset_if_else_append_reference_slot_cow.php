<?php
class Milestone1714_IfElseAppendBag implements ArrayAccess {
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

$bag = new Milestone1714_IfElseAppendBag();
$bag[] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$bag->items["outer"][0]["leaf"]["id"]["function"] = "append-if-else-cow";
$bag->items["outer"][0]["leaf"]["id"]["label"] = "append-if-else-label";
$bag->items["outer"][0]["leaf"]["plain"]["function"] = "plain-if-else-mutated";
$bag->items["outer"][0]["leaf"]["plain"]["label"] = "plain-if-else-mutated";

echo $function,
    "|",
    $label,
    "|",
    $bag->items["outer"][0]["leaf"]["id"]["function"],
    "|",
    $bag->items["outer"][0]["leaf"]["id"]["label"],
    "|",
    $bag->items["outer"][0]["leaf"]["plain"]["function"],
    "|",
    $bag->items["outer"][0]["leaf"]["plain"]["label"];
