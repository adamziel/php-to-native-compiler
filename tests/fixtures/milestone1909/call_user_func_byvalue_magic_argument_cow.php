<?php
function milestone1909_mutate($array) {
    $array["leaf"] = "changed";
    $array["plain"]["value"] = "copy-changed";
}

class Milestone1909_Box {
    public $store = array();
    public $trace = array();

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        return $this->store[$name];
    }
}

$box = new Milestone1909_Box();
$box->store["missing"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $box->store["missing"]["leaf"];

call_user_func("milestone1909_mutate", $box->missing);

echo $alias, "|", $box->store["missing"]["leaf"], "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
