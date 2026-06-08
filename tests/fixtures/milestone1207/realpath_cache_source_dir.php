<?php
$cache = realpath_cache_get();
$dir = __DIR__;
if (!str_starts_with($dir, "/")) {
    $dir = getcwd() . "/" . $dir;
}
echo array_key_exists($dir, $cache) ? "dir-cached" : "dir-missing";
echo "|";
echo $cache[$dir]["is_dir"] === true ? "dir" : "not-dir";
echo "|";
clearstatcache(true, $dir);
echo array_key_exists($dir, realpath_cache_get()) ? "kept" : "cleared";
