<?php
$items = [];
echo ($items[] = "first"), ":", $items[0], "\n";
echo ($items[] = 42), ":", $items[1], "\n";

echo ($created[] = "made"), ":", $created[0], "\n";
$nullable = null;
echo ($nullable[] = "null-made"), ":", $nullable[0], "\n";

function rhs_value() {
    echo "rhs\n";
    return "value";
}

echo ($items[] = rhs_value()), ":", $items[2];
