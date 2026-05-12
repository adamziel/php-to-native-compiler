<?php
$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["empty"] = "";
$items["zero"] = 0;
$items["false"] = false;
$items["2"] = "two";
$key = "present";

if (isset($items[$key])) {
    echo "present:set\n";
}
if (isset($items["null"])) {
    echo "null:set\n";
} else {
    echo "null:unset\n";
}
if (isset($items["missing"])) {
    echo "missing:set\n";
} else {
    echo "missing:unset\n";
}
if (isset($missing[0])) {
    echo "undefined:set\n";
} else {
    echo "undefined:unset\n";
}
$number = 42;
if (isset($number[0])) {
    echo "scalar:set\n";
} else {
    echo "scalar:unset\n";
}
$nullable = null;
if (isset($nullable[0])) {
    echo "nullable:set\n";
} else {
    echo "nullable:unset\n";
}
if (isset($items["present"], $items["empty"], $items["zero"], $items["false"])) {
    echo "multi:set\n";
}
if (isset($items["present"], $items["null"])) {
    echo "multi-null:set\n";
} else {
    echo "multi-null:unset\n";
}
if (isset($items[2])) {
    echo "int-normalized:set";
}
