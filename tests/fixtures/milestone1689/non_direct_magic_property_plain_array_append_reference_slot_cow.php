<?php
$function = "seed-function";
$label = "seed-label";
$storage = [];

class Milestone1689_NonDirectMagicPlainArrayAppendBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$holders = ["box" => new Milestone1689_NonDirectMagicPlainArrayAppendBox()];
$holders["box"]->missing[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $storage[0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-non-direct-magic-array-append";
        $callback["label"] = "via-non-direct-magic-array-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $storage[0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
