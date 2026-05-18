<?php
class Milestone1704_DynamicPrivateThisPropertyMagicBox {
    private $missing = ["outer" => []];

    public function &__get($name) {
        return $this->{$name};
    }

    public function mutate() {
        $this->missing["outer"][0]["id"]["function"] = "dynamic-private-magic-append";
        $this->missing["outer"][0]["id"]["label"] = "dynamic-private-magic-label";
        $this->missing["outer"][0]["plain"]["function"] = "plain-dynamic-private-mutated";
        $this->missing["outer"][0]["plain"]["label"] = "plain-dynamic-private-mutated";
    }

    public function bucket() {
        return $this->missing["outer"][0];
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
];

$box = new Milestone1704_DynamicPrivateThisPropertyMagicBox();
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
