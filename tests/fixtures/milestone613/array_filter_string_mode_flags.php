<?php
function keep_long($value) {
    return strlen($value) > 3;
}

function keep_selected_key($key) {
    if ($key === "long") {
        return true;
    }
    if ($key === 5) {
        return true;
    }
    return false;
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

$value_mode = array_filter($items, "keep_long", "0");
print_r(array_keys($value_mode));
echo count($value_mode), "|", $value_mode["long"], "|", $value_mode[5], "\n";

$both_mode = array_filter($items, "keep_value_and_key", " 1 ");
print_r(array_keys($both_mode));
echo count($both_mode), "|", $both_mode["long"], "|", $both_mode[5], "\n";

$call = "array_filter";
$key_mode = $call($items, "keep_selected_key", "02");
print_r(array_keys($key_mode));
echo count($key_mode), "|", $key_mode["long"], "|", $key_mode[5], "\n";

$null_mode = $call(["empty" => "", "zero" => "0", "space" => " "], null, "+1");
print_r(array_keys($null_mode));
echo count($null_mode), "|", strlen($null_mode["space"]), "\n";
