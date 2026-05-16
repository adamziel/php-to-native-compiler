<?php
$debug = "mysqli_debug";

echo function_exists($debug) ? "yes" : "no";
echo "\n";
echo is_callable($debug) ? "callable" : "missing";
echo "\n";
echo mysqli_debug("d:t:o,/tmp/phpc-mysqli-debug.trace") ? "debug" : "failed";
echo "\n";
echo $debug(null) ? "dynamic" : "failed";
