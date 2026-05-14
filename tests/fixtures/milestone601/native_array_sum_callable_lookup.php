<?php
$name = "array_sum";
$upper = "ARRAY_SUM";
$missing = "missing_array_sum";

echo function_exists("array_sum") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo function_exists($upper) ? "1" : "0";
echo is_callable("array_sum") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo is_callable($upper, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
