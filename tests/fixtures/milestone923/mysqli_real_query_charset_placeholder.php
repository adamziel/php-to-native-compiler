<?php
$real_query = "mysqli_real_query";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($real_query) ? "yes" : "no";
echo "\n";
echo is_callable($real_query) ? "callable" : "missing";
echo "\n";
echo mysqli_real_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "charset-ok" : "charset-failed";
echo "\n";
echo $real_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "dynamic" : "failed";
echo "\n";
echo mysqli_store_result($dbh) === false ? "no-pending" : "pending";
echo "\n";
echo mysqli_use_result($dbh) === false ? "no-use" : "using";
