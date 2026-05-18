<?php
class Milestone1733_Box {
    public $items = [];
}

$source = "seed";
$items = [];
$items["slot"] =& $source;
$alias =& $items["slot"];

$target = [];
$target["copy"] =& $alias;
$target["outer"]["copy"] =& $alias;

$box = new Milestone1733_Box();
$box->items["copy"] =& $alias;
$box->items["outer"][] =& $alias;

$source = "source-write";
$target["copy"] = "direct-target-write";
$target["outer"]["copy"] = "nested-target-write";
$box->items["copy"] = "property-target-write";
$box->items["outer"][0] = "property-append-write";

echo $source,
    "|",
    $items["slot"],
    "|",
    $alias,
    "|",
    $target["copy"],
    "|",
    $target["outer"]["copy"],
    "|",
    $box->items["copy"],
    "|",
    $box->items["outer"][0];
