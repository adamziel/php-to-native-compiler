<?php
class Milestone1708_ThisPropertyOffsetReferenceSourceBox {
    private $store = [];

    public function &__get($name) {
        return $this->store[$name];
    }

    public function mutate() {
        $this->store["missing"]["outer"][0]["id"]["function"] = "refsource-offset";
        $this->store["missing"]["outer"][0]["id"]["label"] = "refsource-label";
        $this->store["missing"]["outer"][0]["plain"]["function"] = "plain-refsource-mutated";
        $this->store["missing"]["outer"][0]["plain"]["label"] = "plain-refsource-mutated";
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

$box = new Milestone1708_ThisPropertyOffsetReferenceSourceBox();
$alias =& $box->missing["outer"];
$alias[] = [
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
