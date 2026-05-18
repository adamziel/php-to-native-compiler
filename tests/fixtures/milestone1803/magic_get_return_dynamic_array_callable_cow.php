<?php
class Milestone1803Box {
    private $store = array();
    public $log = array();

    public function &pick($name) {
        return $this->store[$name];
    }

    public function &__get($name) {
        $cb = array($this, "pick");
        $this->log[] = "get:" . $name;
        return $cb($name);
    }
}

$box = new Milestone1803Box();
$alias =& $box->missing["node"];
$alias["leaf"] = "dynamic-array-callable";
$copy = $alias;
$copy["leaf"] = "dynamic-array-callable-plain";

echo $box->missing["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $box->log);
