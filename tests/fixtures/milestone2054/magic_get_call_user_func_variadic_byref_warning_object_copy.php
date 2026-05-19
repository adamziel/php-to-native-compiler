<?php
error_reporting(0);

class Milestone2054_Helper {
    public function touch(&...$values) {
        $values[0]["ref"]["value"] = "inside";
        $values[0]["plain"]["value"] = "inside-plain";
        return $values[0];
    }
}

class Milestone2054_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $helper = new Milestone2054_Helper();
        return call_user_func(array($helper, "touch"), $bucket);
    }
}

$box = new Milestone2054_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
