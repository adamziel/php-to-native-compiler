<?php
$path = "/tmp/phpc_milestone1543_stat_cache.txt";
$alias = "/tmp/phpc_milestone1543_stat_cache_alias.txt";

if (file_exists($path)) {
    unlink($path);
}
if (file_exists($alias)) {
    unlink($alias);
}

$h = fopen($path, "w");
fwrite($h, "abc");
fclose($h);
link($path, $alias);

$first = filesize($path);

$h = fopen($alias, "w");
fwrite($h, "abcdef");
fclose($h);

$cached = filesize($path);
clearstatcache(false, $path);
$cleared = filesize($path);

$h = fopen($alias, "w");
fwrite($h, "abcdefghi");
fclose($h);

$cached_again = filesize($path);
clearstatcache();
$cleared_all = filesize($path);

echo $first;
echo "|";
echo $cached;
echo "|";
echo $cleared;
echo "|";
echo $cached_again;
echo "|";
echo $cleared_all;

unlink($alias);
unlink($path);
