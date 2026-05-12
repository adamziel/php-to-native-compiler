<?php
$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["2"] = "two";
$items["02"] = "zero two";
$key = "present";

if (array_key_exists($key, $items)) {
    echo "present:exists\n";
}
if (array_key_exists("null", $items)) {
    echo "null:exists\n";
}
if (isset($items["null"])) {
    echo "null:isset\n";
} else {
    echo "null:not-set\n";
}
if (array_key_exists("missing", $items)) {
    echo "missing:exists\n";
} else {
    echo "missing:absent\n";
}
if (array_key_exists(2, $items)) {
    echo "int-normalized:exists\n";
}
if (array_key_exists("02", $items)) {
    echo "leading-zero-string:exists\n";
}
$exists = "array_key_exists";
if ($exists("present", $items)) {
    echo "dynamic:exists";
}
