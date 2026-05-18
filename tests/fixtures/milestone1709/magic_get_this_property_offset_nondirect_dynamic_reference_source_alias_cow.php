<?php
class Milestone1709_NonDirectDynamicReferenceSourceBox {
    private $store = [];

    public function &__get($name) {
        return $this->store["bucket"][$name];
    }

    public function mutate() {
        $this->store["bucket"]["missing"]["outer"][0]["id"]["function"] = "non-direct-dynamic-source";
        $this->store["bucket"]["missing"]["outer"][0]["id"]["label"] = "non-direct-dynamic-label";
        $this->store["bucket"]["missing"]["outer"][0]["plain"]["function"] = "plain-non-direct-dynamic-mutated";
        $this->store["bucket"]["missing"]["outer"][0]["plain"]["label"] = "plain-non-direct-dynamic-mutated";
    }

    public function bucket() {
        return $this->store["bucket"]["missing"]["outer"][0];
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$holders = ["box" => new Milestone1709_NonDirectDynamicReferenceSourceBox()];
$property = "missing";
$alias =& $holders["box"]->{$property}["outer"];
$alias[] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$holders["box"]->mutate();
$bucket = $holders["box"]->bucket();

echo $function,
    "|",
    $label,
    "|",
    $bucket["id"]["function"],
    "|",
    $bucket["id"]["label"],
    "|",
    $bucket["plain"]["function"],
    "|",
    $bucket["plain"]["label"];
