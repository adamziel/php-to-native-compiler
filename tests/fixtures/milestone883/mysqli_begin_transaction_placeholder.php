<?php
$call = "mysqli_begin_transaction";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "\n";

$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_begin_transaction($handle) ? "default" : "failed";
echo "\n";
echo mysqli_begin_transaction($handle, 0, "wp") ? "named" : "failed";
echo "\n";
echo $call($handle, 0, null) ? "dynamic" : "failed";
