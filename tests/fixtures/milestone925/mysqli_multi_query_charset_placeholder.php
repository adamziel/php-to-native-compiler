<?php
$multi_query = "mysqli_multi_query";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($multi_query) ? "yes" : "no";
echo "\n";
echo is_callable($multi_query) ? "callable" : "missing";
echo "\n";
echo mysqli_multi_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "charset-ok" : "charset-failed";
echo "\n";
echo $multi_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "dynamic" : "failed";
echo "\n";
echo mysqli_more_results($dbh) ? "more" : "done";
echo "\n";
echo mysqli_next_result($dbh) ? "next" : "done";
echo "\n";
echo mysqli_store_result($dbh) === false ? "no-pending" : "pending";
echo "\n";
echo mysqli_use_result($dbh) === false ? "no-use" : "using";
