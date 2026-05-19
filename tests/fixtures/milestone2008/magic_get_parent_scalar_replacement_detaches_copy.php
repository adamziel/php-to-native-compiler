<?php
class Milestone2008_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $bucket["ref"] = "inside";
        return $bucket;
    }
}

$box = new Milestone2008_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
