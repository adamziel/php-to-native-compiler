<?php
class Milestone1801Box {
    public $store = array();
    public $log = array();

    public function &__get($name) {
        $fn = function &($box, $slot) {
            return $box->store[$slot];
        };
        $this->log[] = "get:" . $name;
        return call_user_func($fn, $this, $name);
    }
}

$box = new Milestone1801Box();
$alias =& $box->missing["node"];
$alias["leaf"] = "call-user-func-closure";
$copy = $alias;
$copy["leaf"] = "call-user-func-closure-plain";

echo $box->missing["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $box->log);
