<?php
error_reporting(0);

function milestone2262_key($name) {
    return $name;
}

class Milestone2262_Box {
    public $store = array();

    public function &__get($name) {
        return $this->store[milestone2262_key($name)];
    }
}

$box = new Milestone2262_Box();
$box->store = array("slot" => array("value" => "orig"));

$alias =& $box->slot["value"];
$alias = "inside";

echo $box->store["slot"]["value"];
