<?php
$function = "seed-function";
$label = "seed-label";
$storage = ["outer" => []];

class Milestone1690_MagicPlainArrayNestedAppendBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$box = new Milestone1690_MagicPlainArrayNestedAppendBox();
$box->missing["outer"][] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $storage["outer"][0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-magic-nested-append";
        $callback["label"] = "via-magic-nested-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $storage["outer"][0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
