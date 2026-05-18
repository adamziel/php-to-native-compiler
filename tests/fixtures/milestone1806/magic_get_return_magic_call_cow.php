<?php
class Milestone1806Box {
    private $store = array();
    public $log = array();

    public function &__call($method, $args) {
        $this->log[] = "call:" . $method;
        return $this->store[$args[0]];
    }

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        return $this->slot($name);
    }
}

$box = new Milestone1806Box();
$alias =& $box->missing["node"];
$alias["leaf"] = "magic-call";
$copy = $alias;
$copy["leaf"] = "magic-call-plain";

echo $box->missing["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $box->log);
