<?php
function join_with_seed($carry, $value) {
    return $carry . ":" . $value;
}

function collect_with_seed($carry, $value) {
    $carry[] = $value;
    return $carry;
}

function add_value($carry, $value) {
    return $carry + $value;
}

$items = ["Ada", "Grace", "Linus"];
echo array_reduce($items, "join_with_seed", "start"), "\n";

$collected = array_reduce($items, "collect_with_seed", ["seed"]);
print_r($collected);

if (array_reduce([], "join_with_seed", "empty") === "empty") {
    echo "empty-initial\n";
}

$call = "array_reduce";
echo $call([1, 2, 3], "add_value", 10);
