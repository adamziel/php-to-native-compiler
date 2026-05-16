<?php
$path = __DIR__ . "/writable_target.txt";

echo is_writable($path) ? "writable" : "not-writable";
echo "|";
echo is_writable(__DIR__ . "/missing-target.txt") ? "writable" : "missing";
echo "|";
$call = "is_writable";
echo function_exists($call) ? "exists" : "missing-function";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call($path) ? "repeat" : "changed";
