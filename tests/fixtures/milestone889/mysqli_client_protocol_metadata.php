<?php
$client = "mysqli_get_client_info";
$proto = "mysqli_get_proto_info";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_get_client_info();
echo "\n";
echo mysqli_get_client_info($dbh);
echo "\n";
echo $client(null);
echo "\n";
echo mysqli_get_proto_info($dbh);
echo "\n";
echo $proto($dbh);
