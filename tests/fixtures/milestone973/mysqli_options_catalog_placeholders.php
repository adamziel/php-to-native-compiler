<?php
$handle = mysqli_init();
echo mysqli_options($handle, MYSQLI_OPT_CONNECT_TIMEOUT, 5) ? "connect" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_READ_TIMEOUT, 7) ? "read" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_INIT_COMMAND, "SET NAMES utf8mb4") ? "init" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_LOCAL_INFILE, true) ? "local" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_SSL_VERIFY_SERVER_CERT, false) ? "ssl" : "failed";
