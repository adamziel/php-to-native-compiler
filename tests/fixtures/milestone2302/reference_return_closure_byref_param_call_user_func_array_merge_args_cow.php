<?php
error_reporting(0);

class Milestone2302_ClosureByRefParamBox {
    public $store;

    public function &__get($name) {
        $copy = $this->store;
        $pick = function &(&$arr) {
            return $arr["ref"]["v"];
        };
        return call_user_func_array(
            $pick,
            array_merge(array($copy))
        );
    }
}

$box = new Milestone2302_ClosureByRefParamBox();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
