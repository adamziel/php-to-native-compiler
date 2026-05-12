<?php
echo constant("ARRAY_FILTER_USE_KEY"), "|", constant("ARRAY_FILTER_USE_BOTH"), "\n";
$name = "ARRAY_FILTER_USE_KEY";
echo constant($name), "\n";
$call = "constant";
echo $call("ARRAY_FILTER_USE_BOTH"), "\n";

function keep_named_key($key) {
    return $key === "name";
}

$items = ["name" => "Ada", "other" => "Lin"];
$filtered = array_filter($items, "keep_named_key", constant("ARRAY_FILTER_USE_KEY"));
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["name"], "\n";
