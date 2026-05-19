<?php
error_reporting(0);

function &milestone2204_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2204_ArgsHolder {
    public $sets = array();
}

class Milestone2204_Box {
    public $store = array();
    public $holder;

    public function holder() {
        return $this->holder;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder()->sets["pack"] = array($bucket);
        $alias =& call_user_func_array(
            "milestone2204_pick_ref",
            $this->holder()->sets["pack"]
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2204_Box();
$box->holder = new Milestone2204_ArgsHolder();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
