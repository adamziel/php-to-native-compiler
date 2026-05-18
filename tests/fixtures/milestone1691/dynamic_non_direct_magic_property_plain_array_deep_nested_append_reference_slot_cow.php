<?php
$function = "seed-function";
$label = "seed-label";
$storage = ["outer" => ["inner" => []]];

class Milestone1691_DynamicNonDirectMagicPlainArrayDeepNestedAppendBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$property = "missing";
$holders = ["box" => new Milestone1691_DynamicNonDirectMagicPlainArrayDeepNestedAppendBox()];
$holders["box"]->{$property}["outer"]["inner"][] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $storage["outer"]["inner"][0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-dynamic-non-direct-magic-deep-nested-append";
        $callback["label"] = "via-dynamic-non-direct-magic-deep-nested-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $storage["outer"]["inner"][0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
