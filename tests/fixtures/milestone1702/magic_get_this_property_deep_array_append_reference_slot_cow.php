<?php
class Milestone1702_ThisPropertyMagicDeepArrayBox {
    public $store = ["outer" => ["inner" => []]];

    public function &__get($name) {
        return $this->store;
    }

    public function bucket() {
        return $this->store["outer"]["inner"][0];
    }
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$box = new Milestone1702_ThisPropertyMagicDeepArrayBox();
$box->missing["outer"]["inner"][] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$box->store["outer"]["inner"][0]["id"]["function"] = "via-this-property-deep-magic-append";
$box->store["outer"]["inner"][0]["id"]["label"] = "via-this-property-deep-magic-label";
$box->store["outer"]["inner"][0]["id"]["accepted_args"] = 6;
$box->store["outer"]["inner"][0]["plain"]["function"] = "plain-deep-mutated";
$box->store["outer"]["inner"][0]["plain"]["label"] = "plain-deep-mutated";

$again = $box->bucket();
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
