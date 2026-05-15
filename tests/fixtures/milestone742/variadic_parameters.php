<?php
function collect($first, ...$items) {
    echo $first, ":", count($items), ":", $items[0], ":", $items[1], "\n";
}

collect("a", "b", "c");

function empty_rest(...$items) {
    echo count($items), "\n";
}

empty_rest();

function optional_rest($first = "x", ...$items) {
    echo $first, ":", count($items);
}

optional_rest();
echo "\n";
optional_rest("y", "z");
