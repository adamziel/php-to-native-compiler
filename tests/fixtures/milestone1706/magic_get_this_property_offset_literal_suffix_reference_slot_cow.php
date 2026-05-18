<?php
class Milestone1706_ThisPropertyOffsetSuffixMagicBox {
    private $store = [];

    public function &__get($name) {
        return $this->store[$name]["bucket"];
    }

    public function mutate() {
        $this->store["missing"]["bucket"]["outer"][0]["id"]["function"] = "offset-suffix-magic-append";
        $this->store["missing"]["bucket"]["outer"][0]["id"]["label"] = "offset-suffix-magic-label";
        $this->store["missing"]["bucket"]["outer"][0]["plain"]["function"] = "plain-offset-suffix-mutated";
        $this->store["missing"]["bucket"]["outer"][0]["plain"]["label"] = "plain-offset-suffix-mutated";
    }

    public function bucket() {
        return $this->store["missing"]["bucket"]["outer"][0];
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$box = new Milestone1706_ThisPropertyOffsetSuffixMagicBox();
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
