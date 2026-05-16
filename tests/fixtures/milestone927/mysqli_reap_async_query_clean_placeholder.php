<?php
$reap = "mysqli_reap_async_query";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($reap) ? "yes" : "no";
echo "\n";
echo is_callable($reap) ? "callable" : "missing";
echo "\n";
echo mysqli_reap_async_query($dbh) === false ? "no-async" : "async";
echo "\n";
echo $reap($dbh) === false ? "dynamic" : "async";
echo "\n";
echo mysqli_ping($dbh) ? "still-open" : "closed";
