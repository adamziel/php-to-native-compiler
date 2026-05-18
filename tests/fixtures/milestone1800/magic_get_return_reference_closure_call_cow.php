<?php
class Milestone1800Box {
    private $store = array();
    public $log = array();

    public function &__get($name) {
        $fn = function &(&$value) {
            return $value;
        };
        $this->log[] = "get:" . $name;
        return $fn($this->store[$name]);
    }
}

$box = new Milestone1800Box();
$alias =& $box->missing["node"];
$alias["leaf"] = "reference-closure";
$copy = $alias;
$copy["leaf"] = "reference-closure-plain";

echo $box->missing["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $box->log);
