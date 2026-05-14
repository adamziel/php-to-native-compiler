<?php
$name = "array_column";
$missing = "missing_native_function";

echo function_exists("array_column") ? "1" : "0";
echo function_exists("ARRAY_COLUMN") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo is_callable("array_column") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
