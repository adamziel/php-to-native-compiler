<?php
$name = "array_change_key_case";
$missing = "missing_native_function";

echo function_exists("array_change_key_case") ? "1" : "0";
echo function_exists("ARRAY_CHANGE_KEY_CASE") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo is_callable("array_change_key_case") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
