<?php
$name = "array_filter";
$upper = "ARRAY_FILTER";
$missing = "missing_array_filter";

echo function_exists("array_filter") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo function_exists($upper) ? "1" : "0";
echo is_callable("array_filter") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo is_callable($upper, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
