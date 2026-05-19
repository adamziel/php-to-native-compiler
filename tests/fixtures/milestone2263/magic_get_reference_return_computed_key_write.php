<?php
error_reporting(0);

function milestone2263_key($name) {
    return $name;
}

class Milestone2263_Box {
    public $store = array();

    public function &__get($name) {
        return $this->store[milestone2263_key($name)];
    }
}

$box = new Milestone2263_Box();
$box->store = array("slot" => array("value" => "orig"));

$box->slot["value"] = "inside";

echo $box->store["slot"]["value"];
