<?php
error_reporting(0);

class Milestone2265_Box {
    public $store = array();

    public function &__get($name) {
        return $this->store["fixed"];
    }
}

$box = new Milestone2265_Box();
$box->store = array(
    "slot" => array("value" => "slot"),
    "fixed" => array("value" => "orig"),
);

$box->slot["value"] = "inside";

echo $box->store["slot"]["value"], "|", $box->store["fixed"]["value"];
