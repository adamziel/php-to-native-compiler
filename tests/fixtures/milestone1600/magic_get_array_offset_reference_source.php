<?php
$wp_refcow_magic_store = [
    "slot" => "initial",
    "nested" => [
        "leaf" => "nested-initial",
    ],
];
$wp_refcow_magic_dynamic_store = [
    "slot" => "dynamic-initial",
];

class WP_RefCow_Magic_Array_Reference_Box {
    public function &__get($property) {
        echo "get:$property\n";
        global $wp_refcow_magic_store;
        return $wp_refcow_magic_store;
    }
}

class WP_RefCow_Magic_Dynamic_Array_Reference_Box {
    public function &__get($property) {
        echo "get:$property\n";
        global $wp_refcow_magic_dynamic_store;
        return $wp_refcow_magic_dynamic_store;
    }
}

$box = new WP_RefCow_Magic_Array_Reference_Box();
$alias =& $box->missing["slot"];
$alias = "from-alias";
echo $wp_refcow_magic_store["slot"], "|";
$wp_refcow_magic_store["slot"] = "from-store";
echo $alias, "\n";

$nested =& $box->missing["nested"]["leaf"];
$nested = "from-nested";
echo $wp_refcow_magic_store["nested"]["leaf"], "|";
$wp_refcow_magic_store["nested"]["leaf"] = "from-store-nested";
echo $nested, "\n";

$property = "dynamicMissing";
$dynamic_box = new WP_RefCow_Magic_Dynamic_Array_Reference_Box();
$dynamic =& $dynamic_box->{$property}["slot"];
$dynamic = "from-dynamic";
echo $wp_refcow_magic_dynamic_store["slot"], "|";
$wp_refcow_magic_dynamic_store["slot"] = "from-dynamic-store";
echo $dynamic;
