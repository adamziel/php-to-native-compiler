<?php
$assigned = 1;
$nullable = null;
$falsey = false;
$text = "value";

echo isset($assigned) ? "1" : "0";
echo isset($nullable) ? "1" : "0";
echo isset($missing) ? "1" : "0";
echo isset($falsey) ? "1" : "0";
echo isset($text) ? "1" : "0";
echo "\n";
