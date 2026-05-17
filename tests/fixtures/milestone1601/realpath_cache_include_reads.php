<?php
$target = realpath("tests/fixtures/milestone1601/realpath_cache_include_target.inc");
$cache_key = $target;
clearstatcache(true);

echo function_exists("realpath_cache_get") ? "known" : "missing";
echo "|";
echo realpath_cache_size() === 0 ? "empty" : "not-empty";
echo "|";

include $target;

$cache = realpath_cache_get();
echo "|";
echo array_key_exists($cache_key, $cache) ? "include-cached" : "include-missing";
echo "|";
echo realpath_cache_size() > 0 ? "include-sized" : "include-empty";

clearstatcache(true);
echo "|";
echo realpath_cache_size() === 0 ? "cleared" : "still-sized";
