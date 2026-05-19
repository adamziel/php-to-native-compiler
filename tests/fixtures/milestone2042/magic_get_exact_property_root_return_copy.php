<?php
class Milestone2042_Box {
    public $store = array();

    public function __get($name) {
        return $this->store;
    }
}

$box = new Milestone2042_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->ignored;
$copy["slot"]["ref"]["value"] = "copy";
$copy["slot"]["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["slot"]["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["slot"]["plain"]["value"];
