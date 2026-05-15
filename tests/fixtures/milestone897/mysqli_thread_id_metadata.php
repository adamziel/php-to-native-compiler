<?php
$call = "mysqli_thread_id";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($call) ? "yes" : "no";
echo "\n";
echo is_callable($call) ? "callable" : "missing";
echo "\n";
echo mysqli_thread_id($dbh);
echo "\n";
echo $call($dbh);
