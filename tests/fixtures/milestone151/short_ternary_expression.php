<?php
function trace($name, $value) {
    echo "call:", $name, "\n";
    return $value;
}

echo "truthiness\n";
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php"] as $value) {
    var_dump($value ?: "fallback");
}

echo "lazy\n";
echo (trace("truthy", "kept") ?: trace("truthy-fallback", "bad")), "\n";
echo (trace("falsey", "") ?: trace("falsey-fallback", "fallback")), "\n";

echo "contexts\n";
$items = ["fallback" => 10, "php" => 20];
$key = "php" ?: "fallback";
echo $items[$key], "\n";
echo strlen("" ?: "four"), "\n";
$assigned = ($slot = "assigned") ?: "wrong";
echo $assigned, ":", $slot, "\n";
$fallback = ($empty = "") ?: ($empty = "filled");
echo $fallback, ":", $empty;
