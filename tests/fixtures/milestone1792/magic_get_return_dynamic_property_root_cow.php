<?php
class Milestone1792_MagicBox {
    private $missing = [];
    public $log = [];

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        return $this->{$name};
    }

    public function readPlain($key) {
        return $this->missing[$key]["plain"]["value"];
    }
}

$source = "dynamic-root-seed";
$node = ["value" => &$source, "plain" => ["value" => "dynamic-root-copy"]];

$box = new Milestone1792_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "dynamic-root";
$box->missing["node"]["plain"]["value"] = "dynamic-root-plain";

echo $source,
    "|",
    $box->readPlain("node"),
    "|",
    $box->log[0],
    "|",
    $box->log[1],
    "|",
    $box->log[2];
