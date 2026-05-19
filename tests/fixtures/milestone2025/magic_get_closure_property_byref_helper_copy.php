<?php
class Milestone2025_Holder {
    public $bucket = array();
}

class Milestone2025_Box {
    public $store = array();

    public function __get($name) {
        $touchBucket = function (&$value) {
            $value["ref"]["value"] = "inside";
            $value["plain"]["value"] = "plain-inside";
        };
        $holder = new Milestone2025_Holder();
        $holder->bucket = $this->store[$name];
        $touchBucket($holder->bucket);
        return $holder->bucket;
    }
}

$box = new Milestone2025_Box();
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
