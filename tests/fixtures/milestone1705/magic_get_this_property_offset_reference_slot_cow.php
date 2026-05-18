<?php
class Milestone1705_ThisPropertyOffsetMagicBox {
    private $store = [];

    public function &__get($name) {
        return $this->store[$name];
    }

    public function mutate() {
        $this->store["missing"]["outer"][0]["id"]["function"] = "offset-private-magic-append";
        $this->store["missing"]["outer"][0]["id"]["label"] = "offset-private-magic-label";
        $this->store["missing"]["outer"][0]["plain"]["function"] = "plain-offset-mutated";
        $this->store["missing"]["outer"][0]["plain"]["label"] = "plain-offset-mutated";
    }

    public function bucket() {
        return $this->store["missing"]["outer"][0];
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$box = new Milestone1705_ThisPropertyOffsetMagicBox();
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
