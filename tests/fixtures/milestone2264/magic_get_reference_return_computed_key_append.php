<?php
error_reporting(0);

function milestone2264_key($name) {
    return $name;
}

class Milestone2264_Box {
    public $store = array();

    public function &__get($name) {
        return $this->store[milestone2264_key($name)];
    }
}

$box = new Milestone2264_Box();
$box->store = array("slot" => array("a"));

$box->slot[] = "b";

echo $box->store["slot"][0], "|", $box->store["slot"][1];
