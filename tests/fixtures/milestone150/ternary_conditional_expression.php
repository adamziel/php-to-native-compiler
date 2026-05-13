<?php
function trace($name, $value) {
    echo "call:", $name, "\n";
    return $value;
}

function choose($value) {
    return $value ? "truthy" : "falsey";
}

echo "truthiness\n";
foreach ([null, false, true, 0, 1, 0.0, 0.5, "", "0", "php", [], [1]] as $value) {
    echo choose($value), "\n";
}

echo "lazy\n";
echo (trace("condition", true) ? trace("true", "T") : trace("false", "F")), "\n";
echo (trace("condition", false) ? trace("true-missing", $missing) : trace("false", "F")), "\n";

echo "nested\n";
echo (true ? (false ? "bad" : "inner-false") : "outer-false"), "\n";
echo ((false ? "bad" : true) ? "outer-true" : "outer-false"), "\n";

echo "values\n";
$items = ["a" => 10, "b" => 20];
$key = false ? "a" : "b";
echo $items[$key], "\n";
$assigned = true ? ($slot = "assigned") : ($slot = "wrong");
echo $assigned, ":", $slot, "\n";
echo strlen(false ? "no" : "four"), "\n";
