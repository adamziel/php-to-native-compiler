<?php
function milestone2295_magic_id($arr) {
    return $arr;
}

class Box {
    public $store;

    public function __get($name) {
        $copy = $this->store;
        return call_user_func_array(
            "milestone2295_magic_id",
            array_merge(array($copy))
        );
    }
}

$box = new Box();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$tmp = $box->missing;
$tmp["ref"]["v"] = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
