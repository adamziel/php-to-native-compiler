<?php
$call = "mysqli_autocommit";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "\n";

$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_autocommit($handle, false) ? "off" : "failed";
echo "\n";
echo mysqli_autocommit($handle, true) ? "on" : "failed";
echo "\n";
echo $call($handle, false) ? "dynamic" : "failed";
