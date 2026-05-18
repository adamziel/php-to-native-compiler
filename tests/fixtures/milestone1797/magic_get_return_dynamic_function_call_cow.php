<?php
function &milestone1797_pick(&$value) {
    return $value;
}

class Milestone1797_MagicBox {
    public $store = [];
    public $log = [];

    public function &__get($name) {
        $fn = "milestone1797_pick";
        $this->log[] = "get:" . $name;
        return $fn($this->store[$name]);
    }
}

$source = "dynamic-function-seed";
$node = ["value" => &$source, "plain" => ["value" => "dynamic-function-copy"]];

$box = new Milestone1797_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "dynamic-function";
$box->missing["node"]["plain"]["value"] = "dynamic-function-plain";

echo $source,
    "|",
    $box->store["missing"]["node"]["plain"]["value"],
    "|",
    $box->log[0],
    "|",
    $box->log[1],
    "|",
    $box->log[2];
