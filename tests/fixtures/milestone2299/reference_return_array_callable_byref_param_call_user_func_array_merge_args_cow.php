<?php
error_reporting(0);

class Milestone2299_ByRefParamPicker {
    public function &pick(&$arr) {
        return $arr["ref"]["v"];
    }
}

class Milestone2299_ArrayCallableByRefParamBox {
    public $store;
    public $callback;

    public function &__get($name) {
        $copy = $this->store;
        return call_user_func_array(
            $this->callback,
            array_merge(array($copy))
        );
    }
}

$box = new Milestone2299_ArrayCallableByRefParamBox();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$box->callback = array(new Milestone2299_ByRefParamPicker(), "pick");
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
