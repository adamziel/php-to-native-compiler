<?php
$name = "array_reduce";
$upper = "ARRAY_REDUCE";
$missing = "missing_array_reduce";

echo function_exists("array_reduce") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo function_exists($upper) ? "1" : "0";
echo is_callable("array_reduce") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo is_callable($upper, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
