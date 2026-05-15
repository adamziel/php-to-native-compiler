<?php
$server_version = "mysqli_get_server_version";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_get_server_info($dbh);
echo "\n";
echo mysqli_get_server_version($dbh);
echo "\n";
echo $server_version($dbh);
