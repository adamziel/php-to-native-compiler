<?php
error_reporting(0);

function &milestone2231_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2231_Box {
    public $store = array();
    public $calls = 0;

    public function prepare($copy, &$args) {
        $this->calls = $this->calls + 1;
        $args = array($copy);
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $args = array();
        $this->prepare($bucket, $args);
        $alias =& call_user_func_array("milestone2231_pick_ref", $args);
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2231_Box();
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
    "|", $box->calls, ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls;
