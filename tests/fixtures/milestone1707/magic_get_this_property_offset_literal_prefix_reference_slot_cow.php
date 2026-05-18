<?php
class Milestone1707_ThisPropertyOffsetPrefixMagicBox {
    private $store = [];

    public function &__get($name) {
        return $this->store["bucket"][$name];
    }

    public function mutate() {
        $this->store["bucket"]["missing"]["outer"][0]["id"]["function"] = "offset-prefix-magic-append";
        $this->store["bucket"]["missing"]["outer"][0]["id"]["label"] = "offset-prefix-magic-label";
        $this->store["bucket"]["missing"]["outer"][0]["plain"]["function"] = "plain-offset-prefix-mutated";
        $this->store["bucket"]["missing"]["outer"][0]["plain"]["label"] = "plain-offset-prefix-mutated";
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

$box = new Milestone1707_ThisPropertyOffsetPrefixMagicBox();
$box->missing["outer"][] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
    ],
];

$box->mutate();
$bucket = $box->bucket();

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
