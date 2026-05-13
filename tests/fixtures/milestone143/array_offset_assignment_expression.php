<?php
$items = [];
echo ($items["name"] = "Ada"), ":", $items["name"], "\n";
echo ($items[2] = 99), ":", $items[2], "\n";

function key_name() {
    echo "key\n";
    return "slot";
}
function rhs_value() {
    echo "rhs\n";
    return "value";
}

echo ($items[key_name()] = rhs_value()), ":", $items["slot"], "\n";
echo ($created["first"] = "made"), ":", $created["first"], "\n";
$nullable = null;
echo ($nullable["first"] = "null-made"), ":", $nullable["first"];
