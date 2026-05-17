<?php
echo function_exists("clearstatcache") ? "known" : "missing";
echo "|";
echo is_callable("clearstatcache") ? "callable" : "not-callable";
echo "|";
$call = "clearstatcache";
$first = clearstatcache();
$second = $call(true, __FILE__);
echo $first === null ? "first-null" : "first-value";
echo "|";
echo $second === null ? "second-null" : "second-value";
echo "|";
echo file_exists(__FILE__) ? "exists" : "missing";
echo "|";
echo is_int(filemtime(__FILE__)) ? "mtime-int" : "mtime-false";
echo "|";
echo filesize(__FILE__) > 0 ? "size-positive" : "size-empty";
