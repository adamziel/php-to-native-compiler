<?php
error_reporting(0);

function &milestone2215_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2215_ArgsHolder {
    public $args = array();
}

class Milestone2215_Box {
    public $store = array();
    public $holder;
    public $calls = 0;

    public function &holderRef($copy) {
        $this->calls = $this->calls + 1;
        unset($this->holder->args);
        $this->holder->args = array($copy);
        return $this->holder;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder->args = array();
        $alias =& call_user_func_array(
            "milestone2215_pick_ref",
            $this->holderRef($bucket)->args
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2215_Box();
$box->holder = new Milestone2215_ArgsHolder();
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
