<?php
function greet($name = "world", $suffix = "!") {
    echo "hello ", $name, $suffix, "\n";
}

greet();
greet("Ada");
greet("Ada", ".");

function scale($value, $factor = 2, $offset = 1) {
    return $value * $factor + $offset;
}

echo scale(3), "\n";
echo scale(3, 4), "\n";
echo scale(3, 4, 5), "\n";

function defaults($number = 1 + 2, $text = "a" . "b", $flag = !false) {
    if ($flag) {
        echo $number, ":", $text, "\n";
    }
}

defaults();

function default_items($items = ["first", "second" => 2]) {
    echo count($items), ":", $items[0], ":", $items["second"], "\n";
}

default_items();
