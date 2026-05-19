<?php
error_reporting(0);

class Milestone2089_Box {
    public $store = array();

    public function __get($name) {
        $bucket = array(
            "ref" => &$this->store[$name]["ref"],
            "plain" => array("value" => "plain-original"),
        );
        return $bucket;
    }
}

$box = new Milestone2089_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
    ),
);
$leaf =& $box->store["slot"]["ref"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo $leaf["value"], "|", $copy["plain"]["value"];
