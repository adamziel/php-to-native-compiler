<?php
$items = ["name" => "Ada", 2 => "two"];
$items[] = "next";
echo $items["name"], "\n";
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
unset($items[2]);
echo count(array_values($items));
