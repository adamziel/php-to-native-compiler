<?php
error_reporting(0);

class Milestone2049_Helper {
    public function touch(&$value) {
        $value["ref"]["value"] = "inside";
        $value["plain"]["value"] = "inside-plain";
        return $value;
    }
}

class Milestone2049_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $helper = new Milestone2049_Helper();
        return call_user_func(array($helper, "touch"), $bucket);
    }
}

$box = new Milestone2049_Box();
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
