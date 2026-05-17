<?php
$storage = ["slot" => "initial", "nested" => ["leaf" => "inside"]];
$dynamicStorage = [];

class WP_RefCow_Magic_Get_Array_Reference_Parameter_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class WP_RefCow_Magic_Get_Array_Reference_Parameter_Dynamic_Box {
    public function &__get($name) {
        global $dynamicStorage;
        return $dynamicStorage;
    }
}

function wp_refcow_magic_get_touch_slot(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new WP_RefCow_Magic_Get_Array_Reference_Parameter_Box();
wp_refcow_magic_get_touch_slot($box->missing["slot"], "plain");
wp_refcow_magic_get_touch_slot($box->missing["nested"]["leaf"], "nested");
echo $storage["slot"], "\n", $storage["nested"]["leaf"], "\n";

$dynamicBox = new WP_RefCow_Magic_Get_Array_Reference_Parameter_Dynamic_Box();
$property = "dynamic";
wp_refcow_magic_get_touch_slot($dynamicBox->{$property}["created"], "selected");
echo $dynamicStorage["created"];
