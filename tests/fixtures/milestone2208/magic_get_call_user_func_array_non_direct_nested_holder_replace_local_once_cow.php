<?php
error_reporting(0);

function &milestone2208_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2208_ArgsHolder {
    public $sets = array();
}

class Milestone2208_Box {
    public $store = array();
    public $holder;
    public $calls = 0;

    public function holder(&$copy) {
        $this->calls = $this->calls + 1;
        $copy = array(
            "ref" => array("value" => "local-replacement"),
            "plain" => array("value" => "local-replacement"),
        );
        return $this->holder;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder->sets["pack"] = array($bucket);
        $alias =& call_user_func_array(
            "milestone2208_pick_ref",
            $this->holder($bucket)->sets["pack"]
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2208_Box();
$box->holder = new Milestone2208_ArgsHolder();
$box->holder->sets = array("pack" => array());
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
