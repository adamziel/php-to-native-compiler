<?php
$items = [];
$items["name"] = "Ada";
$items["city"] = "Paris";
$items["2"] = "two";
$items[] = "next-before";

unset($items["name"]);
unset($items["missing"]);
unset($items[2]);
$items[] = "next-after";

echo "count:", count($items), "\n";
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
if (isset($items["name"])) {
    echo "name:set\n";
} else {
    echo "name:unset\n";
}
if (array_key_exists(2, $items)) {
    echo "two:set\n";
} else {
    echo "two:unset\n";
}
if (array_key_exists(4, $items)) {
    echo "append:4\n";
} else {
    echo "append:other\n";
}
