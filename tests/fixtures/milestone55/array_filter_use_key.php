<?php
function keep_selected_key($key) {
    if ($key === "short") {
        return true;
    }
    if ($key === 5) {
        return true;
    }
    if ($key === "02") {
        return true;
    }
    return false;
}

$items = [];
$items[""] = "empty-key";
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "tail";
$items["02"] = "zero-two";

$filtered = array_filter($items, "keep_selected_key", 2);
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["short"], "|", $filtered[5], "|", $filtered["02"], "\n";
$filtered[] = "after";
echo $filtered[6], "\n";

$call = "array_filter";
$builtin = $call(["" => "empty", "name" => "Ada", "long-key" => "Grace"], "strlen", 2);
print_r(array_keys($builtin));
echo count($builtin), "|", $builtin["name"], "|", $builtin["long-key"], "\n";

$again = $call($items, "keep_selected_key", 2);
echo count($again), "|", $again[5];
