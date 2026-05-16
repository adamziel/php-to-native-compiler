<?php
$items = ["a", ["nested" => "b"]];

foreach ($items as $key => &$item) {
    if (is_array($item)) {
        $item["seen"] = $key;
    } else {
        $item = $item . "!";
    }
}
unset($item);

echo $items[0], "|", $items[1]["nested"], "|", $items[1]["seen"];
