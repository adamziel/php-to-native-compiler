<?php
$call = "mysqli_get_host_info";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "\n";

$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_get_host_info($handle);
echo "\n";
echo $call($handle);
