<?php
$change = "mysqli_change_user";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($change) ? "yes" : "no";
echo "\n";
echo is_callable($change) ? "callable" : "missing";
echo "\n";
echo mysqli_change_user($dbh, "user", "pass", "wordpress") ? "changed" : "failed";
echo "\n";
echo $change($dbh, "user", "pass", null) ? "changed-null-db" : "failed";
echo "\n";
echo mysqli_ping($dbh) ? "still-open" : "closed";
