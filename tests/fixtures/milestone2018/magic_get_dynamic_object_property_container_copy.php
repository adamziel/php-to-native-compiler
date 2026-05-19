<?php
class Milestone2018_Holder {
    public $bucket = array();
}

class Milestone2018_Box {
    public $store = array();

    public function __get($name) {
        $property = "bucket";
        $holder = new Milestone2018_Holder();
        $holder->{$property} = $this->store[$name];
        return $holder->{$property};
    }
}

$box = new Milestone2018_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
