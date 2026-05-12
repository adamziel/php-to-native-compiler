<?php
function keep_value_and_key($value, $key) {
    if ($key === "short") {
        return $value === "Ada";
    }
    if ($key === 5) {
        return $value === "Linus";
    }
    if ($key === "02") {
        return $value === "zero-two";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "tail";
$items["02"] = "zero-two";
$items["other"] = "Ada";

$filtered = array_filter($items, "keep_value_and_key", 1);
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["short"], "|", $filtered[5], "|", $filtered["02"], "\n";
$filtered[] = "after";
echo $filtered[6], "\n";

$call = "array_filter";
$again = $call($items, "keep_value_and_key", 1);
echo count($again), "|", $again["02"], "\n";

$null_mode = $call(["empty" => "", "zero" => "0", "space" => " "], null, 1);
print_r(array_keys($null_mode));
echo count($null_mode), "|", strlen($null_mode["space"]), "\n";
