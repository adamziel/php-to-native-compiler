<?php
class Milestone1731_Box {
    public $items = [];
}

$direct = "direct-seed";
$nested = "nested-seed";
$property = "property-seed";
$propertyAppend = "property-append-seed";
$dynamicProperty = "dynamic-property-seed";
$dynamicPropertyAppend = "dynamic-property-append-seed";

$items = [];
$items["outer"]["slot"] =& $direct;
$items["outer"][] =& $nested;

$box = new Milestone1731_Box();
$box->items["slot"] =& $property;
$box->items["outer"][] =& $propertyAppend;
$propertyName = "items";
$box->{$propertyName}["dynamic"] =& $dynamicProperty;
$box->{$propertyName}["dynamicOuter"][] =& $dynamicPropertyAppend;

$direct = "direct-variable";
$nested = "nested-variable";
$property = "property-variable";
$propertyAppend = "property-append-variable";
$dynamicProperty = "dynamic-property-variable";
$dynamicPropertyAppend = "dynamic-property-append-variable";

$items["outer"]["slot"] = "direct-slot";
$items["outer"][0] = "nested-slot";
$box->items["slot"] = "property-slot";
$box->items["outer"][0] = "property-append-slot";
$box->items["dynamic"] = "dynamic-property-slot";
$box->items["dynamicOuter"][0] = "dynamic-property-append-slot";

echo $direct,
    "|",
    $nested,
    "|",
    $property,
    "|",
    $propertyAppend,
    "|",
    $dynamicProperty,
    "|",
    $dynamicPropertyAppend,
    "|",
    $items["outer"]["slot"],
    "|",
    $items["outer"][0],
    "|",
    $box->items["slot"],
    "|",
    $box->items["outer"][0],
    "|",
    $box->items["dynamic"],
    "|",
    $box->items["dynamicOuter"][0];
