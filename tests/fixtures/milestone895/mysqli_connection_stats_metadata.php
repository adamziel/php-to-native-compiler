<?php
$stats_call = "mysqli_get_connection_stats";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

$stats = mysqli_get_connection_stats($dbh);
$dynamic = $stats_call($dbh);

echo count($stats);
echo "\n";
echo $stats["bytes_sent"];
echo "\n";
echo $stats["bytes_received"];
echo "\n";
echo $stats["connect_success"];
echo "\n";
echo $stats["active_connections"];
echo "\n";
echo $dynamic["result_set_queries"];
