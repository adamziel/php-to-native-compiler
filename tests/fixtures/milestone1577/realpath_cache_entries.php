<?php
$target = "tests/fixtures/milestone1577/realpath_cache_target.txt";
$resolved = realpath($target);
$cache = realpath_cache_get();

echo $resolved === false ? "unresolved" : "resolved";
echo "|";
echo array_key_exists($resolved, $cache) ? "cached" : "missing";

$entry = $cache[$resolved];
echo "|";
echo $entry["realpath"] === $resolved ? "same" : "different";
echo "|";
echo $entry["is_dir"] === false ? "file" : "dir";
echo "|";
echo is_int($entry["expires"]) ? "expires-int" : "expires-other";

clearstatcache(false);
echo "|";
echo array_key_exists($resolved, realpath_cache_get()) ? "kept" : "cleared";

clearstatcache(true);
echo "|";
echo array_key_exists($resolved, realpath_cache_get()) ? "kept" : "cleared";
