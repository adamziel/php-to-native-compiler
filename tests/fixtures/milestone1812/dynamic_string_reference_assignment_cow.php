<?php
function &milestone1812_pick(&$items, $key) {
    echo "pick:", $key, "|";
    return $items[$key];
}

$items = array("slot" => "start");
$fn = "milestone1812_pick";
$alias =& $fn($items, "slot");
$alias = "changed";
echo "dynamic-string|", $items["slot"];
