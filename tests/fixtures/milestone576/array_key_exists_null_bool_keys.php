<?php
$items = [];
$items[""] = "empty";
$items[0] = "zero";
$items[1] = "one";
$items["01"] = "string-one";

if (array_key_exists(null, $items)) {
    echo "null:exists\n";
}
if (array_key_exists(false, $items)) {
    echo "false:exists\n";
}
if (array_key_exists(true, $items)) {
    echo "true:exists\n";
}
if (array_key_exists("01", $items)) {
    echo "string-one:exists\n";
}

$call = "array_key_exists";
if ($call(false, $items)) {
    echo "dynamic:false";
}
