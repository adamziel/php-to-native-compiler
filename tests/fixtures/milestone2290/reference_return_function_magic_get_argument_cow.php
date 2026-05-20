<?php
function &pick($arr) {
    return $arr["ref"]["v"];
}

class Box {
    public $store;

    public function helper($name) {
        $copy = $this->store;
        $this->store["plain"] = "side";
        return $copy;
    }

    public function __get($name) {
        return $this->helper($name);
    }
}

$box = new Box();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$alias =& $box->store["ref"]["v"];
$result =& pick($box->missing);
echo $box->store["plain"], "\n";
$result = "updated";
echo $box->store["ref"]["v"];
