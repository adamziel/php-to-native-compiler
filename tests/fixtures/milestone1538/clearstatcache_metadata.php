<?php
$source = str_contains(getcwd(), "/compiler") ? "../tests/fixtures/milestone1538/clearstatcache_metadata.php" : "tests/fixtures/milestone1538/clearstatcache_metadata.php";
echo function_exists("clearstatcache") ? "known" : "missing";
echo "|";
echo is_callable("clearstatcache") ? "callable" : "not-callable";
echo "|";
$call = "clearstatcache";
$first = clearstatcache();
$second = $call(true, $source);
echo $first === null ? "first-null" : "first-value";
echo "|";
echo $second === null ? "second-null" : "second-value";
echo "|";
echo file_exists($source) ? "exists" : "missing";
echo "|";
echo is_int(filemtime($source)) ? "mtime-int" : "mtime-false";
echo "|";
echo filesize($source) > 0 ? "size-positive" : "size-empty";
