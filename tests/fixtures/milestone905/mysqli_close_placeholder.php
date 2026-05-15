<?php
$call = "mysqli_close";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($call) ? "yes" : "no";
echo "\n";
echo is_callable($call) ? "callable" : "missing";
echo "\n";
echo mysqli_close($dbh) ? "closed" : "open";
echo "\n";
echo $call($dbh) ? "closed" : "open";
