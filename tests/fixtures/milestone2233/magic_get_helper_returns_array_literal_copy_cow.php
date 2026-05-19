<?php
error_reporting(0);

function milestone2233_wrap($copy) {
    return array($copy);
}

class Milestone2233_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $args = milestone2233_wrap($bucket);
        return $args[0];
    }
}

$box = new Milestone2233_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
