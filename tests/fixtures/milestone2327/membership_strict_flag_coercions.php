<?php
$items = [0, "0", false, true, "x"];

function show_keys($keys, $newline = true) {
    echo count($keys), ":";
    foreach ($keys as $key) {
        echo "[", $key, "]";
    }
    if ($newline) {
        echo "\n";
    }
}

show_keys(array_keys($items, "0", 1));
show_keys(array_keys($items, "0", "0"));
show_keys(array_keys($items, false, "yes"));
show_keys(array_keys($items, false, ""));

var_dump(array_search("0", $items, 2));
var_dump(array_search("0", $items, ""));
var_dump(array_search(false, $items, "yes"));
var_dump(array_search(false, $items, 0));

var_dump(in_array("0", [0, false, true, "x"], 1));
var_dump(in_array("0", [0, false, true, "x"], "0"));

$search = "array_search";
var_dump($search("0", $items, "yes"));

$contains = "in_array";
var_dump($contains("0", [0, false], "yes"));

$keys = "array_keys";
show_keys($keys($items, "0", "yes"), false);
