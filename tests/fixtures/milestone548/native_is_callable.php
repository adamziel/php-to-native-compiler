<?php
$known = "strlen";
$missing = "missing_native_function";
$syntax = true;

echo is_callable("strlen") ? "1" : "0";
echo is_callable("STRLEN") ? "1" : "0";
echo is_callable("assert") ? "1" : "0";
echo is_callable("ASSERT") ? "1" : "0";
echo is_callable("missing_native_function") ? "1" : "0";
echo is_callable("missing_native_function", true) ? "1" : "0";
echo is_callable("assert", false) ? "1" : "0";
echo is_callable("strlen", false) ? "1" : "0";
$known = "assert";
echo is_callable($known) ? "1" : "0";
echo is_callable($missing) ? "1" : "0";
echo is_callable($missing, $syntax) ? "1" : "0";
echo "\n";
