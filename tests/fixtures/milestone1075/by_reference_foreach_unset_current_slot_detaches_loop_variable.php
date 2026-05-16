<?php
$items = ["a" => 1, "b" => 2];

foreach ($items as $key => &$item) {
    echo $key, ":", $item, "|";
    if ($key === "a") {
        unset($items["a"]);
        $items["a"] = 10;
        echo "after=", $item, "|";
        $item = 11;
        echo "assigned=", $items["a"], "|";
    }
}
unset($item);
echo "\n";
foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
