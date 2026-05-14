<?php
$name = "array_product";
$upper = "ARRAY_PRODUCT";
$missing = "missing_array_product";

echo function_exists("array_product") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo function_exists($upper) ? "1" : "0";
echo is_callable("array_product") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo is_callable($upper, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
