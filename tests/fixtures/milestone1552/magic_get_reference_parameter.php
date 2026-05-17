<?php
$storage = "initial";
$dynamicStorage = "dynamic";

class WP_RefCow_Magic_Get_Reference_Parameter_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class WP_RefCow_Magic_Get_Reference_Parameter_Dynamic_Box {
    public function &__get($name) {
        global $dynamicStorage;
        return $dynamicStorage;
    }
}

function wp_refcow_magic_get_touch(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$box = new WP_RefCow_Magic_Get_Reference_Parameter_Box();
wp_refcow_magic_get_touch($box->missing, "plain");
echo $storage, "\n";

$dynamicBox = new WP_RefCow_Magic_Get_Reference_Parameter_Dynamic_Box();
$property = "dynamic";
wp_refcow_magic_get_touch($dynamicBox->{$property}, "selected");
echo $dynamicStorage;
