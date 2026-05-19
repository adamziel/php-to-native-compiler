<?php
class Milestone2043_Box {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

$box = new Milestone2043_Box();
$box->store = array(
    "slot" => array(
        "child" => array(
            "ref" => array("value" => "original"),
            "plain" => array("value" => "plain-original"),
        ),
    ),
);
$alias =& $box->store["slot"]["child"]["ref"]["value"];

$copy = $box->slot["child"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["child"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["child"]["plain"]["value"], "|",
    $copy["plain"]["value"];
