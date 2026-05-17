<?php
$storage = ["slot" => "initial", "nested" => ["leaf" => "inside"]];
$dynamicStorage = ["slot" => "dynamic"];

class WP_RefCow_Magic_Get_Reference_Return_Call_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class WP_RefCow_Magic_Get_Reference_Return_Call_Dynamic_Box {
    public function &__get($name) {
        global $dynamicStorage;
        return $dynamicStorage;
    }
}

class WP_RefCow_Magic_Get_Reference_Return_Call_Picker {
    public function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function &wp_refcow_magic_get_reference_return_touch(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$box = new WP_RefCow_Magic_Get_Reference_Return_Call_Box();
wp_refcow_magic_get_reference_return_touch($box->missing["slot"], "function");

$picker = new WP_RefCow_Magic_Get_Reference_Return_Call_Picker();
$picker->touch($box->missing["nested"]["leaf"], "method");

$dynamicBox = new WP_RefCow_Magic_Get_Reference_Return_Call_Dynamic_Box();
$property = "dynamic";
wp_refcow_magic_get_reference_return_touch($dynamicBox->{$property}["slot"], "dynamic");

echo $storage["slot"], "\n", $storage["nested"]["leaf"], "\n", $dynamicStorage["slot"];
