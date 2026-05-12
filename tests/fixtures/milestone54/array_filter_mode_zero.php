<?php
function keep_long($value) {
    return strlen($value) > 3;
}

$items = [];
$items["empty"] = "";
$items["zero"] = "0";
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "";

$null_mode = array_filter($items, null, 0);
print_r(array_keys($null_mode));
echo count($null_mode), "|", $null_mode["short"], "|", $null_mode["long"], "|", $null_mode[5], "\n";

$callback_mode = array_filter($items, "keep_long", 0);
print_r(array_keys($callback_mode));
echo count($callback_mode), "|", $callback_mode["long"], "|", $callback_mode[5], "\n";

$call = "array_filter";
$builtin = $call(["empty" => "", "zero" => "0", "space" => " "], "strlen", 0);
print_r(array_keys($builtin));
echo count($builtin), "|", $builtin["zero"], "|", strlen($builtin["space"]), "\n";

$again = $call($items, null, 0);
echo count($again), "\n";
