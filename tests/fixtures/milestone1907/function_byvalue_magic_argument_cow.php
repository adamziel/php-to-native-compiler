<?php
function milestone1907_mutate($array) {
    $array["leaf"] = "changed";
    $array["plain"]["value"] = "copy-changed";
}

class Milestone1907_Box {
    public $store = array();
    public $trace = array();

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        return $this->store[$name];
    }
}

$box = new Milestone1907_Box();
$box->store["missing"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $box->store["missing"]["leaf"];

milestone1907_mutate($box->missing);

echo $alias, "|", $box->store["missing"]["leaf"], "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
