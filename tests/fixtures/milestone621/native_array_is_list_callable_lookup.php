<?php
$name = "array_is_list";
$upper = "ARRAY_IS_LIST";
$missing = "missing_array_is_list";

echo function_exists("array_is_list") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo function_exists($upper) ? "1" : "0";
echo is_callable("array_is_list") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo is_callable($upper, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
