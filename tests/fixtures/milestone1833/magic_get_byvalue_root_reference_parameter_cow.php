<?php
error_reporting(0);

function milestone1833_mutate(&$value) {
    $value["leaf"] = "changed";
    return $value["leaf"];
}

class Milestone1833Box {
    public $items = array("x" => array("leaf" => "v"));
    public $hits = 0;

    public function __get($name) {
        $this->hits++;
        return $this->items[$name];
    }
}

$box = new Milestone1833Box();
$result = milestone1833_mutate($box->x);
$read = $box->x["leaf"];

echo $box->items["x"]["leaf"], "|", $result, "|", $read, "|hits=", $box->hits;
