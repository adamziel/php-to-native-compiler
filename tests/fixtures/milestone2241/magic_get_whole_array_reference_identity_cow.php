<?php
error_reporting(0);

function milestone2241_wrap(&$value) {
    return array(&$value);
}

class Milestone2241_Box {
    public $store = array();

    public function __get($name) {
        $args = milestone2241_wrap($this->store[$name]);
        $args[0]["plain"]["value"] = "inside";
        return $this->store[$name];
    }
}

$box = new Milestone2241_Box();
$box->store = array(
    "slot" => array(
        "plain" => array("value" => "plain-original"),
    ),
);

$copy = $box->slot;
echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["plain"]["value"] = "copy";

echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
