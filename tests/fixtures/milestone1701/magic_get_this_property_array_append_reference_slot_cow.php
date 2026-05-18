<?php
class Milestone1701_ThisPropertyMagicArrayBox {
    public $store = ["outer" => []];

    public function &__get($name) {
        return $this->store;
    }

    public function bucket() {
        return $this->store["outer"][0];
    }

}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$box = new Milestone1701_ThisPropertyMagicArrayBox();
$box->missing["outer"][] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$box->store["outer"][0]["id"]["function"] = "via-this-property-magic-append";
$box->store["outer"][0]["id"]["label"] = "via-this-property-magic-label";
$box->store["outer"][0]["id"]["accepted_args"] = 5;
$box->store["outer"][0]["plain"]["function"] = "plain-mutated";
$box->store["outer"][0]["plain"]["label"] = "plain-mutated";

$again = $box->bucket();
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
