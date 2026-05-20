<?php
class Box {
    public $store;

    public function __get($name) {
        $copy = $this->store;
        $wrapped = array_values(array($copy));
        return $wrapped[0];
    }
}

$box = new Box();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$tmp = $box->missing;
$tmp["ref"]["v"] = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
