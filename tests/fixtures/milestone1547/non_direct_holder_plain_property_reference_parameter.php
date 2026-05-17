<?php
class WP_RefCow_Reference_Parameter_Plain_Property_Bag {
    public $items = ["outer" => ["slot" => "seed"]];
    public $dynamicItems = ["outer" => ["slot" => "dynamic"]];
}

function wp_refcow_plain_property_touch_slot(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$holders = [];
$primary = new WP_RefCow_Reference_Parameter_Plain_Property_Bag();
$holders["primary"] = $primary;
wp_refcow_plain_property_touch_slot($holders["primary"]->items["outer"]["slot"], "named");
echo $primary->items["outer"]["slot"], "\n";

$dynamic = new WP_RefCow_Reference_Parameter_Plain_Property_Bag();
$holders["dynamic"] = $dynamic;
$property = "dynamicItems";
wp_refcow_plain_property_touch_slot($holders["dynamic"]->{$property}["outer"]["slot"], "selected");
echo $dynamic->dynamicItems["outer"]["slot"];
