<?php
$source = str_contains(getcwd(), "/compiler") ? "../tests/fixtures/milestone1313/filemtime_local_metadata.php" : "tests/fixtures/milestone1313/filemtime_local_metadata.php";
$mtime = filemtime($source);
echo is_int($mtime) && $mtime > 0 ? "mtime-ok" : "mtime-bad";
echo "|";
$call = "filemtime";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call($source) === $mtime ? "repeat" : "different";
echo "|";
echo @filemtime(dirname($source) . "/missing-file.php") === false ? "missing-false" : "missing-time";
