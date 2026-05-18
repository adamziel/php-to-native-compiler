<?php
class Milestone1794_MagicBox {
    public $store = [];
    public $log = [];

    public function &storeRef() {
        $this->log[] = "storeRef";
        return $this->store;
    }

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        return $this->storeRef();
    }
}

$source = "method-call-seed";
$node = ["value" => &$source, "plain" => ["value" => "method-call-copy"]];

$box = new Milestone1794_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "method-call";
$box->missing["node"]["plain"]["value"] = "method-call-plain";

echo $source,
    "|",
    $box->store["node"]["plain"]["value"],
    "|",
    $box->log[0],
    "|",
    $box->log[1],
    "|",
    $box->log[2],
    "|",
    $box->log[3],
    "|",
    $box->log[4],
    "|",
    $box->log[5];
