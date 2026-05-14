<?php
$items = [];
$items[-1] = "minus";
$items[0] = "zero";
$items[1] = "one";
$items[2] = "two";

if (array_key_exists(1.0, $items)) {
    echo "one:exists\n";
}
if (array_key_exists(2.0, $items)) {
    echo "two:exists\n";
}
if (array_key_exists(-1.0, $items)) {
    echo "minus:exists\n";
}

$call = "array_key_exists";
if ($call(0.0, $items)) {
    echo "dynamic:zero";
}
