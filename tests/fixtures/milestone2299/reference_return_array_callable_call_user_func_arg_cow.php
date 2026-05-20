<?php
class Milestone2299_CallUserFuncPicker {
    public function &pick($arr) {
        return $arr["ref"]["v"];
    }
}

class Milestone2299_CallUserFuncBox {
    public $store;
    public $callback;

    public function &__get($name) {
        $copy = $this->store;
        return call_user_func($this->callback, $copy);
    }
}

$box = new Milestone2299_CallUserFuncBox();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$box->callback = array(new Milestone2299_CallUserFuncPicker(), "pick");
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
