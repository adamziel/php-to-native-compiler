<?php
class Milestone1791_MagicBox {
    public $store = [];
    public $log = [];

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        return $this->store;
    }
}

$source = "property-root-seed";
$node = ["value" => &$source, "plain" => ["value" => "property-root-copy"]];

$box = new Milestone1791_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "property-root";
$box->missing["node"]["plain"]["value"] = "property-root-plain";

echo $source,
    "|",
    $box->store["node"]["plain"]["value"],
    "|",
    $box->log[0],
    "|",
    $box->log[1],
    "|",
    $box->log[2];
