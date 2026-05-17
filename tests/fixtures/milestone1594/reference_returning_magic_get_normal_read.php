<?php
$wp_refcow_magic_slot = "initial";
$wp_refcow_magic_dynamic_slot = "dynamic-initial";

class WP_RefCow_Reference_Returning_Magic_Read_Box {
    private $secret = "declared";

    public function &__get($property) {
        echo "get:$property\n";
        global $wp_refcow_magic_slot;
        return $wp_refcow_magic_slot;
    }
}

class WP_RefCow_Reference_Returning_Magic_Dynamic_Read_Box {
    protected $dynamicSecret = "declared";

    public function &__get($property) {
        echo "get:$property\n";
        global $wp_refcow_magic_dynamic_slot;
        return $wp_refcow_magic_dynamic_slot;
    }
}

$box = new WP_RefCow_Reference_Returning_Magic_Read_Box();
$copy = $box->secret;
$wp_refcow_magic_slot = "changed";
echo $copy, "|", $box->secret, "\n";

$property = "dynamicSecret";
$dynamic_box = new WP_RefCow_Reference_Returning_Magic_Dynamic_Read_Box();
$dynamic_copy = $dynamic_box->{$property};
$wp_refcow_magic_dynamic_slot = "dynamic-changed";
echo $dynamic_copy, "|", $dynamic_box->{$property};
