<?php
class Milestone1712_PrefixAppendBag implements ArrayAccess {
    public $items = ["bucket" => []];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["bucket"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items["bucket"][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        if ($offset === null) {
            $this->items["bucket"][] = $value;
            return;
        }
        $this->items["bucket"][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["bucket"][$offset]);
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$bag = new Milestone1712_PrefixAppendBag();
$bag[] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$bag->items["bucket"][0]["id"]["function"] = "append-prefix-cow";
$bag->items["bucket"][0]["id"]["label"] = "append-prefix-label";
$bag->items["bucket"][0]["plain"]["function"] = "plain-prefix-mutated";
$bag->items["bucket"][0]["plain"]["label"] = "plain-prefix-mutated";

echo $function,
    "|",
    $label,
    "|",
    $bag->items["bucket"][0]["id"]["function"],
    "|",
    $bag->items["bucket"][0]["id"]["label"],
    "|",
    $bag->items["bucket"][0]["plain"]["function"],
    "|",
    $bag->items["bucket"][0]["plain"]["label"];
