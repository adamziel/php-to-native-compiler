<?php
error_reporting(0);

function milestone2200_identity($value) {
    return $value;
}

class Milestone2200_ArgsHolder {
    public $args = array();
}

class Milestone2200_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $holder = new Milestone2200_ArgsHolder();
        $holder->args = array($bucket);
        return call_user_func_array("milestone2200_identity", $holder->args);
    }
}

$box = new Milestone2200_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
