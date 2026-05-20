<?php
function &milestone2302_call_user_func_pick_ref($arr) {
    return $arr["ref"]["v"];
}

class Milestone2302_CallUserFuncStringBox {
    public $store;

    public function &__get($name) {
        $copy = $this->store;
        return call_user_func("milestone2302_call_user_func_pick_ref", $copy);
    }
}

$box = new Milestone2302_CallUserFuncStringBox();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
