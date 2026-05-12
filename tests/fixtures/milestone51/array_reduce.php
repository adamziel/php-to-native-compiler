<?php
function join_values($carry, $value) {
    if ($carry === null) {
        return $value;
    }
    return $carry . ":" . $value;
}

function collect_values($carry, $value) {
    if (!$carry) {
        $carry = [];
    }
    $carry[] = $value;
    return $carry;
}

function sum_pair($carry, $value) {
    if ($carry === null) {
        $carry = 0;
    }
    return $carry + $value;
}

$items = [];
$items["first"] = "Ada";
$items[5] = "Grace";
$items[] = "Linus";

$callback = "join_values";
echo array_reduce($items, $callback), "\n";

$collected = array_reduce($items, "collect_values");
print_r($collected);

if (array_reduce([], "join_values") === null) {
    echo "empty-null\n";
}

print_r($items);

$call = "array_reduce";
echo $call([1, 2, 3], "sum_pair");
