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
$tmp = pass_with_side_effect($box->sink, $box->missing);
$tmp["ref"]["v"] = "updated";
echo $box->sink, "\n";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
