<?php
$handle = mysqli_init();
echo mysqli_ssl_set($handle, null, null, null, null, null) ? "nulls" : "failed";
echo "|";
echo mysqli_ssl_set($handle, "key.pem", "cert.pem", "ca.pem", "capath", "cipher") ? "strings" : "failed";
echo "|";
echo mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0) ? "connected" : "failed";
