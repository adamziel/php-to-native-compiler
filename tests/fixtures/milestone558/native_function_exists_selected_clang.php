<?php
$known = "strlen";
$missing = "missing_native_function";

echo function_exists("strlen") ? "1" : "0";
echo function_exists("STRLEN") ? "1" : "0";
echo function_exists("function_exists") ? "1" : "0";
echo function_exists("missing_native_function") ? "1" : "0";
echo function_exists($known) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
