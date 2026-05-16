<?php
$dump = "mysqli_dump_debug_info";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($dump) ? "yes" : "no";
echo "\n";
echo is_callable($dump) ? "callable" : "missing";
echo "\n";
echo mysqli_dump_debug_info($dbh) ? "dumped" : "failed";
echo "\n";
echo $dump($dbh) ? "dynamic" : "failed";
echo "\n";
echo mysqli_ping($dbh) ? "still-open" : "closed";
