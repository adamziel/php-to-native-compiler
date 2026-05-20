<?php
function pass_with_side_effect(&$sink, $arr) {
    $sink = "touched";
    return $arr;
}

class Box {
    public $store;
    public $sink;

    public function __get($name) {
        $copy = $this->store;
        $this->store["plain"] = "side";
        return $copy;
    }
}

$box = new Box();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$args = array(&$box->sink, $box->missing);
$tmp = call_user_func_array("pass_with_side_effect", $args);
$tmp["ref"]["v"] = "updated";
echo $box->sink, "\n";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
