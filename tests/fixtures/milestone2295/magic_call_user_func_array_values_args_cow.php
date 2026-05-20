<?php
function milestone2295_magic_values_id($arr) {
    return $arr;
}

class BoxValues {
    public $store;

    public function __get($name) {
        $copy = $this->store;
        return call_user_func_array(
            "milestone2295_magic_values_id",
            array_values(array($copy))
        );
    }
}

$box = new BoxValues();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$tmp = $box->missing;
$tmp["ref"]["v"] = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
