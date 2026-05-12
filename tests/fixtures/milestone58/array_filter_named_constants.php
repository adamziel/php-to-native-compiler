<?php
function keep_named_key($key) {
    if ($key === "short") {
        return true;
    }
    if ($key === 5) {
        return true;
    }
    return false;
}

function keep_named_value_and_key($value, $key) {
    if ($key === "long") {
        return $value === "Grace";
    }
    if ($key === 6) {
        return $value === "tail";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "tail";

$key_mode = array_filter($items, "keep_named_key", ARRAY_FILTER_USE_KEY);
print_r(array_keys($key_mode));
echo count($key_mode), "|", $key_mode["short"], "|", $key_mode[5], "\n";

$both_mode = array_filter($items, "keep_named_value_and_key", ARRAY_FILTER_USE_BOTH);
print_r(array_keys($both_mode));
echo count($both_mode), "|", $both_mode["long"], "|", $both_mode[6], "\n";
echo ARRAY_FILTER_USE_KEY, "|", ARRAY_FILTER_USE_BOTH, "\n";

$call = "array_filter";
$again = $call($items, "keep_named_value_and_key", ARRAY_FILTER_USE_BOTH);
echo count($again), "|", $again[6];
