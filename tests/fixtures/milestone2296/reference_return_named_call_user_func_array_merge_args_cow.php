<?php
function &milestone2296_named_pick_ref($arr) {
    return $arr["ref"]["v"];
}

class BoxReferenceReturnNamed {
    public $store;

    public function &__get($name) {
        $copy = $this->store;
        return call_user_func_array(
            "milestone2296_named_pick_ref",
            array_merge(array("arr" => $copy))
        );
    }
}

$box = new BoxReferenceReturnNamed();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
