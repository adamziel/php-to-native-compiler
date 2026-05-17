<?php
$target = "tests/fixtures/milestone1583/realpath_cache_filename_target.txt";
$resolved_target = realpath($target);
$resolved_source = realpath(__FILE__);

echo array_key_exists($resolved_target, realpath_cache_get()) ? "target-cached" : "target-missing";
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "source-cached" : "source-missing";

clearstatcache(true, $resolved_target);
echo "|";
echo array_key_exists($resolved_target, realpath_cache_get()) ? "target-kept" : "target-cleared";
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "source-kept" : "source-cleared";

clearstatcache(false, $resolved_source);
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "stat-kept" : "stat-cleared";

clearstatcache(true, "");
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "empty-kept" : "empty-cleared";

clearstatcache(true);
echo "|";
echo array_key_exists($resolved_source, realpath_cache_get()) ? "all-kept" : "all-cleared";
