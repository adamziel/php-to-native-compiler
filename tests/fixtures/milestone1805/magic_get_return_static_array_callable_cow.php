<?php
class Milestone1805Helper {
    public static function &pick($box, $name) {
        return $box->store[$name];
    }
}

class Milestone1805Box {
    public $store = array();
    public $log = array();

    public function &__get($name) {
        $cb = array("Milestone1805Helper", "pick");
        $this->log[] = "get:" . $name;
        return call_user_func($cb, $this, $name);
    }
}

$box = new Milestone1805Box();
$alias =& $box->missing["node"];
$alias["leaf"] = "static-array-callable";
$copy = $alias;
$copy["leaf"] = "static-array-callable-plain";

echo $box->missing["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $box->log);
