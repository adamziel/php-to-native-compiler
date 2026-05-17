<?php
$size = filesize(__FILE__);
echo $size > 0 ? "size-ok" : "size-bad";
echo "|";
$call = "filesize";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(__FILE__) === $size ? "repeat" : "different";
