<?php
function &milestone1799_store($box) {
    return $box->store;
}

class Milestone1799_MagicBox {
    public $store = [];
    public $log = [];

    public function &__get($name) {
        $fn = "milestone1799_store";
        $this->log[] = "get:" . $name;
        return $fn($this);
    }
}

$source = "dynamic-helper-root-seed";
$node = ["value" => &$source, "plain" => ["value" => "dynamic-helper-root-copy"]];

$box = new Milestone1799_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "dynamic-helper-root";
$box->missing["node"]["plain"]["value"] = "dynamic-helper-root-plain";

echo $source,
    "|",
    $box->store["node"]["plain"]["value"],
    "|",
    $box->log[0],
    "|",
    $box->log[1],
    "|",
    $box->log[2];
