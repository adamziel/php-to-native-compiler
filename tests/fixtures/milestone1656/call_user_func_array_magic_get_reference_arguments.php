<?php
$storage = array("slot" => "initial", "return" => array("leaf" => "seed"), "named" => "name");

class Milestone1656_Magic_Get_Callback_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

function milestone1656_mark_magic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

function &milestone1656_pick_magic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$box = new Milestone1656_Magic_Get_Callback_Box();
echo call_user_func_array("milestone1656_mark_magic", array(&$box->missing["slot"], "normal")), "|", $storage["slot"], "\n";

$alias =& call_user_func_array("milestone1656_pick_magic", array(&$box->missing["return"]["leaf"], "return"));
$alias = $alias . ":alias";
echo $storage["return"]["leaf"], "|", $alias, "\n";

echo call_user_func_array("milestone1656_mark_magic", array("suffix" => "named", "value" => &$box->missing["named"])), "|", $storage["named"];
