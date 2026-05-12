<?php
class Box {}

$box = new box();
$values = [
    null,
    false,
    7,
    3.5,
    "x",
    ["nested"],
    $box,
];

foreach ($values as $value) {
    echo get_debug_type($value), "\n";
}

$call = "get_debug_type";
echo $call($box), "\n";
