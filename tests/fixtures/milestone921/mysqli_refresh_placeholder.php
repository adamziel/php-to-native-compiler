<?php
$refresh = "mysqli_refresh";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($refresh) ? "yes" : "no";
echo "\n";
echo is_callable($refresh) ? "callable" : "missing";
echo "\n";
echo MYSQLI_REFRESH_REPLICA === MYSQLI_REFRESH_SLAVE ? "replica-alias" : "different";
echo "\n";
echo mysqli_refresh($dbh, MYSQLI_REFRESH_LOG | MYSQLI_REFRESH_TABLES) ? "refreshed" : "failed";
echo "\n";
echo $refresh($dbh, MYSQLI_REFRESH_STATUS | MYSQLI_REFRESH_THREADS | MYSQLI_REFRESH_BACKUP_LOG) ? "dynamic" : "failed";
echo "\n";
echo mysqli_ping($dbh) ? "still-open" : "closed";
