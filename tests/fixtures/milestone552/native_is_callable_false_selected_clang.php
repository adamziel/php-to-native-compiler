<?php
$flag = true;
$syntax = true;

echo is_callable(null) ? "1" : "0";
echo is_callable(false) ? "1" : "0";
echo is_callable(42) ? "1" : "0";
echo is_callable(3.5) ? "1" : "0";
echo is_callable($flag) ? "1" : "0";
echo is_callable(42, true) ? "1" : "0";
echo is_callable(false, $syntax) ? "1" : "0";
echo "\n";
