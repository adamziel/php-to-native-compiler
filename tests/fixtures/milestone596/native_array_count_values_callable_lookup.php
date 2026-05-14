<?php
$name = "array_count_values";
$upper = "ARRAY_COUNT_VALUES";
$missing = "missing_array_count_values";

echo function_exists("array_count_values") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo function_exists($upper) ? "1" : "0";
echo is_callable("array_count_values") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo is_callable($upper, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
