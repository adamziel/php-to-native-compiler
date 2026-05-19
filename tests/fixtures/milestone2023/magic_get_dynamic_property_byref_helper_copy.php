<?php
function milestone2023_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "plain-inside";
}

class Milestone2023_Holder {
    public $bucket = array();
}

class Milestone2023_Box {
    public $store = array();

    public function __get($name) {
        $property = "bucket";
        $holder = new Milestone2023_Holder();
        $holder->{$property} = $this->store[$name];
        milestone2023_touch_bucket($holder->{$property});
        return $holder->{$property};
    }
}

$box = new Milestone2023_Box();
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
