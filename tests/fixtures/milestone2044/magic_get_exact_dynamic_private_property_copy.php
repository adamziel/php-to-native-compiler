<?php
class Milestone2044_Box {
    private $slot = array();

    public function seed($value) {
        $this->slot = $value;
    }

    public function &alias() {
        return $this->slot["ref"]["value"];
    }

    public function __get($name) {
        return $this->{$name};
    }
}

$box = new Milestone2044_Box();
$box->seed(array(
    "ref" => array("value" => "original"),
    "plain" => array("value" => "plain-original"),
));
$alias =& $box->alias();

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $copy["ref"]["value"], "|", $copy["plain"]["value"];
