<?php
$mtime = filemtime(__FILE__);
echo is_int($mtime) && $mtime > 0 ? "mtime-ok" : "mtime-bad";
echo "|";
$call = "filemtime";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(__FILE__) === $mtime ? "repeat" : "different";
echo "|";
echo @filemtime(__DIR__ . "/missing-file.php") === false ? "missing-false" : "missing-time";
