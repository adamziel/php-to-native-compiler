<?php
$storage = array("slot" => "initial", "return" => array("leaf" => "seed"), "named" => "name");

class Milestone1658_Magic_Get_Stored_Callback_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

function milestone1658_mark_magic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

function &milestone1658_pick_magic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$box = new Milestone1658_Magic_Get_Stored_Callback_Box();
$args = array(&$box->missing["slot"], "normal");
echo call_user_func_array("milestone1658_mark_magic", $args), "|", $storage["slot"], "|", $args[0], "\n";

$returnArgs = array(&$box->missing["return"]["leaf"], "return");
$alias =& call_user_func_array("milestone1658_pick_magic", $returnArgs);
$alias = $alias . ":alias";
echo $storage["return"]["leaf"], "|", $returnArgs[0], "|", $alias, "\n";

$named = array("suffix" => "named", "value" => &$box->missing["named"]);
echo call_user_func_array("milestone1658_mark_magic", $named), "|", $storage["named"], "|", $named["value"];
