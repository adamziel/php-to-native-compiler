<?php
$wp_refcow_secret = "secret";
$wp_refcow_dynamic = "dynamic";
$wp_refcow_data = ["slot" => "array"];

class WP_RefCow_Inaccessible_Magic_Secret_Box {
    private $secret = "declared";

    public function &__get($name) {
        global $wp_refcow_secret;
        return $wp_refcow_secret;
    }
}

class WP_RefCow_Inaccessible_Magic_Dynamic_Box {
    protected $dynamic_secret = "declared";

    public function &__get($name) {
        global $wp_refcow_dynamic;
        return $wp_refcow_dynamic;
    }
}

class WP_RefCow_Inaccessible_Magic_Array_Box {
    private $data = [];

    public function &__get($name) {
        global $wp_refcow_data;
        return $wp_refcow_data;
    }
}

function wp_refcow_touch_inaccessible_magic(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new WP_RefCow_Inaccessible_Magic_Secret_Box();
wp_refcow_touch_inaccessible_magic($box->secret, "private");
echo $wp_refcow_secret, "\n";

$property = "dynamic_secret";
$dynamic_box = new WP_RefCow_Inaccessible_Magic_Dynamic_Box();
wp_refcow_touch_inaccessible_magic($dynamic_box->{$property}, "dynamic");
echo $wp_refcow_dynamic, "\n";

$array_box = new WP_RefCow_Inaccessible_Magic_Array_Box();
wp_refcow_touch_inaccessible_magic($array_box->data["slot"], "slot");
echo $wp_refcow_data["slot"], "\n";

$alias =& $box->secret;
$alias = "alias";
echo $wp_refcow_secret, "|";
$wp_refcow_secret = "global";
echo $alias;
