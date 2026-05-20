<?php
class BoxReferenceReturnClosure {
    public $store;

    public function &__get($name) {
        $copy = $this->store;
        $pick = function &($arr) {
            return $arr["ref"]["v"];
        };
        return call_user_func_array(
            $pick,
            array_values(array($copy))
        );
    }
}

$box = new BoxReferenceReturnClosure();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$slot =& $box->missing;
$slot = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
