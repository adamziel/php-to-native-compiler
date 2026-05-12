<?php
function pick($label, $key) {
    echo "pick:", $label, "\n";
    return $key;
}

$items = ["name" => "Ada", "city" => "Paris", "role" => "dev"];
$target = "live";

unset($items[pick("first", "name")], $target, $items[pick("second", "city")], $missing);

echo "count:", count($items), "\n";
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
if (isset($target)) {
    echo "target:set\n";
} else {
    echo "target:unset\n";
}
if (isset($missing)) {
    echo "missing:set";
} else {
    echo "missing:unset";
}
