<?php
function &milestone2300_pick_keep($arr) {
    return $arr["ref"]["v"];
}

class Milestone2300_Box {
    public $store;

    public function __get($name) {
        return $this->store[$name];
    }
}

$box = new Milestone2300_Box();
$box->store = array(
    "keep" => array("plain" => "left", "ref" => array("v" => "leaf")),
    "changed" => array("plain" => "right", "ref" => array("v" => "old")),
);
$alias =& $box->store["keep"]["ref"]["v"];
$args = array("keep" => $box->keep, "changed" => $box->changed);
$changed =& $args["changed"];
$changed = array("plain" => "replaced", "ref" => array("v" => "new"));
$slot =& call_user_func_array("milestone2300_pick_keep", array($args["keep"]));
$slot = "updated";
echo $box->store["keep"]["plain"], "\n";
echo $box->store["keep"]["ref"]["v"], "\n";
echo $args["changed"]["plain"];
