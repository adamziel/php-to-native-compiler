<?php
class Box {
    public $store;

    public function __get($name) {
        $copy = $this->store;
        $this->store["plain"] = "side";
        return $copy;
    }
}

$box = new Box();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$tmp = $box->missing;
$tmp["ref"]["v"] = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
