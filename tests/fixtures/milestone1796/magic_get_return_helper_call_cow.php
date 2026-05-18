<?php
function &milestone1796_identity(&$value) {
    return $value;
}

class Milestone1796_MagicBox {
    public $store = [];
    public $log = [];

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        return milestone1796_identity($this->store[$name]);
    }
}

$source = "helper-call-seed";
$node = ["value" => &$source, "plain" => ["value" => "helper-call-copy"]];

$box = new Milestone1796_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "helper-call";
$box->missing["node"]["plain"]["value"] = "helper-call-plain";

echo $source,
    "|",
    $box->store["missing"]["node"]["plain"]["value"],
    "|",
    $box->log[0],
    "|",
    $box->log[1],
    "|",
    $box->log[2];
