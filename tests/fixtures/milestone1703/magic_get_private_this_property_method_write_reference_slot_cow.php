<?php
class Milestone1703_PrivateThisPropertyMagicBox {
    private $store = ["outer" => []];

    public function &__get($name) {
        return $this->store;
    }

    public function mutate() {
        $this->store["outer"][0]["id"]["function"] = "private-method-magic-append";
        $this->store["outer"][0]["id"]["label"] = "private-method-magic-label";
        $this->store["outer"][0]["id"]["accepted_args"] = 6;
        $this->store["outer"][0]["plain"]["function"] = "plain-private-mutated";
        $this->store["outer"][0]["plain"]["label"] = "plain-private-mutated";
    }

    public function bucket() {
        return $this->store["outer"][0];
    }
}

$function = "original";
$label = "label-original";
$node = [
    "function" => &$function,
    "label" => &$label,
    "accepted_args" => 1,
];

$box = new Milestone1703_PrivateThisPropertyMagicBox();
$box->missing["outer"][] = [
    "id" => $node,
    "plain" => [
        "function" => "plain-original",
        "label" => "plain-original",
        "accepted_args" => 1,
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
    $bucket["id"]["accepted_args"],
    "|",
    $bucket["plain"]["function"],
    "|",
    $bucket["plain"]["label"];
