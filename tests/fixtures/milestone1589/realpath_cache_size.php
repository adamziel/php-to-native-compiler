<?php
clearstatcache(true);
$target = "tests/fixtures/milestone1589/realpath_cache_size_target.txt";
$call = "realpath_cache_size";

echo function_exists($call) ? "known" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call() === 0 ? "empty" : "not-empty";

$resolved = realpath($target);
$size = realpath_cache_size();

echo "|";
echo $resolved === false ? "unresolved" : "resolved";
echo "|";
echo is_int($size) ? "int" : "other";
echo "|";
echo $size > 0 ? "positive" : "zero";

clearstatcache(true);
echo "|";
echo realpath_cache_size() === 0 ? "cleared" : "still-sized";
