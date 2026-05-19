<?php
error_reporting(0);

function &milestone2212_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2212_ArgsHolder {
    public $args = array();
}

class Milestone2212_Box {
    public $store = array();
    public $holder;
    public $calls = 0;
    public $argProperty = "args";

    public function holder(&$copy) {
        $this->calls = $this->calls + 1;
        $property = $this->argProperty;
        unset($this->holder->$property);
        $this->holder->$property = array($copy);
        $copy = array(
            "ref" => array("value" => "local-replacement"),
            "plain" => array("value" => "local-replacement"),
        );
        return $this->holder;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $property = $this->argProperty;
        $this->holder->$property = array();
        $alias =& call_user_func_array(
            "milestone2212_pick_ref",
            $this->holder($bucket)->$property
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2212_Box();
$box->holder = new Milestone2212_ArgsHolder();
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
