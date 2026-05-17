<?php
$target = __DIR__ . "/realpath_cache_stream_target.txt";
$cache_key = realpath($target);
clearstatcache(true);

echo function_exists("realpath_cache_get") ? "known" : "missing";
echo "|";
echo realpath_cache_size() === 0 ? "empty" : "not-empty";

$contents = file_get_contents($target);
$cache = realpath_cache_get();
echo "|";
echo str_contains($contents, "stream-cache") ? "read" : "missing-read";
echo "|";
echo array_key_exists($cache_key, $cache) ? "fgc-cached" : "fgc-missing";
echo "|";
echo realpath_cache_size() > 0 ? "fgc-sized" : "fgc-empty";

clearstatcache(true);
$handle = fopen($target, "r");
fclose($handle);
$cache = realpath_cache_get();
echo "|";
echo array_key_exists($cache_key, $cache) ? "fopen-cached" : "fopen-missing";
echo "|";
echo realpath_cache_size() > 0 ? "fopen-sized" : "fopen-empty";

clearstatcache(true);
echo "|";
echo realpath_cache_size() === 0 ? "cleared" : "still-sized";
