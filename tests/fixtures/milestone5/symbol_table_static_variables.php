<?php
$name = "Ada";
$label = $name . "-static";
$items = [];
$items["label"] = $label;
echo isset($name), "\n";
echo $items["label"], "\n";
function shadow($name = "local") {
    $name = $name . "-scope";
    return $name;
}
echo shadow(), "\n";
echo $name, "\n";
