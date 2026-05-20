<?php
class Milestone2302_ValueClosureBox {
    public $store;

    public function __get($name) {
        $copy = $this->store;
        $pick = function ($arr) {
            return $arr["ref"];
        };
        return call_user_func_array(
            $pick,
            array_merge(array($copy))
        );
    }
}

$box = new Milestone2302_ValueClosureBox();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$slot = $box->missing;
$slot["v"] = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
