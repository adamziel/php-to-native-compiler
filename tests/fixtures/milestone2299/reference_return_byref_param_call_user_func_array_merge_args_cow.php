<?php
error_reporting(0);

function &milestone2299_pick_ref_param(&$arr) {
    return $arr["ref"]["v"];
}

class Milestone2299_ByRefParamBox {
    public $store;

    public function &__get($name) {
        $copy = $this->store;
        return call_user_func_array(
            "milestone2299_pick_ref_param",
            array_merge(array($copy))
        );
    }
}

$box = new Milestone2299_ByRefParamBox();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
