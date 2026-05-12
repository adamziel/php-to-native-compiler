<?php
function keep_long($value) {
    return strlen($value) > 3;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items["empty"] = "";
$items[5] = "Linus";

$callback = "keep_long";
$filtered = array_filter($items, $callback);
print_r(array_keys($filtered));
echo $filtered["long"], "|", $filtered[5], "\n";
$filtered[] = "after";
echo $filtered[6], "\n";

$call = "array_filter";
$builtin = $call(["empty" => "", "zero" => "0", "space" => " "], "strlen");
print_r(array_keys($builtin));
echo count($builtin), "|", $builtin["zero"], "|", strlen($builtin["space"]);
