<?php
$wp_refcow_magic_append_store = [];
$wp_refcow_magic_dynamic_append_store = [];

class WP_RefCow_Magic_Array_Append_Reference_Box {
    public function &__get($property) {
        echo "get:$property\n";
        global $wp_refcow_magic_append_store;
        return $wp_refcow_magic_append_store;
    }
}

class WP_RefCow_Magic_Dynamic_Array_Append_Reference_Box {
    public function &__get($property) {
        echo "get:$property\n";
        global $wp_refcow_magic_dynamic_append_store;
        return $wp_refcow_magic_dynamic_append_store;
    }
}

$box = new WP_RefCow_Magic_Array_Append_Reference_Box();
$alias =& $box->missing[];
$alias = "from-alias";
echo $wp_refcow_magic_append_store[0], "|";
$wp_refcow_magic_append_store[0] = "from-store";
echo $alias, "\n";

$property = "dynamicMissing";
$dynamic_box = new WP_RefCow_Magic_Dynamic_Array_Append_Reference_Box();
$dynamic =& $dynamic_box->{$property}[];
$dynamic = "from-dynamic";
echo $wp_refcow_magic_dynamic_append_store[0], "|";
$wp_refcow_magic_dynamic_append_store[0] = "from-dynamic-store";
echo $dynamic;
