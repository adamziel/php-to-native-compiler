<?php
class Milestone2024_Holder {
    public $bucket = array();
}

class Milestone2024_Box {
    public $store = array();

    private function touchBucket(&$value) {
        $value["ref"]["value"] = "inside";
        $value["plain"]["value"] = "plain-inside";
    }

    public function __get($name) {
        $holder = new Milestone2024_Holder();
        $holder->bucket = $this->store[$name];
        $this->touchBucket($holder->bucket);
        return $holder->bucket;
    }
}

$box = new Milestone2024_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $alias, "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
