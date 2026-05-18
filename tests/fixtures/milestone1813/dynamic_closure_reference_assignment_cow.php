<?php
$items = array("slot" => "start");
$fn = function &(&$items, $key) {
    echo "closure:", $key, "|";
    return $items[$key];
};

$alias =& $fn($items, "slot");
$alias = "changed";
echo "dynamic-closure|", $items["slot"];
