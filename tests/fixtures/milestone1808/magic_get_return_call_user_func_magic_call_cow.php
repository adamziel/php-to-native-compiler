<?php
class Milestone1808Box {
    private $store = array();
    public $log = array();

    public function &__call($method, $args) {
        $this->log[] = "call:" . $method;
        return $this->store[$args[0]];
    }

    public function &__get($name) {
        $cb = array($this, "slot");
        $this->log[] = "get:" . $name;
        return call_user_func($cb, $name);
    }
}

$box = new Milestone1808Box();
$alias =& $box->missing["node"];
$alias["leaf"] = "call-user-func-magic-call";
$copy = $alias;
$copy["leaf"] = "call-user-func-magic-call-plain";

echo $box->missing["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $box->log);
