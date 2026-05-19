<?php
function milestone1910_mutate($array, $label) {
    $array["leaf"] = $label;
    $array["plain"]["value"] = "copy-" . $label;
}

class Milestone1910_Box {
    public $store = array();
    public $trace = array();

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        return $this->store[$name];
    }
}

$box = new Milestone1910_Box();
$box->store["literal"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$literalAlias =& $box->store["literal"]["leaf"];

call_user_func_array("milestone1910_mutate", array($box->literal, "literal"));

$box->store["stored"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$storedAlias =& $box->store["stored"]["leaf"];
$args = array($box->stored, "stored");
call_user_func_array("milestone1910_mutate", $args);

echo $literalAlias, "|", $box->store["literal"]["plain"]["value"], "|", $storedAlias, "|", $box->store["stored"]["plain"]["value"], "|", implode(",", $box->trace);
