<?php
$items = [1, 2];

foreach ($items as $key => &$item) {
    echo $key, ":", $item, "|";
    if ($item === 1) {
        $items[] = 3;
    }
}
unset($item);
echo "\n";
foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
echo "\n";

$named = ["a" => 1, "b" => 2];
foreach ($named as $key => &$value) {
    echo "before=", $value, "|";
    if ($key === "a") {
        $named["a"] = 10;
        echo "after-direct=", $value, "|";
        $named["c"] = 3;
    }
}
unset($value);
echo "\n";
foreach ($named as $key => $value) {
    echo $key, ":", $value, "|";
}
