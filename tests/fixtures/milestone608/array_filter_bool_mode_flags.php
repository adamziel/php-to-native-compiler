<?php
function keep_long($value) {
    return strlen($value) > 3;
}

function keep_value_and_key($value, $key) {
    if ($key === "long") {
        return $value === "Grace";
    }
    if ($key === 5) {
        return $value === "Linus";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "";

$false_mode = array_filter($items, "keep_long", false);
print_r(array_keys($false_mode));
echo count($false_mode), "|", $false_mode["long"], "|", $false_mode[5], "\n";

$true_mode = array_filter($items, "keep_value_and_key", true);
print_r(array_keys($true_mode));
echo count($true_mode), "|", $true_mode["long"], "|", $true_mode[5], "\n";

$call = "array_filter";
$null_true = $call(["empty" => "", "zero" => "0", "space" => " "], null, true);
print_r(array_keys($null_true));
echo count($null_true), "|", strlen($null_true["space"]), "\n";

$again = $call($items, "keep_long", false);
echo count($again), "|", $again["long"];
