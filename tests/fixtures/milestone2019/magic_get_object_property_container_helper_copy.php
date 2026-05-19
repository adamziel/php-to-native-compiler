<?php
function milestone2019_pass($value) {
    return $value;
}

class Milestone2019_Holder {
    public $bucket = array();
}

class Milestone2019_Box {
    public $store = array();

    public function __get($name) {
        $holder = new Milestone2019_Holder();
        $holder->bucket = $this->store[$name];
        return milestone2019_pass($holder->bucket);
    }
}

$box = new Milestone2019_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
